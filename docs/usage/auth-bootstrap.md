# `plaude-cli auth bootstrap`

One-time onboarding command that captures your Plaud device's auth
token from your phone app, with **no** `adb`, no `tshark`, and no
reliance on plaud.ai cloud services.

## How it works

1. Run `plaude-cli auth bootstrap` on your laptop.
2. The CLI spins up a local BLE peripheral advertising `PLAUD_NOTE`
   with the Nordic manufacturer id `0x0059`.
3. Open the Plaud phone app with your real pen asleep or out of
   range. The app scans for `PLAUD_NOTE`, finds your laptop, and
   connects to it thinking it's the pen.
4. The app writes a standard V0095 auth frame
   (`01 01 00 02 00 00 <token>`) to the vendor write characteristic.
5. The CLI decodes the write, stores the token via the same
   `AuthStore` chain the `auth set-token` subcommand uses, and exits
   with the token's fingerprint.

You now have the token in your keyring (or `~/.config/plaude/token`
fallback) and can run every other `plaude-cli` command against the
real device.

## Synopsis

```
plaude-cli [--backend <sim|ble>] auth bootstrap [--timeout <SECS>]
```

## Options

| Flag | Default | Description |
|---|---|---|
| `--timeout <SECS>` | `120` | Budget in seconds to wait for the phone's auth write. |
| `--backend sim\|ble` | `ble` | Runtime backend. |

## Backend note

The `ble` backend currently returns a runtime error with the stable
marker `ble-hardware-backend`. It lands alongside the btleplug-backed
central in a later milestone — the two share BlueZ D-Bus plumbing so
we wire them in one go rather than twice.

The `sim` backend runs the full capture-and-store pipeline against a
hermetic in-process fake phone that writes a deterministic token. It
is how the CI suite validates the command surface and is a useful
dogfooding target while real hardware support is pending:

```console
$ plaude-cli --backend sim auth bootstrap
Token captured. Fingerprint: a4f5...1234
$ plaude-cli auth show
Token stored.
Fingerprint: a4f5...1234
```

## Preconditions

For the real hardware flow (once it ships):

1. Put your real Plaud Note to sleep or take it out of Bluetooth
   range, so the phone app is not already connected to it.
2. Your laptop's Bluetooth adapter supports LE peripheral mode
   (every modern Linux box with BlueZ ≥ 5.56 does).
3. You have the `bluetooth` group on your user, or you can run the
   command under `sudo` the one time this onboarding runs.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Token captured and stored. |
| `1` | Runtime error — timeout, decode failure, or `ble` backend not yet wired. |
| `2` | Usage error — invalid flag value. |

## Privacy note

The captured token is a 16- or 32-character ASCII hex string that
authenticates **your device** to its own cloud. plaude-cli stores
it locally and never transmits it anywhere. `auth show` only ever
prints a SHA-256 fingerprint, not the raw value.

## See also

- [`plaude-cli auth set-token`](../../README.md#plaude-cli-auth) —
  manually paste a token you already have.
- [`plaude-cli auth import`](../../README.md#plaude-cli-auth) —
  extract a token from an Android HCI snoop log.
- [`plaude-cli battery`](battery.md)
- [`plaude-cli device info`](device-info.md)
