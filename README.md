# plaude-cli

Offline command-line interface for the Plaud Note voice recorder.
Talks to Plaud hardware directly over BLE, USB, or Wi-Fi with zero
ongoing plaud.ai cloud contact and zero Plaud phone-app dependency
after a one-time auth bootstrap.

> **Pre-release — not for production use.** This repository is in
> active development. Protocol behaviour is live-tested on firmware
> V0095; other firmware revisions may require additional work. See
> [`specs/plaude-cli-v1/ROADMAP.md`](specs/plaude-cli-v1/ROADMAP.md)
> for the current implementation status.

## What it does

Once released, plaude-cli will offer:

- **Sync recordings off the device** into a local directory, with
  resumable, idempotent downloads.
- **Read battery, storage, and every device setting** exposed by the
  vendor protocol.
- **Control recording** (start / stop / pause / resume) from the
  command line.
- **Sanitised export** that strips the forensic device-serial watermark
  Plaud bakes into every WAV file.
- **Optional local transcription** via a user-supplied `whisper.cpp`
  binary — still fully offline.

No plaud.ai account is used at runtime. No phone app is needed for
day-to-day operation. The one-time auth bootstrap is documented in
`docs/usage/` once the corresponding milestones land.

## Build

Requirements:

- Rust 1.85+ (Rust 2024 edition).
- Linux with BlueZ ≥ 5.66 for BLE transport; USB fallback needs a
  VFAT-capable kernel.

```bash
make build        # release build of the plaude-cli binary
make test         # unit + integration + e2e tests
make lint         # clippy, rustfmt check, cargo audit
make install      # install to ~/.cargo/bin
```

The `Makefile` is the single source of truth for CI invocations.

## Project layout

```
.
├── Cargo.toml                     # Workspace root
├── crates/
│   ├── plaud-domain/              # Pure domain types (M1)
│   ├── plaud-transport/           # Transport / discovery / auth traits (M1)
│   ├── plaud-proto/               # Wire-format codec (M2)
│   ├── plaud-auth/                # Token storage + bootstrap (M4, M8)
│   ├── plaud-transport-ble/       # btleplug-based BLE transport (M5)
│   ├── plaud-transport-wifi/      # Wi-Fi Fast Transfer client (M13)
│   ├── plaud-transport-usb/       # USB-MSC fallback (M10)
│   ├── plaud-sim/                 # CI device simulator (M3)
│   ├── plaud-sync/                # High-level sync orchestrator (M9)
│   └── plaude-cli/                # The `plaude-cli` binary (M0, M4+)
├── docs/
│   ├── protocol/                  # Reverse-engineered wire spec
│   └── usage/                     # User-facing command docs
└── specs/
    ├── plaude-cli-v1/             # PLAN, ROADMAP, per-milestone journeys
    └── re/                        # Reverse-engineering evidence tree
```

## Contributing / working on plaude-cli

- Read [`AGENTS.md`](AGENTS.md) for the process contract (micro-TDD,
  evidence-backed protocol claims, no unwraps in production, zero
  clippy warnings as the merge gate).
- Read [`specs/plaude-cli-v1/ROADMAP.md`](specs/plaude-cli-v1/ROADMAP.md)
  for the current milestone and its DoR/DoD.
- Every protocol claim in `docs/protocol/` cites its evidence under
  `specs/re/captures/` or `specs/re/apk-notes/`.

## Privacy notice

The Plaud Note has several privacy characteristics you should be aware of:

1. **BLE traffic is cleartext** on V0095 firmware. Anyone with a BLE
   sniffer within ~10 m can record file data and the auth token during
   a sync. Do not sync in hostile physical environments.
2. **Every WAV file contains the device serial** (18 digits) embedded
   in a custom RIFF `pad ` chunk. Use `--sanitise` on `sync` to scrub
   the serial on copy.
3. **The auth token is a long-lived credential** stored in your OS
   keyring (or `~/.config/plaude/token` mode `0600`). Treat it like a
   password.

Run `plaude-cli --about` to see this disclosure at any time. See
`docs/protocol/overview.md#security-model` for the full security model
derived from our reverse-engineering work.

## License

Dual-licensed under either the MIT license or the Apache License 2.0
at your option.
