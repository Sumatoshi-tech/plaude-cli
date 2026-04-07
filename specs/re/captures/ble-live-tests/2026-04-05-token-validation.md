# BLE live token-validation tests — 2026-04-05

- **Device**: PLAUD NOTE V0095, MAC `D1:A6:DE:62:DF:14` (observed RPA)
- **Host**: Fedora 43, BlueZ 5.86
- **BLE client**: `bleak` (Python, BlueZ backend)
- **Goal**: empirically determine how strictly the device validates the
  `0x0001` auth token, and whether a valid token replayed from a
  non-phone BLE central works.

Scripts preserved under
[`scripts/`](scripts/) for reproducibility:
`plaud-test2.py`, `plaud-test2b.py`, `plaud-test2c.py`.

## Test 2a — minimal fake-token smoke test

**Script**: `scripts/plaud-test2.py`.

**Method**: connect, enable notifications on `0x2BB0` (handle `0x0011`
CCCD), write a hand-crafted `0x0001` auth frame with 32 ASCII `'0'`
bytes as the token, observe for 6 seconds.

**Wire payload sent** (38 bytes):

```
01 01 00 02 00 00  30 30 30 30 30 30 30 30 30 30 30 30 30 30 30 30
30 30 30 30 30 30 30 30 30 30 30 30 30 30 30 30
```

**Observed response** (notification on `0x2BB0`, ~4 s after write):

```
01 01 00 01 0a 00 03 00 01 01 00 00 56 5f 00 00
            ^^
            status byte = 0x01
```

**Result**: **device did not disconnect** for the full 6-second window.
Device acknowledged the write and produced an auth-response
notification with `status byte = 0x01`.

## Test 2b — post-fake-auth command probe

**Script**: `scripts/plaud-test2b.py`.

**Method**: same fake auth as 2a, then a standard SIG battery read plus
four follow-up vendor commands
(`GetDeviceName`, `GetState`, `0x0006`, `0x0009`).

**Results**:

| Probe | Result |
|---|---|
| SIG battery char `0x2A19` read | `0x64` = **100 %** — succeeded with NO vendor auth |
| `GetDeviceName` (0x6C) | **no response** |
| `GetState` (0x03) | **no response** |
| Opcode 0x06 | **no response** |
| Opcode 0x09 | **no response** |
| Connection state at end | **still connected** |

**Observations**:

1. The standard SIG Battery Service is fully readable without any
   vendor authentication. `plaud-transport-ble` should expose
   `plaude battery` as a free, no-auth command path.
2. Vendor commands issued after a failed auth are **silently
   ignored**: no response, no error frame, no disconnect. This is a
   soft-reject policy, distinct from a hard-disconnect policy.
3. The auth response for the fake token arrived late (~3.3 s after
   the write, after GetDeviceName had already been sent), which
   suggests the device runs some computation before replying.

## Test 2c — real-token replay from non-phone central

**Script**: `scripts/plaud-test2c.py`.

**Method**: extract the real auth frame bytes from the gitignored 0day
btsnoop log at `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.log`,
replay it verbatim from the laptop's BLE adapter, then fire three
vendor commands to confirm the session is usable.

**Wire payload sent**: the exact bytes the phone app sent during the
0day capture (content redacted from this document; see
[`../btsnoop/sanitisation.md`](../btsnoop/sanitisation.md)).

**Observed responses**:

```
[+5562 ms]  01 01 00 00 0a 00 03 00 01 01 00 00 56 5f 00 00        ← auth response
                    ^^
                    status byte = 0x00 (success)

[+5562 ms]  01 6c 00 50 4c 41 55 44 5f 4e 4f 54 45 00 00 00 00 …  ← "PLAUD_NOTE" ASCII
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                    literal GetDeviceName response

[+7346 ms]  01 03 00 01 00 00 00 01 04 00 00 00 00 00 00           ← GetState 15 B tuple

[+11426 ms] 01 06 00 00 00 86 8f 0e 00 00 00 00 00 88 8f 0e
            00 00 00 90 ab ce 36 00 00 00 00                       ← storage/counter tuple
```

All responses match the shapes observed in the real phone-side 0day
capture byte-for-byte (modulo the dynamic storage counter values).

**Result**: **full vendor-protocol access from a non-phone BLE central
using a replayed token**.

## Analysis

### What Test 2c proves about the auth model

1. **The token is a device-side secret, not a session key.**
   It is replayable verbatim and has no nonce, no challenge, no MAC
   binding, no IRK binding, no time component, no sequence number,
   no post-auth key derivation. The token is sent once at the start
   of a session and then the rest of the session is cleartext
   vendor-protocol traffic.

2. **The token is not bound to the originating phone.** Our replay
   came from a Linux laptop with a different BT adapter, a different
   central address, and no prior association with the device. The
   device accepted it identically to how it accepted it from the
   phone.

3. **There is no per-session derived state.** Subsequent command
   opcodes (`0x6C`, `0x03`, `0x06`) worked without sending any
   additional key material or sequence number. Whatever the device
   keeps in memory after auth is limited to a single boolean
   "authenticated" flag for the current BLE link.

4. **Auth status is encoded in byte 3 of the `0x01 0x01 0x00` response.**
   - `0x00` = success (Test 2c)
   - `0x01` = failure (Tests 2a and 2b)
   The remaining 13 bytes of the response (`0a 00 03 00 01 01 00 00 56 5f 00 00`)
   appear to be a versioning / capability tuple that is identical
   between success and failure. `0x5f56 = 24406` is a stable device
   firmware/capability identifier.

5. **The device's rejection policy is "silent soft reject".** On
   `status = 0x01`, the device keeps the connection alive, keeps
   the standard SIG services reachable (battery was read in Test 2b),
   but **silently drops every vendor opcode** it receives. There is
   no error frame. This is useful: a CLI that sends the wrong token
   gets a clear diagnostic (status byte `0x01`) and does not need to
   implement reconnect logic.

### What this means for the CLI

**The auth layer is a solved problem.** The CLI needs to:

1. Obtain the token **once** per physical Plaud device. Three UX paths:
   (a) `plaude auth bootstrap` — runs a fake peripheral, waits for
       the user's phone app to try to connect, captures the token
       from the first vendor-characteristic write. One command,
       one minute, once per device lifetime.
   (b) `plaude auth import <btsnoop.log>` — extract the token from a
       pre-existing Android HCI snoop log (what we did for this
       research).
   (c) `plaude auth set-token <hex>` — manual paste for users who
       extract the token themselves.
2. Store the token in the OS keyring (Linux Secret Service, macOS
   Keychain, Windows Credential Manager), with a file fallback at
   `~/.config/plaude/token` (mode 0600).
3. On every `plaude` invocation that uses BLE: fetch the token from
   the keyring, frame it as `01 01 00 02 00 00 <32-byte ASCII hex>`,
   write to characteristic `0x2BB1`, verify the response has
   `status = 0x00` (abort with a clear error if not).
4. Treat battery level as a free, no-auth capability — reachable
   even without a stored token.

### Firmware version caveat

These tests were run against the V0095 plaintext auth path
(hypothesis A in
[`../../apk-notes/3.14.0-620/auth-token.md`](../../apk-notes/3.14.0-620/auth-token.md)).
If a future firmware update switches the device into the
RSA + ChaCha20-Poly1305 handshake path
(observable as notification type `0xFE12 = STICK_PREHANDSHAKE_CNF`
from the device at connect time), this replay approach becomes
obsolete and we would need to implement the new handshake. That
code path is already fully spec'd from the APK analysis in
[`../../apk-notes/3.14.0-620/architecture.md`](../../apk-notes/3.14.0-620/architecture.md#mode-b--rsa--chacha20-poly1305-newer-firmware).

### The USB deprecation finding

During the APK string hunt we found the literal user-facing warning:

> *"After the update, USB file access and RAW file features will be
> permanently disabled. You can still access and export WAV recordings
> securely through the Plaud app."*

**Plaud is actively removing USB-MSC access in a firmware update.**
This confirms that BLE must be the primary transport for the CLI to
have any longevity. USB is demoted to an opportunistic fallback that
works on current firmware but cannot be relied upon for users whose
devices have been updated.

## Evidence

- [`scripts/plaud-test2.py`](scripts/plaud-test2.py) — Test 2a
- [`scripts/plaud-test2b.py`](scripts/plaud-test2b.py) — Test 2b
- [`scripts/plaud-test2c.py`](scripts/plaud-test2c.py) — Test 2c
- [`../btsnoop/2026-04-05-plaud-0day-pair.log`](../btsnoop/2026-04-05-plaud-0day-pair.log)
  (gitignored) — source of the real token used in Test 2c
- [`../btsnoop/sanitisation.md`](../btsnoop/sanitisation.md) — redaction policy
- [`../../apk-notes/3.14.0-620/ble-protocol.md`](../../apk-notes/3.14.0-620/ble-protocol.md)
  — source-backed opcode spec the tests referenced
