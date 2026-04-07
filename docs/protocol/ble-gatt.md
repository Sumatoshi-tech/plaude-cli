# BLE GATT map — Plaud Note

## Status

**R0 + R2 complete for firmware V0095.** The service/characteristic
table below is reconstructed from the dynamic btsnoop capture of a real
Plaud-app sync session; advertising-level data is from the passive
BlueZ scan.

## Advertising profile

| Field | Value | Evidence |
|---|---|---|
| Local name | `PLAUD_NOTE` | [`2026-04-05-hci0-plaud-note-r0-partial.md`](../../specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md) |
| Address type | Random (BLE privacy, RPA rotation observed within one scan window) | same |
| Advertising flags | `0x06` — LE General Discoverable, BR/EDR Not Supported | same |
| Manufacturer ID | `0x0059` — **Nordic Semiconductor ASA** | same |
| Manufacturer data payload | 26 bytes, structure partially mapped, field meanings unknown on a single sample | same |
| Connectable | Yes | same |
| Link-layer encryption / BLE bonding | **None** — the device never initiates SMP pairing. All ATT traffic is cleartext. | [0day walkthrough §2](../../specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md#2-correction-the-plaud-note-has-no-ble-bonding) |
| Access control | **Application-layer auth only** — device disconnects any central that fails to send a valid `0x0001` token within ~1 s of enabling notifications on the vendor CCCD | same |

## Stable identification

Because the device uses Resolvable Private Address rotation, **MAC is not
a stable identifier**. A `plaud-transport-ble` implementation must match
candidates using:

1. Local name equal to `PLAUD_NOTE`, **and**
2. Manufacturer data key equal to `0x0059`.

After first-time bonding, the device's IRK resolves rotating RPAs back
to a single logical identity.

## Primary services

| Handle | Group end | UUID | Name | Evidence |
|---|---|---|---|---|
| `0x0001` | `0x0009` | `0x1800` | GAP | [btsnoop walkthrough §3](../../specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md#3-full-gatt-map) |
| `0x000A` | `0x000A` | `0x1801` | GATT | same |
| `0x000B` | `0x0012` | **`0x1910`** | **Plaud vendor service** (squatted 16-bit UUID) | same |
| `0x0013` | `0xFFFF` | `0x180F` | Battery | same |

**Important**: UUID `0x1910` is a 16-bit UUID not assigned by the
Bluetooth SIG. Plaud is using it as a pseudo-private vendor UUID in
place of the conformant 128-bit alternative. Non-conformant with the
BLE spec but interoperable with all major phones.

## Characteristics

### GAP service (`0x1800`)

| Char val handle | UUID | Props | Name |
|---|---|---|---|
| `0x0003` | `0x2A00` | Read, Write | Device Name |
| `0x0005` | `0x2A01` | Read | Appearance |
| `0x0007` | `0x2A04` | Read | Peripheral Preferred Connection Parameters |
| `0x0009` | `0x2AA6` | Read | Central Address Resolution |

### Plaud vendor service (`0x1910`)

| Char val handle | UUID | Props | Role |
|---|---|---|---|
| **`0x000D`** | **`0x2BB1`** | `0x0C` = Write + Write Without Response | **Control in (phone → device)** |
| **`0x0010`** | **`0x2BB0`** | `0x10` = Notify | **Control + bulk out (device → phone)** |
| `0x0011` | — | (descriptor `0x2902`, CCCD) | **Vendor notify CCCD** — phone writes `0x0100` here to subscribe. Confirmed by observing the ATT `0x12` Write Request + `0x13` Write Response pair on this exact handle in the 0day capture. |

- **`0x2BB0` and `0x2BB1`** are SIG-registered for BLE 5.1 Constant Tone
  Extension (direction-finding). Plaud is **squatting on them for their
  vendor protocol** — this is almost certainly deliberate obscurity.
  A BLE scanner that honours SIG semantics would completely mis-identify
  the device.

See [`ble-commands.md`](ble-commands.md) for the frame format and the
observed opcode dictionary that travels over this pair.

### Battery service (`0x180F`)

| Char val handle | UUID | Props | Name |
|---|---|---|---|
| `0x0015` | `0x2A19` | Read + Notify | Battery Level |

Standard SIG battery service. Our CLI can read battery level without
touching the vendor protocol at all.

## Architectural implication

Combined with USB VID `0x0bda` (Realtek) from the USB baseline, the
Nordic manufacturer ID on BLE confirms the **dual-SoC architecture**:
Nordic nRF5x handles BLE + the vendor opcode dispatcher, Realtek RTL87xx
handles Wi-Fi Fast Transfer + USB-MSC + flash storage. Opcodes observed
in section 5 of the btsnoop walkthrough that touch recording metadata
(`0x08`, `0x16`, `0x1A`, `0x1C`) must proxy across an internal UART/SPI
bridge between the two chips on the device side.

## Evidence

- [`specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md`](../../specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md)
  — R0 passive recon: advertising data, Nordic manufacturer ID, RPA rotation, pairing requirement.
- [`specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md`](../../specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md)
  — R2 dynamic capture: full GATT map, frame format, opcode dictionary,
  session timeline, sanitisation log.
