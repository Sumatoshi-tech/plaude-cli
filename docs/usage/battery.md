# `plaude-cli battery`

Print the battery percentage of a connected Plaud device.

The battery service is the standard Bluetooth SIG Battery Service
(`0x180F`, characteristic `0x2A19`). Reading it **does not require an
auth token** — this is the same invariant observed on real hardware
during the Test 2b token-validation capture.

## Synopsis

```
plaude-cli [--backend <sim|ble>] battery [--output <text|json>]
```

## Options

| Flag | Default | Description |
|---|---|---|
| `--backend` | `ble` | Runtime backend. `sim` drives the in-process deterministic simulator (used by every CI test and by local dogfooding); `ble` is reserved for the future real-hardware backend. |
| `--output` | `text` | Output format. `text` is human-readable; `json` emits a single-line object. |

## Examples

Human-readable (default):

```console
$ plaude-cli --backend sim battery
Battery: 100%
```

JSON for scripts:

```console
$ plaude-cli --backend sim battery --output json
{"percent":100}
```

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success — battery was read. |
| `1` | Runtime error — the selected backend is not yet wired, or the device disconnected. |
| `2` | Usage error — invalid flag value. |

## Backend note

The `ble` backend currently returns
`TransportError::Unsupported { capability: "ble-hardware-backend" }`.
A later milestone replaces the stub with a real btleplug central that
performs a scan → connect → read over GATT. Until that ships, use
`--backend sim` for local runs.

## See also

- [`plaude-cli device info`](device-info.md) — full device identity
  and storage summary.
- [`plaude-cli auth`](../../README.md#plaude-cli-auth) — token storage
  (not required for `battery`).
