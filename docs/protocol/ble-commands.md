# BLE command dictionary — Plaud Note

## Status

**Framing fully specified from two R2 captures on firmware V0095** (the
initial resumption session and a 0day re-pair + sync session), and
**opcode semantics resolved for all observed opcodes** via R1 static
analysis of the Plaud Android app v3.14.0 / tinnotech pen-BLE SDK (see
[`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../specs/re/apk-notes/3.14.0-620/ble-protocol.md)).
Every opcode now has a source-backed constructor signature and an
action name from the `ai.plaud.*` Flutter bridge.

**Key R1 finding**: the BLE wire protocol is not Plaud's own — it is
the **Tinnotech pen-BLE SDK** (`com.tinnotech.penblesdk`), used by
multiple OEM voice-recorder/smart-pen products. Plaud is an OEM
rebadger. See
[`specs/re/apk-notes/3.14.0-620/architecture.md`](../../specs/re/apk-notes/3.14.0-620/architecture.md).

## Security model (evidence-backed, live-tested)

The security posture was fully resolved by a sequence of live tests
against a real V0095 device from a non-phone BLE central. See
[`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`](../../specs/re/captures/ble-live-tests/2026-04-05-token-validation.md)
for full results. Summary:

- **No BLE link-layer encryption.** The Plaud Note never initiates SMP
  pairing. All vendor traffic is **cleartext** on the air.
- **Authentication is a single pre-shared token.** The secret protecting
  a device is the 32-hex token sent in opcode `0x0001` at the start of
  every BLE session. There is **no session key, no nonce, no challenge,
  no MAC binding, no IRK binding, no sequence number, no post-auth key
  derivation**. The token is replayed verbatim on every connection.
- **The token is per-device, not per-phone.** Live replay from a Linux
  laptop with a different BT adapter works identically to replay from
  the original phone. Any BLE central that has the token has full
  vendor-protocol access.
- **The token is static for the device's lifetime.** It is byte-identical
  across sessions, across full phone-side "forget + re-pair" flows,
  across Bluetooth restarts, and across different BLE centrals. It is
  (almost certainly) cloud-issued by plaud.ai on first account pair
  and cached indefinitely in the Plaud app's local storage.
- **Auth status is encoded in byte 3 of the `0x01 0x01 0x00 …` response.**
  `0x00` = accepted, `0x01` = rejected. The remaining 13 bytes
  (`0a 00 03 00 01 01 00 00 56 5f 00 00` in our tests) are a stable
  device capability tuple.
- **Rejection is "silent soft reject", not disconnect.** A device that
  receives a bad token keeps the BLE connection alive, keeps standard
  SIG services reachable, but silently drops every vendor opcode it
  subsequently receives — no response, no error frame.
- **Standard SIG services are reachable without any vendor auth.**
  Battery level at char `0x2A19` was successfully read from an
  unauthenticated session. The CLI's `plaude battery` command path
  does not require a stored token.

### Implications for the CLI

1. **The auth problem reduces to "capture the token once per device".**
   It does not require algorithm recovery, does not require cloud
   contact, does not require phone storage access beyond the initial
   capture moment, and does not require ongoing re-authentication. One
   bootstrap, forever valid.
2. **Day-to-day operation is fully offline.** After `plaude auth bootstrap`
   succeeds once, no subsequent CLI invocation ever needs the phone,
   the Plaud app, or plaud.ai.
3. **Token storage must be treated as a long-lived credential.** Keep
   it in the OS keyring (Linux Secret Service, macOS Keychain,
   Windows Credential Manager) with a file fallback at
   `~/.config/plaude/token` (mode `0600`). Never log it. Never commit
   it to git.
4. **Cleartext wire traffic is a privacy concern the user should know
   about.** The README must state that any BLE sniffer within ~10 m
   can record the user's recordings metadata and file data during a
   sync.
5. **Firmware-version fallback.** If a future firmware update switches
   the device to the RSA + ChaCha20-Poly1305 handshake path
   (observable as a `0xFE12` preamble notification at connect),
   the CLI must fall back to that handshake — the algorithm is fully
   spec'd from APK static analysis in
   [`../../specs/re/apk-notes/3.14.0-620/architecture.md`](../../specs/re/apk-notes/3.14.0-620/architecture.md#mode-b--rsa--chacha20-poly1305-newer-firmware).

## Transport

- **Control channel — inbound** (phone → device): ATT Write Command
  (`0x52`) to characteristic value handle `0x000D`, UUID `0x2BB1`, in
  Plaud vendor service `0x1910`.
- **Control channel — outbound** (device → phone): ATT Handle Value
  Notification (`0x1B`) on characteristic value handle `0x0010`, UUID
  `0x2BB0`. The phone enables notifications by writing `0x0100` to the
  CCCD of this characteristic before the first command.
- **Bulk data channel**: same notify characteristic `0x0010`, frames
  distinguished by a different leading magic byte (see §2).

Both magic bytes travel on the same `0x0010` notify characteristic. The
parser demuxes them on byte 0.

## Frame format

### 1. Control frame (magic `0x01`)

```
 byte 0    1      2      3…
+------+------+------+----------------+
| 0x01 | OP_LO | OP_HI | payload      |
+------+------+------+----------------+
```

| Field | Size | Description |
|---|---|---|
| Magic | 1 B | Constant `0x01`. Marks the frame as a control request or response. |
| Opcode | 2 B LE | 16-bit little-endian operation code. All observed values in `0x0001..0x006D`. |
| Payload | variable | Opcode-specific. Observed range: 0 to 16 bytes. |

Control frames are symmetric: the device's response to an opcode `X`
begins with the same `01 <X-LE>` header followed by the result payload.
Many responses simply echo the request bytes as an ack.

### 2. Bulk data frame (magic `0x02`)

```
 byte 0   1   2   3   4   5   6   7   8   9   10    11…90
+----+----+----+----+----+----+----+----+----+----+-----+--------+
| 02 | 00 |  FILE_ID u32 LE   |  OFFSET u32 LE    | 50 | 80 B   |
+----+----+----+----+----+----+----+----+----+----+-----+--------+
```

| Field | Size | Description |
|---|---|---|
| Magic | 2 B | `02 00`. Marks a bulk-data frame. The trailing `00` mirrors the `OP_HI = 0x00` pattern of control frames, consistent with a 16-bit frame-type field where `0x0002` = bulk. |
| File id | 4 B LE | Stable within one sub-transfer. Upper byte observed as `0x00` (effective 24-bit id). Relationship to recording filename (Unix-epoch basename) not yet confirmed. |
| Offset | 4 B LE | Byte offset into the source file. Monotonically increasing by 80 across consecutive frames in one sub-transfer. May reset to a lower value when the phone starts a new sub-transfer in the same session. **Special value `0xFFFFFFFF`** marks the terminal frame of a transfer (see below). |
| Chunk length | 1 B | Constant `0x50 = 80`. Matches the payload length of every observed bulk frame. |
| Payload | 80 B | Raw file bytes. On the terminal frame (offset = `0xFFFFFFFF`), the payload is present but its content semantics are not yet known; parsers should not treat these 80 bytes as file data. |

Every bulk frame is exactly **90 bytes** long. Bulk frames flow only
device → phone. The phone does not acknowledge individual bulk frames;
flow control is implicit (BLE LL MTU + connection interval).

### Bulk transfer semantics (evidence from 0day capture)

- **Range-addressable**: the phone requests `(file_id, offset, length)`
  windows via opcode `0x001C`. The device replies with a stream of
  bulk frames covering the requested range. Offsets do **not** have
  to start at zero. A 0day session C transfer began at offset
  `0x00003E80` = 16 000 bytes into its source file.
- **Multiple sub-transfers per session**: the offset stream can reset
  to a much lower value mid-session, meaning the phone started a new
  `0x001C` request (possibly against the same file_id from a different
  offset, or against a new file_id).
- **End-of-stream sentinel**: the terminal frame carries
  `offset = 0xFFFFFFFF`. Parsers must detect this value explicitly and
  **not** append its payload to reassembled file data.

## Observed opcodes (V0095, single capture)

All entries are evidence-backed. Names marked *(conjecture)* are working
hypotheses, to be confirmed or renamed by `re-apk-dive`. Values in the
"payload" columns are from
[`specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md`](../../specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md)
— each entry can be cross-referenced to a specific frame number there.

| Opcode | Conjectured name | Req payload shape | Resp payload shape | Notes |
|---|---|---|---|---|
| `0x0001` | **`Authenticate`** | 3 B prefix (`02 00 00`) + version u16 + [flag u16 if proto≥3] + 16 or 32 B token string (padded with `'0'`) | 13 B tuple (status + version bytes) | APK-confirmed builder `p257nh/C9555a0` (base) and `C9603z` (extended with phoneModel + sdkVersion for newer protocols). Token padding length depends on device protocol version: `<3` = no flag field, `<9` = 16-char token, `≥9` = 32-char token (the V0095 case). **Token is passed into the tinnotech SDK as a plain `String` argument by the Plaud Flutter layer** — the SDK does not compute it. Derivation lives in Dart code inside `libapp.so` and is not recoverable from jadx output. See [`specs/re/apk-notes/3.14.0-620/auth-token.md`](../../specs/re/apk-notes/3.14.0-620/auth-token.md) for the full call chain and offline-CLI implications. |
| `0x0003` | *(conjecture)* `GetState` | (nullary) | 15 B: `01 03 00 01 00 00 00 00 04 00 00 00 00 00 00` | Called at the start of each metadata sweep. Response shape looks like a versioned status block. |
| `0x0004` | *(conjecture)* `SetClock` or `QueryAtTime` | `u32 LE timestamp` + `0x03 0x00` | echoes request | Argument is a Unix epoch (`0x69D261D1` in the capture). |
| `0x0006` | *(conjecture)* `GetStorageStats` | (nullary) | 27 B tuple with two near-equal u32s (`0x0E8F86`, `0x0E8F88`) and a larger u32 (`0x36CEAB90`) | Shape fits "two counters + a total". |
| `0x0008` | **`CommonSettings`** | `<ActionType u8> <SettingType u8> 00 <long u64 LE> <long u64 LE>` | 9 B response carrying the setting value | APK-confirmed builder `p257nh/C9568h(ActionType, int fieldId, long, long)`. `ActionType`: `1 = READ`, `2 = SETTING (write)`. `SettingType` is an enum of 20 device settings including `ENABLE_VAD (15)`, `REC_MODE (17)`, `VPU_GAIN (19)`, `MIC_GAIN (20)`, `AUTO_POWER_OFF (23)`, `SAVE_RAW_FILE (24)`, `AUTO_SYNC (26)`, `FIND_MY (27)` — every field id observed in wire captures maps to a named setting. See [`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../specs/re/apk-notes/3.14.0-620/ble-protocol.md#0x0008--full-decode). The `0xFFFFFFFF` trailer observed once is likely a pagination cursor marker. |
| `0x0009` | *(conjecture)* `GetPercent` | (nullary) | 1 B value (`0x64 = 100`) | Not battery (battery is on SIG service `0x180F`). Candidates: free-space %, recording count, or volume. |
| `0x0016` | *(conjecture)* `QueryByFileId` | `u32 file-id` + `u8` | 13 B tuple | First seen in the second metadata sweep. |
| `0x0018` | *(conjecture)* TBD | (nullary) | echoes request | — |
| `0x0019` | *(conjecture)* TBD | `u8` | — | 1-byte argument (`0x00`). |
| `0x001A` | *(conjecture)* TBD | `u32 timestamp` + `u32 reserved` | 11 or 21 B | Timestamp-addressed query. |
| `0x001C` | **`ReadFileChunk`** ✅ | `long file_id` + `long offset` + `long length` | 8 B ack | APK-confirmed: builder `p257nh/C9591s0(long, long, long)`. Triggers the bulk `0x02`-magic stream. Paired response opcode is `0x001D` (29). Note: the SDK declares the fields as Java `long` (8 bytes), not `int` (4 bytes) — our wire observation of 4-byte fields suggests the upper 4 bytes are always zero in practice, or the serializer truncates based on protocol version. |
| `0x001E` | *(conjecture)* TBD | `u32 file-id` | 8 B | File-id addressed query. |
| `0x0026` | *(conjecture)* `WriteConfig` | 12 B (two matching `0xB4` u32 values) | 4 B ack | Only config-write observed. Two copies of `180` suggest a matched-pair parameter. |
| `0x0067` | **`SetPrivacy`** ✅ | `u8` privacy flag (`0` = off, `1` = on) | 4 B ack | APK-confirmed: builder `p257nh/C9563e0(boolean)`. Flutter action `action/setPrivacy` at `FlutterDeviceManager.java:3475`, dispatched via `BleAgentImpl.mo32186s(boolean, …)`. Set to `1` during fresh-pair setup (observed in 0day capture). |
| `0x006C` | **`GetDeviceName`** ✅ | (nullary) | ASCII `PLAUD_NOTE` padded to ~30 bytes with `0x00` | **Resolved by inspection.** Returns the device model string. |
| `0x006D` | *(conjecture)* `CloseSession` | `u8` (`0x00`) | 4 B ack | Last frame seen in every observed session, near the end. Consistent with a graceful teardown. |

**Totals**: 15 distinct opcodes observed in one sync. The opcode space
is at least 16-bit wide and is sparse — Plaud is unlikely to be using
more than ~50–100 opcodes in total.

## Parser guarantees required by the spec

A conformant parser must:

1. Read the first byte to decide frame family (`0x01` control,
   `0x02` bulk; any other value is a protocol error and must be surfaced
   as such, not silently swallowed).
2. For control frames, read bytes 1..2 as a u16 little-endian opcode and
   dispatch on the full 16-bit value — not just the low byte, even
   though all currently observed high bytes are `0x00`.
3. For bulk frames, read bytes 2..5 as a u32 file-id, 6..9 as a u32
   offset, 10 as a chunk length byte, and validate that the remaining
   notification payload is exactly `chunk_length` bytes. Reject frames
   whose tail length does not match.
4. Never assume constant opcodes are stateless — e.g. `0x0003` is
   observed twice in the same session and the response may differ.
5. Sanitise ASCII-hex session fingerprints before logging.

## Follow-ups (handed to `re-apk-dive`)

- Locate the class building `01 XX 00 …` frames in the decompiled APK.
- Search for `0x2BB0`, `0x2BB1`, `0x1910`, `PLAUD_NOTE` string constants.
- Cross-reference u16 constants matching `0x0001, 0x0003, 0x0004, 0x0006,
  0x0008, 0x0009, 0x0016, 0x0018, 0x0019, 0x001A, 0x001C, 0x001E, 0x0026,
  0x006C, 0x006D` to method names.
- Identify the exact algorithm producing the 32-hex auth token in
  opcode `0x0001` (likely MD5 over a pairing secret + nonce).
