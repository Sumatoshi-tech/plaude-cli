# `plaude-cli settings` — read and write device settings

## Overview

The `settings` subcommand tree provides read/write access to the
device's CommonSettings register, which controls recording behaviour,
display preferences, power management, and network features.

All settings commands require an authenticated transport (a stored
token). If no token is stored, the command exits with code 77
(`EX_NOPERM`). If the device rejects the token, exit code 78
(`EX_CONFIG`).

## Commands

### `settings list`

Print every setting that has a stored value on the device.

```
$ plaude-cli --backend sim settings list
enable-vad = true
mic-gain = 20
auto-power-off = 300
```

With `--output json`:

```
$ plaude-cli --backend sim settings list --output json
[{"name":"enable-vad","value":"true"},{"name":"mic-gain","value":"20"},{"name":"auto-power-off","value":"300"}]
```

### `settings get <name>`

Read a single setting by its CLI name.

```
$ plaude-cli --backend sim settings get enable-vad
enable-vad = true
```

Unknown setting names exit with code 2 (usage error).

### `settings set <name> <value>`

Write a single setting. The value is parsed as a boolean (`true`/`false`),
`u8` (0--255), or `u32` (256--4294967295), in that priority order.

```
$ plaude-cli --backend sim settings set enable-vad false
enable-vad = false
```

## Available settings

| CLI name | Code | Description |
|---|---|---|
| `back-light-time` | 1 | Screen backlight-on duration (seconds) |
| `back-light-brightness` | 2 | Screen backlight brightness |
| `language` | 3 | UI language code |
| `auto-delete-record-file` | 4 | Auto-delete old recordings on full flash |
| `enable-vad` | 15 | Voice activity detection master switch |
| `rec-scene` | 16 | Recording scene profile |
| `rec-mode` | 17 | Recording mode variant |
| `vad-sensitivity` | 18 | VAD sensitivity level |
| `vpu-gain` | 19 | Voice processing unit gain |
| `mic-gain` | 20 | Microphone pre-amp gain |
| `wifi-channel` | 21 | Wi-Fi channel for Fast Transfer |
| `switch-handler-id` | 22 | Physical mode switch handler |
| `auto-power-off` | 23 | Auto-power-off timeout |
| `save-raw-file` | 24 | Retain raw WAV alongside Opus sidecar |
| `auto-record` | 25 | Auto-start recording |
| `auto-sync` | 26 | Upload recordings over Wi-Fi when idle |
| `find-my` | 27 | Find My feature toggle |
| `vpu-clk` | 30 | Voice processing unit clock rate |
| `auto-stop-record` | 31 | Auto-stop recording after interval |
| `battery-mode` | 32 | Battery power profile |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Runtime error (setting not found on device, transport failure) |
| 2 | Usage error (unknown setting name, bad value) |
| 77 | No auth token stored |
| 78 | Device rejected the stored token |
