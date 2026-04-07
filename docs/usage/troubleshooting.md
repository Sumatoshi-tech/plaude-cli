# Troubleshooting

## Common errors and resolutions

### Exit 69 — "the `ble` backend is not yet wired"

The real BLE hardware backend is not yet implemented. Use one of:

- `--backend sim` for testing and development.
- `--backend usb --mount /path/to/PLAUD_NOTE` for the USB fallback.

### Exit 77 — "no auth token stored for this device"

A command that requires device authentication was run without a stored
token. Fix:

```bash
# Option 1: bootstrap from the phone app (sim path for now)
plaude-cli --backend sim auth bootstrap

# Option 2: manually enter a token
plaude-cli auth set-token <32-hex-chars>

# Option 3: import from a btsnoop capture
plaude-cli auth import /path/to/btsnoop_hci.log
```

### Exit 78 — "device rejected the stored token"

The device rejected the stored auth token. The token may be stale or
for a different device. Fix:

```bash
plaude-cli auth clear
plaude-cli auth bootstrap
```

### "protocol error: invalid recording-state transition"

A recording control command (`record start/stop/pause/resume`) was
issued in a state that doesn't allow that transition. For example:

- `record stop` when no recording is in progress.
- `record pause` when the device is idle.
- `record start` when already recording.

Check the device's current state before issuing the command.

### "not found: <setting-name>"

The setting exists in the CLI's enum but has no stored value on the
device. Not all settings are initialised on all firmware versions. Use
`settings list` to see which settings have values.

### "unknown setting name: <name>"

The setting name doesn't match any known `CommonSettingKey`. Run
`settings list` to see valid names, or check `docs/usage/settings.md`
for the full table.

## Enabling debug logs

Set the `RUST_LOG` environment variable:

```bash
# Human-readable text logs on stderr
RUST_LOG=debug plaude-cli --backend sim battery

# JSON-formatted logs for log aggregators
RUST_LOG=info plaude-cli --backend sim --log-format json battery
```

Log output goes to stderr; stdout remains clean for command output.

## Configuring timeouts

```bash
# Via CLI flag (seconds)
plaude-cli --timeout 10 --backend sim battery

# Via environment variable
export PLAUDE_TIMEOUT=10
plaude-cli --backend sim battery
```

Default timeout is 30 seconds.

## Filing a bug

If you encounter an unexpected error:

1. Reproduce with `RUST_LOG=debug` and capture stderr.
2. Check the exit code: `echo $?`
3. Note the firmware version: `plaude-cli device info` (if reachable).
4. File an issue at the project repository with the above information.
