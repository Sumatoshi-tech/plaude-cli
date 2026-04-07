# btsnoop walkthrough — Plaud 0day re-pair + sync, 2026-04-05

**This document is a diff against
[`2026-04-05-plaud-sync-session.md`](2026-04-05-plaud-sync-session.md).**
Findings that repeat the earlier capture are not re-stated here; only new
or corrected facts appear below.

- **Date**: 2026-04-05
- **Raw log**: `2026-04-05-plaud-0day-pair.log` (gitignored), 3.5 MB,
  17 888 frames, 1907 ATT frames, 1298 s total time.
- **User action sequence**:
  1. Forgot the Plaud device on the phone (Plaud app + Android Bluetooth settings).
  2. Toggled Bluetooth off/on on the phone.
  3. Made a 2–3 minute recording and (separately) a 15 s recording on the device.
  4. Went through the Plaud app's full "add device" flow, authenticated, synced.
- **Cumulative log warning**: Samsung One UI's snoop log **did not rotate**
  on the BT toggle. This log contains the earlier resumption session
  (frames ~1428–2640, 31–98 s) followed by the 0day sessions (after 822 s).
  All analysis below starts from 822 s.

## 1. Session boundaries

| Session | Start | End | Duration | What |
|---|---|---|---|---|
| A (old resumption) | 31.4 s | 98.6 s | 67 s | Covered in the previous walkthrough. |
| *gap* | — | — | 723 s | No ATT. User was forgetting + re-pairing on the phone. |
| **B1** | 822.0 s | 835.0 s | 13 s | Fresh connect after "forget", discovery, CCCD, first auth, short metadata sweep. |
| *idle* | — | — | 346 s | No ATT. |
| **B2** | 1181.6 s | 1196.3 s | 15 s | Second connect, discovery, auth, full metadata sweep, `0x0026` config write, new opcode `0x0067`. |
| *idle* | — | — | 22 s | No ATT. **Strong candidate window for Wi-Fi Fast Transfer** of the 2–3 minute recording, which is invisible to btsnoop. |
| **C** | 1218.1 s | 1228.5 s | 10 s | Third connect, bulk transfer of ~76 kB over BLE (likely the 15 s recording's small ASR or a summary), `0x006D` session close. |

## 2. Correction: the Plaud Note has no BLE bonding

The previous walkthrough implied pairing/bonding was happening because
it wasn't visible in the resumption capture. This capture definitively
disproves that:

- `tshark -Y btsmp` returns **zero frames across the entire 3.5 MB log**.
- No L2CAP SMP channel is ever created.
- All ATT traffic is decrypted by `tshark` without keys — the link is
  **unencrypted**.
- The phone was genuinely unpaired before session B1 (the user removed
  the bond in both the Plaud app and Android Bluetooth settings). A
  standard BLE first-pair flow would have produced a cluster of SMP
  frames (Pairing Request, Public Key, DHKey Check, Pairing Random,
  Pairing Confirm, encryption start). None are present.

**Conclusion**: the Plaud Note's "authentication" is done purely at the
application layer via opcode `0x0001`, travelling over an unencrypted
LE connection. The earlier R0 observation that the device dropped our
unbonded connection was not a BLE bonding check — it was the device's
application logic disconnecting centrals that fail to send a valid
`0x0001` token within a short window after CCCD enable.

**Security implication**: every byte of Plaud BLE traffic — control
commands, metadata, file chunks — is visible in cleartext to any
passive sniffer within range. The `0x0001` token is the single secret
protecting the device. If an attacker learns a device's token once,
they can impersonate the Plaud app against that device indefinitely.

## 3. Correction: the auth token is static per device

The token sent in opcode `0x0001` is **byte-identical across all three
0day sessions (B1, B2, C) and identical to the earlier resumption
capture**.

```
01 01 00 02 00 00
62 34 62 34 38 63 32 31 30 37 34 66 38 39 64 32
38 37 63 30 31 65 39 66 34 62 31 66 66 61 62 37
```

Decoded: the 32 trailing bytes are ASCII hex representing the 128-bit
value `<AUTH_TOKEN_32HEX>`. The same value appeared in the resumption
session before the phone's bond was ever cleared, and reappeared three
more times after the bond was cleared and re-established. The token is
**not** a session-specific nonce, **not** a pairing-derived secret, and
**not** negotiated between peer and host — it is a **fixed per-device
value** that the Plaud Android app already knows by the time it opens a
BLE connection.

The app's sequence at first contact (observed in session B1) is:

```
822.01  …  ATT service + characteristic discovery (no reads of vendor chars)
822.66      Write Request 0x12 → CCCD on handle 0x0011 (enable notifications)
822.76      Write Response 0x13 ← CCCD ACK
822.78      Write Command 0x52 → 0x000D  |  01 01 00 02 00 00 <TOKEN>
824.89      Handle Value Notification    |  01 01 00 00 0a 00 03 00 01 01 00 00 56 5f 00 00
```

The token write is **the very first byte of application traffic** —
there is **no prior read** that could have seeded a computation. The app
must therefore either have the token pre-computed in persistent storage
(plausible if the app cached it during USB setup), or derive it
deterministically from device identifiers it already has from the scan
(local name and manufacturer data), or from a factory secret baked into
the Plaud APK.

Common hash derivations have been tested against the device serial,
factory-MAC-like bytes embedded in the manufacturer data, and the raw
manufacturer data payload. None match. **The algorithm is non-trivial
and is a hard dependency on `re-apk-dive`.**

## 4. Confirmed: handle `0x0011` is the vendor notify CCCD

Session B1's first writes enumerated descriptors with opcode `0x04`
(Find Information Request) and received `0x0011` and `0x0012` as
descriptors under the notify characteristic `0x0010`. The subsequent
`0x12` Write Request went to handle `0x0011` with value `0x0100`
(standard "enable notifications" CCCD write). This confirms the hint
from the earlier walkthrough: **handle `0x0011` is the CCCD descriptor
(`0x2902`) of the vendor notify characteristic**. Handle `0x0012` is
most likely another descriptor on the same characteristic (User
Description or Characteristic Extended Properties).

The `ble-gatt.md` table is updated accordingly.

## 5. New opcode: `0x0067`

Seen once, in session B2:

```
1187.64  01 08 00 02 17 00 00 00 00   # 0x08 with new type 0x02, field 0x17
1187.67  01 67 00 01                  # 0x67 with 1-byte arg 0x01
```

| Opcode | Request | Response | Notes |
|---|---|---|---|
| `0x0067` | `01 67 00 01` (1 B arg = `0x01`) | (not observed in isolation) | Immediately preceded by a new variant of `0x08` with `type=0x02, field=0x17`. Could be a "commit / start transfer" flag, a "mark as read" operation, or a mode switch. Name deferred to `re-apk-dive`. |

## 6. Clarified `0x0008` structure

Every observed `0x08` request is now explained by the following shape:

```
01 08 00  <type:u8>  <field:u8>  00  [trailer:u32 = 0xFFFFFFFF for "invalid"]
```

- **type** = `0x01` in every request of the first session. In session
  B2 we see one call with **type = `0x02`**. Likely a record-kind or
  page selector (e.g. "metadata page 1" vs "metadata page 2", or
  "NOTE recordings" vs "CALL recordings").
- **field** = `0x0F, 0x11, 0x13, 0x14, 0x17, 0x18, 0x1A` in the first
  session; B2 adds **`0x1B`**. Field ids enumerate metadata slots of
  a record.
- The trailing u32 is usually zero; in one B2 frame it is
  `0xFFFFFFFF`, which is the same "end/sentinel" value that appears
  in bulk offsets (§7). This byte is probably a "next chunk" cursor
  that returns `0xFFFFFFFF` when exhausted.

The updated `ble-commands.md` entry for `0x0008` is rewritten with this
full signature.

## 7. Bulk transfer: non-zero start offsets and end-of-stream sentinel

Session C's 947 bulk frames on magic `0x02` do **not** start at offset 0.
Sampled offsets:

```
Start of session C bulk burst:
  0x00003E80, 0x00003ED0, 0x00003F20, …    # contiguous, stepping by 0x50 = 80
…
Later in session C:
  0x000001E0, 0x00000230, …                 # resets to a much lower offset
…
Final bulk frame of session C:
  0xFFFFFFFF                                # end-of-stream sentinel
```

**New facts** added to the bulk frame spec:

- **Bulk transfers are range-addressable**: the phone can request an
  arbitrary `(file_id, offset, length)` window via opcode `0x001C` and
  the device replies with `0x02` frames carrying that exact range.
  Offsets do not have to start at 0.
- **Multiple transfers can occur in one session**: the offset stream
  can reset to a much lower value, meaning either a new file_id was
  requested or the same file was re-read from a different position.
- **End-of-stream sentinel**: the last frame of a transfer carries
  `offset = 0xFFFFFFFF` (4 bytes of `FF`) in its offset field. This is
  distinct from a normal chunk and should be interpreted as a
  terminator, not as a data chunk with `0xFFFFFFFF` bytes.

`ble-commands.md` §2 (bulk frame spec) is updated with these.

## 8. Fast Transfer window — circumstantial evidence

Between session B2 end (1196.3 s) and session C start (1218.1 s) there
is a **22-second silent window** on BLE. Session B2 ended with:

```
1196.34  01 26 00 c8 00 00 00 00 00 00 00 c8 00 00 00
```

— a `0x0026` config write carrying two `0xC8 = 200` values. In the old
capture, session A ended with a similar `0x0026` carrying two `0xB4 =
180` values.

**Hypothesis (not yet evidence-backed)**: `0x0026` is the Fast Transfer
trigger. The 2–3 minute recording made by the user is large (~12 MB WAV
+ ~720 kB ASR) and would take hours to transfer over BLE at the
observed ~9 kB/s. 22 seconds is a plausible duration for a Wi-Fi burst
on an open hotspot. Btsnoop cannot see Wi-Fi traffic, which explains
the silent window.

This hypothesis will be **tested** by a future `re-wifi-probe` run
where we watch the phone's Wi-Fi interface for Plaud hotspot
association during a sync. For now it lives in the backlog as a strong
conjecture, not as spec content.

## 9. Summary of updates to the spec

| Doc | Change |
|---|---|
| `ble-gatt.md` | Correction: **no BLE bonding**. Handle `0x0011` confirmed as vendor notify CCCD. |
| `ble-commands.md` | Correction on auth (static token, not nonce). `0x0008` structure rewritten. Bulk frame spec adds range-addressability + `0xFFFFFFFF` sentinel. New candidate opcode `0x0067`. |
| `overview.md` | Security note added: BLE traffic is cleartext. |
| `backlog.md` | Multiple rows resolved or updated, several new candidates added (Fast Transfer trigger, token derivation algorithm, bulk-range semantics, field ids). |

## 10. Sanitisation

See [`sanitisation.md`](sanitisation.md) for the current substitution
log. The auth token in this capture is the same value as before and is
redacted the same way. No new identifiers appear.
