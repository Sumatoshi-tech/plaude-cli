# Plaude — User Guide

Offline command-line tool for the [Plaud Note](https://www.plaud.ai/) voice recorder.
Connects over Bluetooth — no cloud, no phone app needed after initial setup.

## Quick Start

### 1. Install

```bash
make install    # installs `plaude` to ~/.cargo/bin
```

### 2. Set up your auth token

If you already have your token (32 hex characters):

```bash
plaude auth set-token <your-32-hex-token>
```

Or capture it automatically from the phone app (requires the Plaud app open on your phone):

```bash
plaude auth bootstrap
```

### 3. Use it

```bash
plaude battery                         # check battery (no token needed)
plaude device info                     # device name, storage stats
plaude files list                      # list recordings
plaude files pull-one <ID> -o ~/plaud  # download a recording
plaude record start                    # start recording remotely
plaude record stop                     # stop recording
```

## Commands

| Command | What it does | Auth required? |
|---|---|---|
| [`battery`](battery.md) | Show battery percentage | No |
| [`device info`](device-info.md) | Show device name, model, storage | Yes |
| [`device privacy on/off`](device-info.md#privacy) | Toggle privacy mode | Yes |
| [`device name`](device-info.md#name) | Show device name | Yes |
| [`files list`](files.md) | List recordings on device | Yes |
| [`files pull-one`](files.md#downloading) | Download a recording | Yes |
| [`record start/stop/pause/resume`](record.md) | Remote recording control | Yes |
| [`settings list/get/set`](settings.md) | Read/write device settings | Yes |
| [`sync`](sync.md) | Mirror all recordings to a directory | Yes |
| [`auth set-token/show/clear/import`](auth.md) | Manage stored token | No |
| [`auth bootstrap`](auth.md#bootstrap) | Capture token from phone app | No |
| [`transcribe`](transcribe.md) | Transcribe WAV via whisper.cpp | No |

## Backends

Plaude supports multiple ways to connect to your device:

| Flag | Transport | Use case |
|---|---|---|
| `--backend ble` (default) | Bluetooth Low Energy | Primary — wireless, all commands |
| `--backend usb --mount /path` | USB Mass Storage | Fallback — full WAV files, fast |
| `--backend sim` | In-process simulator | Testing and development |

## Exit Codes

| Code | Meaning | What to do |
|------|---------|------------|
| `0` | Success | — |
| `1` | Runtime error | Check the error message |
| `2` | Bad usage | Check `plaude --help` |
| `69` | Device unreachable | Is the device on? Is Bluetooth enabled? |
| `77` | No auth token | Run `plaude auth bootstrap` or `plaude auth set-token` |
| `78` | Token rejected | Run `plaude auth bootstrap` to get a fresh token |

## Privacy Notice

Run `plaude --about` to see the full disclosure. Key points:

1. **BLE traffic is unencrypted** — don't sync in hostile environments
2. **WAV files contain your device serial** — use `--sanitise` with sync
3. **The auth token is a permanent credential** — stored in your OS keyring
