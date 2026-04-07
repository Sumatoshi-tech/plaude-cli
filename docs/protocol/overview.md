# Plaud Note Protocol Overview

This document is the top-level index of the reverse-engineered protocol spec
for the original Plaud Note device. Every claim in this directory is backed
by a citation into `specs/re/captures/` or `specs/re/apk-notes/`.

## Device

- **Product**: PLAUD NOTE (original, credit-card form factor)
- **Observed firmware**: V0095 (Feb 28 2024) — see
  [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`](../../specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md)
- **Architecture**: **dual-SoC** (evidence-backed, not speculation):
  - **Nordic nRF5x** for BLE — derived from BLE manufacturer ID `0x0059`
    (Nordic Semiconductor ASA) observed in advertisement data.
    See [`specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md`](../../specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md).
  - **Realtek RTL87xx** for Wi-Fi, USB-MSC, and storage — derived from
    USB `idVendor=0x0bda` (Realtek Semiconductor Corp.) and the device's
    USB Mass Storage interface. See the USB baseline capture.
- **Shared identity**: both SoCs surface the same device serial — it appears
  in USB `iSerial`, `/MODEL.txt`, and the custom `pad ` RIFF chunk of every
  `.WAV` file. There must be an internal bus (UART/SPI) between the two
  chips to synchronise commands and file transfers.

## Transports

| Transport | Status | Evidence | Use |
|---|---|---|---|
| **USB Mass Storage** | Confirmed on V0095 | [usb baseline](../../specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md) | Read-only bulk access to recordings and `MODEL.txt`. Requires one-time app toggle. |
| **BLE control** | Unknown | — | Control channel used by the phone app. To be mapped in R0. |
| **Wi-Fi Fast Transfer** | Unknown | — | Bulk transfer over a device-hosted open Wi-Fi AP. To be mapped in R2. |

## Capability matrix

| Capability | USB | BLE | Wi-Fi | Spec |
|---|---|---|---|---|
| `DeviceInfo` | ✅ via `MODEL.txt` | 🟡 via `GetDeviceName` opcode `0x006C` | ? | [file-formats.md](file-formats.md), [ble-commands.md](ble-commands.md) |
| `ListRecordings` | ✅ via VFAT walk | 🟡 metadata sweep via opcodes `0x08`/`0x16`/`0x1A` (conjectured) | ? | [file-formats.md](file-formats.md), [ble-commands.md](ble-commands.md) |
| `ReadRecording` | ✅ paired `.WAV` / `.ASR` | 🟡 `0x1C` + bulk magic `0x02` stream | ? | [file-formats.md](file-formats.md), [ble-commands.md](ble-commands.md) |
| `DeleteRecording` | 🟡 VFAT unlink (not yet verified) | ⬜ | ? | — |
| `BatteryLevel` | — | ✅ SIG service `0x180F` char `0x2A19` | — | [ble-gatt.md](ble-gatt.md) |
| `Authenticate` | — | 🟡 opcode `0x0001`, 32-hex token | — | [ble-commands.md](ble-commands.md) |
| `StartFastTransfer` | — | ⬜ (not seen in the first capture — phone used BLE-only transfer) | — | — |
| `StopRecording` | — | ⬜ | — | — |

Legend: ✅ confirmed, ? unknown, — not applicable on this transport.

## Security model

- **USB**: unauthenticated, read/write access to the raw filesystem when
  the "Access via USB" toggle is enabled in the Plaud phone app. No
  runtime authentication. **Deprecated by vendor** — an in-app warning
  string in `libapp.so` announces that "USB file access and RAW file
  features will be permanently disabled" in an upcoming firmware
  update. Treat USB as a transitional fallback, not a primary
  transport for new code.
- **BLE**: **no link-layer encryption, no BLE bonding**. The device
  enforces access via a single application-layer pre-shared token sent
  in opcode `0x0001` at session start. **Live-tested**: the token is
  per-device (not per-phone), static for the device's lifetime,
  replayable verbatim from any BLE central. Once captured, it works
  forever without rotation or refresh. After auth, all vendor traffic
  is cleartext. Standard SIG services (battery) are reachable without
  auth. See [ble-commands.md](ble-commands.md#security-model-evidence-backed-live-tested).
- **Wi-Fi Fast Transfer**: open hotspot, no password (per vendor docs
  and community reports). Triggered by a BLE command (still pending
  identification among opcodes `0x78`–`0x7D`). Protocol on the hotspot
  side is not yet analysed. Presumed cleartext.
- **Forensic identifiers in recordings**: every `.WAV` file contains
  the device serial inside a custom `pad ` RIFF subchunk (see
  [file-formats.md](file-formats.md)). Our CLI must offer a
  `--sanitise` export mode.
- **Privacy disclosure the README must carry**: because BLE traffic
  is cleartext, any BLE sniffer within ~10 m during a sync session
  can record file metadata, file contents, and the auth token itself.
  Users should understand this before using the CLI in untrusted
  physical environments.

## Sub-documents

- [ble-gatt.md](ble-gatt.md) — populated by `re-ble-recon`.
- [ble-commands.md](ble-commands.md) — populated by `re-spec` from btsnoop + APK evidence.
- [wifi-fast-transfer.md](wifi-fast-transfer.md) — populated by `re-wifi-probe` + `re-spec`.
- [file-formats.md](file-formats.md) — `MODEL.txt`, audio container, any on-disk metadata.
