# USB baseline — PLAUD NOTE, firmware V0095

- **Date**: 2026-04-05
- **Host**: Fedora 43, Linux 6.19.8-200.fc43.x86_64
- **Device**: PLAUD NOTE (original, credit-card form factor)
- **Firmware**: `V0095@00:47:14 Feb 28 2024` (from `MODEL.txt` on the device)
- **Serial**: `<SERIAL>` (redacted; matches USB `iSerial` descriptor byte-for-byte)
- **Connection**: magnetic pogo-pin charger cable, USB-A into laptop
- **Trigger**: "Access via USB" toggle enabled once in the Plaud phone app

## USB enumeration

```
Bus 005 Device 038: ID 0bda:4042 Realtek Semiconductor Corp. NOTE
```

- **idVendor**: `0x0bda` — **Realtek Semiconductor Corp.**
- **idProduct**: `0x4042` — product string `NOTE`
- **iManufacturer**: `PLAUD`
- **iProduct**: `NOTE`
- **iSerial**: `<SERIAL>` (same value as `MODEL.txt`)
- **bcdUSB**: 2.00
- **bcdDevice**: 1.00
- **Negotiated speed**: High Speed (480 Mbps)
- **bmAttributes**: `0x80` (bus-powered)
- **MaxPower**: 100 mA
- **bNumConfigurations**: 1

### Interface 0

| Field | Value |
|---|---|
| `bInterfaceClass` | `0x08` Mass Storage |
| `bInterfaceSubClass` | `0x06` SCSI transparent |
| `bInterfaceProtocol` | `0x50` Bulk-Only (BBB) |
| `bNumEndpoints` | 2 |

### Endpoints

| Address | Direction | Type | wMaxPacketSize |
|---|---|---|---|
| `0x81` | IN  | Bulk | 512 |
| `0x02` | OUT | Bulk | 512 |

Plain, spec-compliant USB Mass Storage Class / Bulk-Only Transport. No
vendor-specific interfaces, no extra alternate settings, no HID, no CDC.
This matches what the Linux kernel's `usb-storage` driver binds to
automatically.

## Filesystem

```
/dev/sda1  vfat  58.2G  label=PLAUD_NOTE
```

- **Filesystem**: FAT (VFAT)
- **Label**: `PLAUD_NOTE`
- **Capacity**: 58.2 GiB
- **In use**: 128 KiB (device is empty — FAT metadata only)
- **Mount point** (GNOME auto-mount): `/run/media/<user>/PLAUD_NOTE`

### Directory layout

```
/
├── MODEL.txt          (70 bytes, ASCII)
├── NOTES/             (recordings made with the PLAUD button)
└── CALLS/             (phone-call recordings)
```

This is the canonical layout documented (informally) by Plaud's own support
page for the "Access via USB" feature and now confirmed against our hardware.

### `MODEL.txt` contents (70 bytes, ASCII)

```
PLAUD NOTE V0095@00:47:14 Feb 28 2024
Serial No.:<SERIAL>
```

Format appears to be a fixed-schema metadata file, two lines:

```
<product-name> V<firmware-build>@<build-time> <build-date>
Serial No.:<serial>
```

**Usability note for our CLI**: `MODEL.txt` is the cheapest `DeviceInfo`
source we have. The USB transport can parse it without touching BLE, and
it gives us product, firmware version, and serial in one 70-byte read.

## Evidence references

- `lsusb` raw output: captured in `2026-04-05-plaud-note-v0095-baseline.lsusb.txt`
  (gitignored under `specs/re/captures/usb/unsanitised/` when personal data
  is present; not needed here since the only sensitive value is the serial).
- `MODEL.txt`: read via VFAT mount, redacted copy above.
- `lsblk`, `df`, and `ls` outputs: captured inline above.

## Implications for the roadmap

- **M3 (USB-MSC transport)** is trivially achievable: the kernel already
  exposes the device as a block device with a FAT filesystem. `plaud-transport-usb`
  becomes "find a VFAT volume labelled `PLAUD_NOTE`, walk `NOTES/` and
  `CALLS/`, read `MODEL.txt` for `DeviceInfo`."
- **Realtek SoC** (`0bda:*`) is a strong prior that the BLE and Wi-Fi stacks
  are part of a Realtek RTL87xx combo chip. That narrows where to look in
  the APK for BLE service UUIDs and will help sanity-check GATT dumps in M4.
- **MODEL.txt schema** should be the first entry in `docs/protocol/file-formats.md`.
