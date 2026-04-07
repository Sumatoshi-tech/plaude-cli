# Device Settings

Read and write the device's configuration over Bluetooth.

## List all settings

```bash
plaude settings list
# enable-vad = 1
# mic-gain = 20
# auto-power-off = 0
```

JSON output:

```bash
plaude settings list --output json
```

## Read a single setting

```bash
plaude settings get mic-gain
# mic-gain = 20
```

## Write a setting

```bash
plaude settings set enable-vad false
# enable-vad = false
```

Values are parsed as: boolean (`true`/`false`), then u8 (0–255), then u32.

## Available settings

| Name | Description |
|---|---|
| `back-light-time` | Screen backlight duration (seconds) |
| `back-light-brightness` | Screen brightness |
| `language` | UI language code |
| `auto-delete-record-file` | Auto-delete old recordings when full |
| `enable-vad` | Voice activity detection on/off |
| `rec-scene` | Recording scene profile |
| `rec-mode` | Recording mode |
| `vad-sensitivity` | VAD sensitivity level |
| `vpu-gain` | Voice processing unit gain |
| `mic-gain` | Microphone pre-amp gain |
| `wifi-channel` | Wi-Fi channel for Fast Transfer |
| `switch-handler-id` | Physical mode switch handler |
| `auto-power-off` | Auto-power-off timeout |
| `save-raw-file` | Keep raw WAV alongside Opus |
| `auto-record` | Auto-start recording |
| `auto-sync` | Auto-upload over Wi-Fi when idle |
| `find-my` | Find My feature |
| `vpu-clk` | Voice processing clock rate |
| `auto-stop-record` | Auto-stop after interval |
| `battery-mode` | Battery power profile |
