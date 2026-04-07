# BLE R0 passive recon — PLAUD NOTE (partial)

- **Date**: 2026-04-05
- **Host**: Fedora 43, Linux 6.19.8-200.fc43.x86_64, BlueZ 5.86
- **Adapter**: `50:BB:B5:B9:93:AB` (public) on `hci0`
- **Target**: PLAUD_NOTE, firmware V0095 (per the paired USB baseline)
- **Outcome**: **partial** — advertisement data captured, GATT enumeration
  blocked by device-enforced bonding requirement.

## Summary

The Plaud Note is advertising LE-only with general discoverability. It
accepts the LE link-layer connection from an unbonded central, then
immediately terminates the link at the GATT-discovery stage. We were able
to read its advertisement record fully before the disconnect.

## Address and flags

| Field | Value | Notes |
|---|---|---|
| MAC (sample A) | `D1:A6:DE:62:DF:14` | Random address type |
| MAC (sample B) | `ED:8E:A4:99:0F:87` | Random address type |
| Local name | `PLAUD_NOTE` | Constant across both samples |
| Advertising flags | `0x06` | LE General Discoverable + BR/EDR Not Supported |
| Address type | `random` | Top 2 bits of both MACs = `11` |

**Both addresses appeared within the same 30-second scan window from the
same physical device**, which is strong evidence that the Plaud Note uses
**BLE privacy with Resolvable Private Address rotation**. Consequence:
MAC is not a stable identifier across sessions. Stable identity must come
from the local name, the manufacturer data key, and (post-bond) the IRK.

## Manufacturer data

```
ManufacturerData.Key: 0x0059 (89)
ManufacturerData.Value (26 bytes):
  02 78 03 04 56 5f 00 00 09 88 83 17 44 28 36 35
  88 84 0a 00 04 da 6d a7 ce 01 01
```

**Manufacturer ID `0x0059`** is assigned by the Bluetooth SIG to
**Nordic Semiconductor ASA**. Combined with the USB path's Realtek VID
(`0x0bda`), this is strong evidence of a **dual-SoC architecture**:

- **Nordic nRF5x** (likely nRF52840, matching BLE 5.2) handling BLE
  control + advertising.
- **Realtek RTL87xx** handling Wi-Fi Fast Transfer, USB-MSC, and storage.

The two radios share a common device serial (the same `888…` string
appears in the USB `iSerial` descriptor, in `MODEL.txt`, and embedded in
WAV `pad ` chunks). There must be an internal bus (UART? SPI?) between
the Nordic and Realtek silicon to exchange commands and transfer the
`.WAV`/`.ASR` file pairs when the phone requests Fast Transfer.

### Manufacturer-data payload — structural observations (single sample, no speculation)

Byte-offset view, no claims on field meanings yet:

| Offset | Bytes | Observation |
|---|---|---|
| `0x00..0x01` | `02 78` | Two bytes. Could be a protocol version, product id, or TLV header. |
| `0x02..0x05` | `03 04 56 5f` | Four bytes. Looks like a marker or TLV. |
| `0x06..0x07` | `00 00` | Possibly reserved / flags. |
| `0x08` | `09` | Could be a length byte preceding the following 9-byte block, or just a count. |
| `0x09..0x11` | `88 83 17 44 28 36 35 88 84` | Nine bytes. No clean match to the device serial in BCD or ASCII. Likely a device id / MAC-like identifier or a firmware-derived token. |
| `0x12..0x14` | `0a 00 04` | Three bytes. |
| `0x15..0x18` | `da 6d a7 ce` | Four bytes. High-entropy, plausibly a rotating counter or nonce used with RPA rotation. |
| `0x19..0x1a` | `01 01` | Trailer. Could be status bits (mode? battery bucket?). |

These are observations from one capture on one day with the device on the
NOTE slider position. **Zero speculation about field meanings** until we
have a second capture to diff against (different day, different slider
position, different charge level).

## Connection attempt

```
> connect ED:8E:A4:99:0F:87
Attempting to connect to ED:8E:A4:99:0F:87
[CHG] Device ED:8E:A4:99:0F:87 RSSI: 0xffffffa5 (-91)
[CHG] Device ED:8E:A4:99:0F:87 Connected: yes
Connection successful
[PLAUD_NOTE]> [SIGNAL] LE.Disconnected - org.bluez.Reason.Remote,
              Connection terminated by remote user
[SIGNAL] Disconnected - org.bluez.Reason.Remote,
              Connection terminated by remote user
[CHG] Device ED:8E:A4:99:0F:87 Connected: no
```

**Interpretation**: the device completed the LE link-layer connection and
then actively closed it. `org.bluez.Reason.Remote` means the disconnect
was initiated by the peer, not by a timeout on our side. GATT service
discovery did not complete before the close.

Conclusion: **the Plaud Note refuses GATT enumeration from an unbonded
central**. Full R0 coverage requires either:

1. **Bonding** (Just Works pairing from Linux) — skill contract requires
   explicit user authorisation. May or may not succeed depending on
   whether the device is in an app-initiated pairing-accept state.
2. **Dynamic capture** via `re-hci-capture` on the user's Android phone
   — observe the real app's bonding sequence and subsequent GATT
   discovery in an Android `btsnoop_hci.log`. Non-invasive.

User chose option (2). This capture is pinned as the best non-invasive
R0 result achievable without bonding.

## BlueZ session details

- **Discovery mode used**: `bluetoothctl scan on`, 30-second scripted window.
- **Privileges**: normal user, member of the `bluetooth` group. `btmgmt find`
  returned `Permission Denied` without `sudo`; not pursued.
- **`btmon`**: not captured (requires root / CAP_NET_ADMIN). Future R0 runs
  should capture `btmon` output alongside if sudo is authorised — it gives
  raw HCI events including extended adv PDUs and scan responses that
  `bluetoothctl` may filter.

## Implications for the roadmap

- **Backlog row `GATT baseline` → partial**. Advertisement data is
  resolved; service/characteristic enumeration is blocked on R2.
- **Dual-SoC architecture** must be reflected in `docs/protocol/overview.md`.
- **`plaud-transport-ble`** cannot rely on MAC for identity — it must
  match on local name and manufacturer-data key `0x0059`, and after bond,
  on the device's IRK.
- **`re-apk-dive`** should now look for *two* distinct manager classes:
  one Nordic-style BLE manager, one Realtek-style Wi-Fi/HTTP client. The
  command dispatcher likely lives on the BLE side with some commands
  proxying through to the Realtek side.

## Evidence references

- This file: first-party capture on V0095.
- [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`](../usb/2026-04-05-plaud-note-v0095-baseline.md)
  — USB enumeration showing Realtek VID/PID, confirming the dual-SoC hypothesis.
- [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md`](../usb/2026-04-05-plaud-note-v0095-first-recording.md)
  — WAV `pad ` chunk containing the shared device serial.
