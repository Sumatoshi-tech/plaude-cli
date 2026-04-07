# btsnoop walkthrough — Plaud sync session, 2026-04-05

- **Date**: 2026-04-05
- **Phone**: Samsung S25 Ultra (`R5CY214FQHM`), One UI
- **Phone BT address**: `8C:C5:D0:9C:1F:AB` (matches `SamsungElect_9c:1f:ab` in pcap)
- **Device**: PLAUD NOTE, firmware V0095
- **Device BT address in session**: `D1:A6:DE:62:DF:14` (one of the two RPAs
  observed in R0; stable for the entire 67-second device conversation)
- **User action sequence**: open Plaud app → wait for connect → tap sync on
  the single test recording → wait for sync to finish → close app.
- **Raw log**: `2026-04-05-plaud-sync-session.log` (gitignored), 488 KB,
  BTSnoop v1 / HCI UART (H4), 2962 frames over 170.7 s.

## 1. High-level protocol shape

From `tshark -z io,phs`:

```
frame                         2962  416647 B
  bluetooth                   2962  416647 B
    hci_h4                    2962  416647 B
      bthci_cmd                285    7973 B   host → controller
      bthci_evt               1027   37185 B   controller → host
      bthci_acl               1650  371489 B   ACL data
        btl2cap               1644  371073 B
          btatt                544   45906 B   ← the interesting layer
```

From `tshark -z conv,bluetooth` (Plaud side only):

```
SamsungElect_9c:1f:ab ↔ d1:a6:de:62:df:14
  phone → device:  55 frames  990 B
  device → phone: 497 frames  45 kB
  start:  31.357 s
  duration: 67.29 s
```

## 2. Key finding: no SMP frames in this capture

`tshark -Y btsmp` returns **zero frames**. The phone and device were
already bonded from a prior setup; this session resumed the bond via
HCI LE Start Encryption below the ATT layer. The initial pairing method
(Just Works / Passkey / Numeric Comparison / OOB) is **not observable
from this capture**. Capturing it requires a second capture taken during
a fresh "forget device + re-pair" flow on the phone — deferred as a
backlog item, not a blocker.

## 3. Full GATT map

Reconstructed from the service-discovery ATT traffic (`0x10/0x11` and
`0x08/0x09` opcodes).

### Primary services

| Handle | Group end | UUID | Name |
|---|---|---|---|
| `0x0001` | `0x0009` | `0x1800` | GAP |
| `0x000A` | `0x000A` | `0x1801` | GATT |
| `0x000B` | `0x0012` | **`0x1910`** | **Vendor — Plaud control service** |
| `0x0013` | `0xFFFF` | `0x180F` | Battery |

**Observation**: `0x1910` is a 16-bit UUID. The Bluetooth SIG registry
does not assign this value to any standard service. Plaud is squatting
on an unassigned 16-bit UUID instead of registering a 128-bit vendor
UUID as the BLE spec requires for vendor services. Non-conformant but
functional; phones treat UUIDs as opaque.

### GAP service (0x1800)

| Handle | Char val handle | UUID | Props | Name |
|---|---|---|---|---|
| `0x0002` | `0x0003` | `0x2A00` | Read, Write | Device Name |
| `0x0004` | `0x0005` | `0x2A01` | Read | Appearance |
| `0x0006` | `0x0007` | `0x2A04` | Read | Peripheral Preferred Connection Parameters |
| `0x0008` | `0x0009` | `0x2AA6` | Read | Central Address Resolution |

### Plaud vendor service (0x1910)

| Handle | Char val handle | UUID | Props | Role |
|---|---|---|---|---|
| `0x000C` | **`0x000D`** | **`0x2BB1`** | `0x0C` = Write + Write Without Response | **CONTROL in (phone → device)** |
| `0x000F` | **`0x0010`** | **`0x2BB0`** | `0x10` = Notify | **CONTROL/BULK out (device → phone)** |

**Observation**: Plaud reuses `0x2BB0` and `0x2BB1`, which are SIG-registered
for BLE 5.1 "Constant Tone Extension" direction-finding characteristics
(Advertising Constant Tone Extension Interval / Transmit Duration). The
Plaud Note is not a direction-finding device — these UUIDs are being
**squatted on for obscurity**. A BLE scanner that assumes SIG semantics
would completely mis-identify the device's functionality. This is
almost certainly intentional.

One handle in the vendor service range (`0x000E` or `0x0011`) is
unaccounted for in the Read-By-Type sweep and was probably enumerated
via the subsequent Find Information Request / Response pair (opcodes
`0x04`/`0x05`). It is most likely a **Client Characteristic Configuration
Descriptor (CCCD, `0x2902`)** on the notify characteristic `0x0010`;
the phone must have written to it to enable notifications, which matches
the single `0x12` Write Request / `0x13` Write Response pair observed
in the opcode histogram.

### Battery service (0x180F)

| Handle | Char val handle | UUID | Props | Name |
|---|---|---|---|---|
| `0x0014` | `0x0015` | `0x2A19` | Read + Notify | Battery Level |

Standard. `0x0016` is almost certainly the CCCD for battery notifications.

## 4. Frame format (vendor protocol, magic-byte demultiplexed)

Every byte on the vendor control channel falls into one of two frame
types, distinguished by the **first byte (magic)**:

### 4a. Control frame (magic `0x01`)

```
 0    1    2    3…
+----+----+----+----...
| 01 | OP_LO | OP_HI | PAYLOAD |
+----+----+----+----...
```

- **Byte 0**: constant `0x01` (control magic).
- **Bytes 1..2**: 16-bit **little-endian opcode**. All observed opcodes
  have `OP_HI = 0x00`, so the effective opcode is `0x0001..0x006D`.
- **Bytes 3..**: opcode-specific payload, variable length.

Control frames travel in **both directions** over the same vendor service:

- **Phone → device**: Write Command (ATT opcode `0x52`) to handle `0x000D`.
- **Device → phone**: Handle Value Notification (ATT opcode `0x1B`) on
  handle `0x0010`. Typically echoes the request opcode followed by the
  result payload.

### 4b. Bulk data frame (magic `0x02`)

```
 0    1    2    3    4    5    6    7    8    9     10…
+----+----+----+----+----+----+----+----+----+----+-------+
| 02 | 00 |    FILE-ID (u32 LE)  |     OFFSET (u32 LE)  | 50 | …80 B… |
+----+----+----+----+----+----+----+----+----+----+-------+
```

- **Bytes 0..1**: `02 00` — bulk magic + reserved byte (mirrors the
  `01 XX 00` pattern of control frames, so `00` is probably the high
  byte of a u16 "frame type" = `0x0002`).
- **Bytes 2..5**: **4-byte file identifier**, little-endian. Constant
  across the entire 432-frame stream observed in this session. Upper
  byte was `0x00`, so the effective ID fits in 24 bits.
- **Bytes 6..9**: **4-byte chunk offset**, little-endian. Monotonically
  increasing by exactly `0x50 = 80` across consecutive frames.
- **Byte 10**: `0x50` — **chunk length = 80**. Constant.
- **Bytes 11..90**: **80 bytes of payload data**.

Every bulk frame is exactly **90 bytes** long (10-byte header + 80-byte
payload). Bulk frames travel only in the **device → phone** direction
as notifications on handle `0x0010`.

**Throughput**: 432 × 80 = **34,560 bytes** of payload transferred over
the 67-second device conversation. Effective goodput ≈ 515 B/s on the
control channel, which is very low — consistent with the device using
BLE only for small files / metadata and deferring to Wi-Fi Fast Transfer
for anything large. The 1.08 MB WAV from the test recording was **not
transferred** in this session; only a smaller artifact was, possibly the
`.ASR` sidecar (69 kB) in part, a summary, or a preview. Identifying the
exact file is a follow-up after `re-apk-dive`.

**Open question**: is the 4-byte file-id actually related to the
Unix-epoch basename of the recording (`1775393534`)? The first three
hex bytes of the file-id match the upper three bytes of neighbouring
timestamps, but a one-to-one mapping has not been confirmed.

## 5. Control-channel opcode dictionary (observed candidates)

**Status**: these are **candidates** — locations and structure are
evidence-backed, but field meanings will be resolved only after
`re-apk-dive` cross-references them to the Plaud Android app source.
The naming below is strictly conjectural and is marked as such.

| Opcode (LE) | Req sig | Resp sig | Freq | Example req | Example resp | Conjectured meaning |
|---|---|---|---|---|---|---|
| `0x0001` | `01 01 00` + 3 B prefix + 32 B ASCII-hex | `01 01 00 00 0a 00 03 00 01 01 00 00 56 5f 00 00` | 1 | (redacted — see sanitisation.md) | response carries session/version tuples | **Auth handshake**, first write after connect. Request carries a 128-bit MD5-style fingerprint in ASCII-hex. Response returns 13 bytes that look like `(status, N?, N?, version bytes)`. |
| `0x0003` | `01 03 00` (nullary) | `01 03 00 01 00 00 00 00 04 00 00 00 00 00 00` (15 B) | 3 | — | status/version tuple | Ping or get-state. Response has the shape of a versioned status block. |
| `0x0004` | `01 04 00` + u32 LE + u16 | echo of request | 2 | `01 04 00 d1 61 d2 69 03 00` | same | Takes a **Unix timestamp** (`0x69D261D1`). Possibly "set device clock" or "query records at/after time". Device echoes the request as ack. |
| `0x0006` | `01 06 00` (nullary) | 27-byte tuple incl. `86 8f 0e 00`, `88 8f 0e 00`, `90 ab ce 36 00 00 00` | 2 | — | `01 06 00 00 00 86 8f 0e 00 00 00 00 00 88 8f 0e 00 00 00 90 ab ce 36 00 00 00 00` | Likely storage stats or recording counters: two near-equal u32 values (`0x0e8f86` and `0x0e8f88`) plus a larger value (`0x36ceab90` ≈ 920 M). |
| `0x0008` | `01 08 00` + **u16 field id** + u16 | 9 B response carrying value | 9 | `01 08 00 01 0f 00` | `01 08 00 0f 00 00 00 00 00` | **Indexed get** — the commonest command by count. Field ids seen: `0x0f`, `0x11`, `0x13`, `0x14`, `0x17`, `0x18`, `0x1a`. Likely reads different fields of a recording-metadata record. The repeated pattern at 35 s and 82 s suggests the phone iterates the same field set twice. |
| `0x0009` | `01 09 00` (nullary) | `01 09 00 64` | 1 | — | value = 0x64 = 100 | 1-byte percentage. Plausibly recording count, free-space %, or similar. Distinct from standard Battery Level (which is on its own SIG service). |
| `0x0016` | `01 16 00 00` + u32 LE | `01 16 00 00 62 d2 69 00 00 00 00 04 01` | 1 | `01 16 00 00 62 d2 69 00` | 13 B tuple | File-id addressed query. Response tail `04 01` may be a status/type. |
| `0x0018` | `01 18 00` (nullary) | 4-byte response | 2 | — | `01 18 00` echoed | — |
| `0x0019` | `01 19 00 00` | — | 1 | `01 19 00 00` | — | — |
| `0x001A` | `01 1a 00` + u32 + u32 | 11 B or 21 B response | 4 | `01 1a 00 d2 61 d2 69 00 00 00 00` | `01 1a 00 d2 61 d2 69 00 00 00 00` (echo) | Timestamp-addressed query with a 4-byte reserved trailer. |
| `0x001C` | `01 1c 00` + u32 file-id + u32 offset + u32 length | 8 B response | 3 | `01 1c 00 00 62 d2 69 b0 68 00 00 20 76 00 00` | `01 1c 00 …` | **Strong candidate for "read chunk of file"** — the request carries a file-id, an offset (`0x68B0` = 26800), and a length (`0x7620` = 30240). Likely the trigger for the subsequent bulk transfer on magic `0x02`. |
| `0x001E` | `01 1e 00` + u32 | 8 B response | 1 | `01 1e 00 00 62 d2 69` | `01 1e 00 …` | File-id addressed query. |
| `0x0026` | `01 26 00` + 12 B | 4 B response | 1 | `01 26 00 b4 00 00 00 00 00 00 00 b4 00 00 00` | `01 26 00 …` | Config write. Two copies of `0xB4 = 180` — might be a pair of matched parameters. |
| `0x006C` | `01 6c 00` (nullary) | `01 6c 00 50 4c 41 55 44 5f 4e 4f 54 45 00 …` | 2 | — | ASCII **`PLAUD_NOTE`** padded to ~30 bytes | **Get device name**. Resolved. |
| `0x006D` | `01 6d 00 00` | 4 B response | 1 | `01 6d 00 00` | `01 6d 00 00` | Nullary-with-zero-arg. Session teardown candidate (last frame seen at 98.6 s, near end of conversation). |

**Totals**: 15 distinct opcodes, 35 control writes, 39 control responses
(device responses include a few unsolicited or multi-part replies), 432
bulk frames.

## 6. Session phases (time-ordered)

Extracted from the ATT timeline in `2026-04-05-plaud-sync-session.att.csv`.

| Phase | Frames | Relative time | What happened |
|---|---|---|---|
| **Discovery** | 1428–1467 | 31.36–31.74 s | MTU exchange, primary service discovery (paginated twice), char discovery inside GAP and vendor service, battery service discovery. |
| **Auth** | 1500 | 31.92 s | First write on `0x000D`: 32-byte ASCII-hex token. Device accepts — no disconnect. |
| **Metadata sweep #1** | 1537–1622 | 34.33–35.68 s | Phone issues opcodes `0x04`, `0x09`, `0x03`, `0x08` (7×), `0x6C`, `0x06`, `0x1A`. Appears to fetch every metadata field of the single record on the device. |
| **Idle** | — | 35.68–58.43 s | ~23 s silence on the control channel. Phone is presumably updating UI / asking user for confirmation to sync. |
| **Retry opcode 0x08** | 1789 | 58.43 s | Phone repeats `01 08 00 01 1a 00` once. Possibly a keep-alive. |
| **Metadata sweep #2 + chunk requests** | 1939–2630 | 82.55–98.62 s | Repeat of opcodes `0x03`, `0x16`, `0x08` (4×), `0x1C` (3×), `0x26`, then the bulk stream (432 `0x02` frames, phase 83.6–93.0 s), then `0x06`, `0x1A`, `0x1E`, `0x6D`. |
| **Teardown** | 2630+ | after 98.6 s | Trailing HCI events, no more ATT traffic. |

## 7. Candidate opcodes handoff to `re-apk-dive`

The 15 opcodes in section 5 are the backlog input for the APK pass.
`re-apk-dive` should search the decompiled Plaud Android app for:

- The constant byte `0x01` used as a frame header when building BLE writes.
- The byte `0x02` used as a bulk-frame discriminator when parsing notifications.
- u16 little-endian constants matching the opcode values: `0x0001, 0x0003,
  0x0004, 0x0006, 0x0008, 0x0009, 0x0016, 0x0018, 0x0019, 0x001A, 0x001C,
  0x001E, 0x0026, 0x006C, 0x006D`.
- The characteristic UUIDs `0x2BB0` and `0x2BB1` (expected to appear in
  a class that manages the vendor BLE service).
- The string `PLAUD_NOTE` used as a device-name matcher.

When these land in a single class, that class is the BLE command
dispatcher and its method names will give proper names to every opcode
in one pass.

## 8. Sanitisation

See [`sanitisation.md`](sanitisation.md) for the full substitution log.
Summary: the first control-write payload contains a 32-character ASCII-hex
auth token tied to the current phone↔device pairing. It has been replaced
with `<AUTH_TOKEN_32HEX>` in this walkthrough. The device serial does
**not** appear in any ATT payload and needed no substitution.

The raw `.log` and the `.att.csv` both contain the unredacted token and
are gitignored under `specs/re/captures/btsnoop/*.log` and `*.att.csv`.
They are kept locally for re-analysis but **must not be committed**.
