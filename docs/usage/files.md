# `plaude-cli files`

Manage recordings on a connected Plaud device.

Two subcommands ship in M7:

- `plaude-cli files list` — enumerate every recording on the device.
- `plaude-cli files pull-one <id>` — download a single recording's
  paired `.WAV` + `.ASR` files to disk.

Both subcommands require an authenticated transport — store a token
first with `plaude-cli auth set-token` or `plaude-cli auth import`.

## `plaude-cli files list`

```
plaude-cli [--backend <sim|ble>] files list [--output <text|json>]
```

### Example

```console
$ plaude-cli --backend sim files list
ID           KIND  STARTED              WAV        ASR
1775393534   note  1775393534           18         18
```

```console
$ plaude-cli --backend sim files list --output json
[{"id":"1775393534","kind":"note","started_at_unix_seconds":1775393534,"wav_size":18,"asr_size":18}]
```

## `plaude-cli files pull-one`

```
plaude-cli [--backend <sim|ble>] files pull-one <ID> [-o <DIR>] [--resume]
```

### Arguments

| Argument | Description |
|---|---|
| `<ID>` | Recording id (the value printed in the `ID` column of `files list`). |
| `-o`, `--output-dir <DIR>` | Destination directory. Created if missing. Defaults to the current working directory. |
| `--resume` | If the target files already exist at the expected byte size, skip them (idempotent). Without this flag, pre-existing files are overwritten. |

### Output files

Two files land at `<DIR>/<ID>.wav` (stereo PCM) and `<DIR>/<ID>.asr`
(mono Opus sidecar). The base name matches the recording id
byte-for-byte.

### Example

```console
$ plaude-cli --backend sim files pull-one 1775393534 -o /tmp/dump
$ ls /tmp/dump
1775393534.asr  1775393534.wav
```

### Resume semantics

M7 implements idempotent-skip resume: if both `.wav` and `.asr`
already exist at the expected sizes, the command exits 0 with an
`already up to date` message without touching the files. If either
file is the wrong size (partial, truncated, corrupt), the command
rewrites it from scratch.

Mid-offset resume (start-at-byte-N) requires range-read support on
the transport layer and lands in a later milestone alongside the
full `plaude-cli sync` command.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Runtime error — unknown recording id, I/O failure, transport error. |
| `2` | Usage error — invalid recording id format. |
| `77` | `EX_NOPERM` — no auth token stored. See [`plaude-cli auth`](../../README.md#plaude-cli-auth). |
| `78` | `EX_CONFIG` — the device rejected the stored token. |

## See also

- [`plaude-cli battery`](battery.md)
- [`plaude-cli device info`](device-info.md)
