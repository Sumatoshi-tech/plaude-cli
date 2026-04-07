# `plaude-cli device info`

Print a formatted summary of a connected Plaud device: local name,
model, firmware version, serial, and storage stats.

Unlike `battery`, this command **requires an authenticated transport**,
which means a token must be in the store before running it. Use
`plaude-cli auth set-token <hex>` or `plaude-cli auth import <btsnoop>`
to seed the token first.

## Synopsis

```
plaude-cli [--backend <sim|ble>] device info [--output <text|json>]
```

## Options

| Flag | Default | Description |
|---|---|---|
| `--backend` | `ble` | Runtime backend. `sim` drives the in-process simulator; `ble` is reserved for the future real-hardware backend. |
| `--output` | `text` | Output format: `text` or `json`. |

## Examples

Human-readable:

```console
$ plaude-cli --backend sim device info
Device:     PLAUD_NOTE
Model:      Plaud Note
Firmware:   0000
Serial:     00000000
Storage:    0 / 0 bytes used (0 recordings)
```

JSON for scripts:

```console
$ plaude-cli --backend sim device info --output json
{"local_name":"PLAUD_NOTE","model":"Plaud Note","firmware":"0000","serial":"00000000","storage":{"total_bytes":0,"used_bytes":0,"free_bytes":0,"recording_count":0}}
```

## Exit codes

| Code | Name | Meaning |
|---|---|---|
| `0` | success | Device info was read. |
| `1` | runtime | Runtime error — the selected backend is not yet wired, or an I/O failure. |
| `2` | usage | Usage error — invalid flag value. |
| `77` | `EX_NOPERM` | No auth token stored. Run `plaude-cli auth --help` to fix. |
| `78` | `EX_CONFIG` | The device rejected the stored token. Run `plaude-cli auth bootstrap` or re-import a fresh token. |

The 77 / 78 split lets wrapper scripts distinguish "the user has never
set up this device" from "the stored token is stale and needs to be
re-captured", without having to grep stderr.

## Privacy note

`device info` prints the device **serial** verbatim in both text and
JSON outputs, because the user running the CLI is presumed to own the
device. The stored auth **token** is never printed — only a SHA-256
fingerprint is ever emitted by the `auth` subcommand tree.

## See also

- [`plaude-cli battery`](battery.md)
- [`plaude-cli auth`](../../README.md#plaude-cli-auth)
