# plaude-cli usage guide

This directory is the user-facing reference for plaude-cli. One page
per top-level subcommand, plus a troubleshooting section and an
exit-code table.

## Current status (M0–M12)

The binary currently supports:

- `plaude-cli --help` / `-h` — top-level help
- `plaude-cli --version` / `-V` — installed version
- `plaude-cli` (no arguments) — prints help and exits 2
- `plaude-cli auth set-token <hex>` — store a 16- or 32-char ASCII-hex
  BLE auth token into the OS keyring + file fallback
- `plaude-cli auth import <btsnoop_hci.log>` — extract the auth token
  from an Android HCI snoop log via a pure-Rust parser (no `tshark`
  required) and store it
- `plaude-cli auth show` — print the SHA-256-truncated-to-16-char
  fingerprint of the stored token; **never** prints the raw value
- `plaude-cli auth clear` — remove the stored token from every backend
- `plaude-cli --config-dir <PATH>` — global override for the token
  file location. Useful for tests and for users who want to keep the
  token outside `~/.config`.

### Token storage model

Tokens are stored via a two-layer chain:

1. **Primary**: OS keyring (Linux Secret Service / macOS Keychain /
   Windows Credential Manager) under service name `plaude-cli`.
2. **Fallback**: `~/.config/plaude/token` on Linux (platform-equivalent
   path elsewhere), mode `0600`, parent directory mode `0700`.

`auth set-token` and `auth import` write to the keyring if available;
otherwise to the file. `auth show` reads the primary first and falls
back to the file. `auth clear` removes from both.

On headless systems with no secret-service daemon, the file fallback
is always used. No functionality is lost.

### Privacy guarantees of `auth show`

The `show` subcommand computes `sha256(token)[:16]` and prints only
that fingerprint plus a "backend: file|keyring" hint. The raw token
never reaches stdout or stderr. The fingerprint is stable across
invocations and deterministic, so users can match it against
evidence documents without needing to log the original token.

## Planned subcommands

The pages below will be filled in as their corresponding milestones
land. Each page is written during the milestone and includes example
input, expected output, exit codes, and troubleshooting.

| Command                         | Milestone | Status |
|---------------------------------|-----------|--------|
| `plaude auth set-token`         | M4        | ✅ |
| `plaude auth import`            | M4        | ✅ |
| `plaude auth show`              | M4        | ✅ |
| `plaude auth clear`             | M4        | ✅ |
| `plaude-cli auth bootstrap`     | M8        | ✅ sim path (see [auth-bootstrap.md](auth-bootstrap.md)); real BlueZ backend deferred |
| `plaude devices scan`           | M5        | not yet |
| `plaude-cli battery`            | M6        | ✅ (see [battery.md](battery.md)) |
| `plaude-cli device info`        | M6        | ✅ (see [device-info.md](device-info.md)) |
| `plaude-cli files list`         | M7        | ✅ (see [files.md](files.md)) |
| `plaude-cli files pull-one`     | M7        | ✅ (see [files.md](files.md)) |
| `plaude-cli sync <dir>`         | M9        | ✅ (see [sync.md](sync.md)) |
| `plaude-cli --backend usb`      | M10       | ✅ (see [usb.md](usb.md)) — USB MSC fallback with `--mount`, `--sanitise`, MODEL.txt parser, `WavSanitiser` |
| `plaude-cli settings list\|get\|set` | M11  | ✅ (see [settings.md](settings.md)) |
| `plaude-cli record start\|stop\|pause\|resume` | M11 | ✅ (see [record.md](record.md)) |
| `plaude-cli device privacy on\|off` | M11  | ✅ |
| `plaude-cli device name`        | M11       | ✅ |
| `plaude-cli --about`            | M12       | ✅ privacy disclosure |
| `plaude-cli --log-format json`  | M12       | ✅ structured logging (see [troubleshooting.md](troubleshooting.md)) |
| `plaude-cli --timeout <SECS>`   | M12       | ✅ configurable timeouts |
| Exit codes                      | M12       | ✅ (see [exit-codes.md](exit-codes.md)) |
| Troubleshooting                 | M12       | ✅ (see [troubleshooting.md](troubleshooting.md)) |
| `plaude-cli transcribe`         | M15       | ✅ (see [transcribe.md](transcribe.md)) — offline whisper.cpp wrapper |

## Exit codes

plaude-cli aims at CLIG / `sysexits(3)` compliance. A full exit-code
table lands in M12. The stable codes used today are:

| Code | Name            | Meaning                                                              |
|------|-----------------|----------------------------------------------------------------------|
| `0`  | success         | The command completed normally.                                     |
| `1`  | runtime error   | An I/O, parse, or transport failure.                                 |
| `2`  | usage error     | The CLI was invoked without required arguments, or with invalid ones. |
| `77` | `EX_NOPERM`     | A vendor command ran without an auth token. (M6) |
| `78` | `EX_CONFIG`     | The device rejected the stored token. (M6) |

## Privacy and security

plaude-cli surfaces the following facts about the Plaud Note so you
can make an informed decision about using it:

1. **Every `.WAV` file on a Plaud Note contains the device serial**
   embedded in a custom RIFF `pad ` chunk. plaude-cli's future
   `--sanitise` export mode scrubs this on copy.
2. **BLE traffic is cleartext on V0095 firmware.** Anyone with a BLE
   sniffer within ~10 m can record file bytes and the auth token
   itself during a sync. Do not sync in hostile physical environments.
3. **The BLE auth token is a long-lived per-device credential.** It is
   stored in your OS keyring (or `~/.config/plaude/token` mode `0600`)
   after first-run bootstrap. Treat it like a password.

See [`../protocol/overview.md`](../protocol/overview.md) for the full
security model derived from our reverse-engineering work.
