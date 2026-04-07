# Device Info & Control

## Device info

```bash
plaude device info
# Device:     PLAUD_NOTE
# Model:      Plaud Note
# Firmware:   0000
# Serial:     00000000
# Storage:    919512072 / 919514000 bytes used (0 recordings)
```

JSON output:

```bash
plaude device info --output json
```

> **Note:** Firmware version and serial number are only available via USB (`--backend usb`).
> Over BLE, placeholder values are shown.

## Privacy toggle {#privacy}

Enable or disable the device's privacy mode:

```bash
plaude device privacy on
# privacy on

plaude device privacy off
# privacy off
```

## Device name {#name}

Print the device's advertised name:

```bash
plaude device name
# PLAUD_NOTE
```
