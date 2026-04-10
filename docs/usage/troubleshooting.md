# Troubleshooting

## "no PLAUD_NOTE device found"

- Is the device turned on?
- Is it within Bluetooth range (~10 meters)?
- Wait 10–15 seconds after turning it on before trying
- The device goes to sleep after inactivity — press the button to wake it

## "device rejected the stored token" (exit 78)

Your token is stale or for a different device.

```bash
plaude auth clear
plaude auth bootstrap    # re-capture from phone app
```

## "no auth token stored" (exit 77)

You haven't set up authentication yet.

```bash
plaude auth bootstrap    # automatic capture
# or
plaude auth set-token <32-hex-chars>
```

## "operation timed out" (exit 69)

The device didn't respond in time. Common causes:
- Device went to sleep during the operation
- Too many rapid reconnections — wait 15 seconds between commands
- BLE adapter issue — try `bluetoothctl power off && bluetoothctl power on`

## "protocol error: control response opcode mismatch"

Stale notifications from a previous connection. Wait 15 seconds and try again.

## Files list shows no recordings

The BLE file list only shows **unsynced** recordings. If the phone app already synced them, they won't appear. Options:
- Make a new recording on the device, then list again
- Use USB for a complete listing: `plaude --backend usb --mount /path files list`

## Download is very slow

BLE transfers at ~500 bytes/second. A 25-second recording takes about 3 minutes. This is a Bluetooth limitation. For faster transfers:
- Use USB: `plaude --backend usb --mount /path files pull-one <ID>`
- Keep recordings short when using BLE

## LLM: "Status: unreachable"

Ollama is not running or the model is not downloaded.

```bash
ollama serve                    # start Ollama
ollama pull llama3.2:3b         # download default model
plaude llm check                # verify connectivity
```

For cloud providers, set the API key and model in `~/.config/plaude/llm.toml`. See [LLM Integration](llm.md).

## Summarize: "no transcript found"

You need to transcribe the recording first:

```bash
plaude transcribe --quality high recording.wav > recording.txt
plaude summarize recording.txt
```

## Enabling debug logs

```bash
RUST_LOG=debug plaude battery
RUST_LOG=plaud_transport_ble=debug plaude files list
```

Logs go to stderr; command output stays on stdout.

## JSON logs for automation

```bash
plaude --log-format json battery 2>log.json
```
