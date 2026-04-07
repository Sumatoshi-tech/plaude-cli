# plaude-cli v1 — Roadmap

Master implementation checklist. Each milestone (`Mxx`) is scoped to
deliver **testable, shippable value on its own**. Dependencies are
documented explicitly; a milestone may not start until its DoR is met
and it may not merge until every DoD box is ticked.

## Journey docs are written just-in-time

Each milestone gets a detailed CJM-style journey document under
[`journeys/`](journeys/) **only when work on that milestone is about
to start**. We deliberately do not author journey docs for future
milestones upfront because:

- Earlier milestones will teach us things that change the design of
  later ones (type ergonomics, error shapes, async-trait choices,
  crate dep decisions).
- Stale journey docs accumulate "dead" detail that disagrees with
  the landed code — a maintenance liability, not an asset.
- Writing a journey doc at start-of-milestone is itself part of the
  TDD planning phase and forces a fresh read of the latest evidence.

Currently authored journey docs:

- [x] [`journeys/M00-scaffold.md`](journeys/M00-scaffold.md) — **closed**
- [x] [`journeys/M01-domain-traits.md`](journeys/M01-domain-traits.md) — **closed**
- [x] [`journeys/M02-proto-codec.md`](journeys/M02-proto-codec.md) — **closed**
- [x] [`journeys/M03-sim-v0.md`](journeys/M03-sim-v0.md) — **closed**
- [x] [`journeys/M11-settings-record-control.md`](journeys/M11-settings-record-control.md) — **closed**
- [x] [`journeys/M12-hardening.md`](journeys/M12-hardening.md) — **closed**
- [x] [`journeys/M15-whisper-transcribe.md`](journeys/M15-whisper-transcribe.md) — **closed**
- [ ] M13, M14, M16 — authored at start of each milestone

The per-milestone **DoR / DoD / dependencies / high-level scope** live
in the milestone sections of *this* file and are stable from day one.
The *implementation detail* (exact types, test file names, friction
notes, UX nuance) belongs in the journey docs and is deferred.

## Current state (as of this writing)

The research phase is **complete**. The codebase contains no Rust code
yet; every milestone below begins from an empty Cargo workspace. What
exists is the evidence and specification tree:

| Asset | Status | Location |
|---|---|---|
| Plan | ✅ Complete | [`PLAN.md`](PLAN.md) |
| Protocol spec (BLE) | ✅ Complete | `docs/protocol/ble-gatt.md`, `docs/protocol/ble-commands.md` |
| Protocol spec (file formats) | ✅ Complete | `docs/protocol/file-formats.md` |
| Protocol spec (overview + security) | ✅ Complete | `docs/protocol/overview.md` |
| Protocol spec (Wi-Fi Fast Transfer) | 🟡 Stub | `docs/protocol/wifi-fast-transfer.md` |
| R0 BLE passive recon | ✅ | `specs/re/captures/ble-gatt/2026-04-05-hci0-plaud-note-r0-partial.md` |
| R2 dynamic capture (resumption) | ✅ | `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` |
| R2 dynamic capture (0day re-pair) | ✅ | `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md` |
| R1 APK analysis | ✅ | `specs/re/apk-notes/3.14.0-620/{architecture,ble-protocol,auth-token}.md` |
| R2.5 live BLE token validation | ✅ | `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md` |
| USB MSC baseline | ✅ | `specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md` |
| USB first-recording capture | ✅ | `specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md` |
| `Cargo.toml` / workspace | ✅ M0 | [`/Cargo.toml`](../../Cargo.toml) — 10 member crates, resolver 3, edition 2024 |
| `plaude-cli` binary (M0 skeleton) | ✅ M0 | [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs) — `--help`, `--version`, CLIG no-args exit 2 |
| CI workflow | ✅ M0 | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — lint/test/build on `ubuntu-latest` |
| Top-level README + `docs/usage/` | ✅ M0 | [`/README.md`](../../README.md), [`docs/usage/index.md`](../../docs/usage/index.md) |
| `plaud-domain` crate body (14 public types) | ✅ M1 | [`crates/plaud-domain/src/`](../../crates/plaud-domain/src/) — 7 modules, `thiserror` + `zeroize` only |
| `plaud-transport` crate body (3 traits + error enum) | ✅ M1 | [`crates/plaud-transport/src/`](../../crates/plaud-transport/src/) — dyn-compatible via `async-trait` |
| `plaud-proto` crate body (codec + encoders) | ✅ M2 | [`crates/plaud-proto/src/`](../../crates/plaud-proto/src/) — Frame demux, auth + file + device encoders, `Bytes`-based zero-copy decode |
| `plaud-sim` crate body (in-process fake Transport + Discovery) | ✅ M3 | [`crates/plaud-sim/src/`](../../crates/plaud-sim/src/) — `SimDevice` + `SimTransport` + `SimDiscovery` + `bulk::frames_for`, deterministic, failure-injectable |
| `plaud-auth` crate body (storage + btsnoop parser) | ✅ M4 | [`crates/plaud-auth/src/`](../../crates/plaud-auth/src/) — `FileStore` / `KeyringStore` / `ChainStore`, SHA-256 fingerprint, pure-Rust btsnoop parser |
| `plaude-cli auth` subcommand tree | ✅ M4 | [`crates/plaude-cli/src/commands/auth.rs`](../../crates/plaude-cli/src/commands/auth.rs) — `set-token`, `import`, `show`, `clear` + `--config-dir` sandbox override |
| `plaud-transport-ble` crate body (session + transport + discovery) | ✅ M5 | [`crates/plaud-transport-ble/src/`](../../crates/plaud-transport-ble/src/) — `BleSession` (auth / control / bulk), `BulkReassembler`, `BleTransport` with real `battery()` + `Error::Unsupported` stubs pointing at M6/M7/M11, hermetic `BleChannel::loopback_pair` test harness |
| `plaude-cli battery` + `plaude-cli device info` | ✅ M6 | [`crates/plaude-cli/src/commands/`](../../crates/plaude-cli/src/commands/) — `battery`, `device info`, global `--backend sim\|ble` flag, `TransportProvider` abstraction, `EX_NOPERM (77)` / `EX_CONFIG (78)` exit-code mapping; docs at [`docs/usage/battery.md`](../../docs/usage/battery.md), [`docs/usage/device-info.md`](../../docs/usage/device-info.md) |
| `plaude-cli files list` + `plaude-cli files pull-one` | ✅ M7 | [`crates/plaude-cli/src/commands/files.rs`](../../crates/plaude-cli/src/commands/files.rs) — enumerate + pull `<id>.wav` + `<id>.asr` with `indicatif` progress, idempotent-skip `--resume`, `-o DIR`. `Transport::read_recording_asr` added to the trait; real on sim, stubbed on `BleTransport`. Docs at [`docs/usage/files.md`](../../docs/usage/files.md). |
| `plaude-cli auth bootstrap` (fake peripheral, sim path) | ✅ M8 | [`crates/plaud-transport-ble/src/bootstrap/`](../../crates/plaud-transport-ble/src/bootstrap/) — `BootstrapSession` + `LoopbackBootstrap` + `TestPhone` hermetic stack, plus `plaud_proto::parse_auth_write` inverse decoder. CLI dispatches `--backend sim` through a deterministic fake phone that writes a 32-hex token and verifies end-to-end capture + store. `--backend ble` returns the stable `ble-hardware-backend` marker until the real BlueZ wiring ships. Docs at [`docs/usage/auth-bootstrap.md`](../../docs/usage/auth-bootstrap.md). |
| `plaude-cli sync <dir>` | ✅ M9 | [`crates/plaude-cli/src/commands/sync/`](../../crates/plaude-cli/src/commands/sync/) — idempotent mirror with a versioned JSON state file (`.plaude-sync.json`), SHA-256 inventory hash, incremental + dry-run + deleted-on-device handling. `PLAUDE_SIM_RECORDINGS` env hook in the sim backend lets e2e tests vary the device fixture between runs. Docs at [`docs/usage/sync.md`](../../docs/usage/sync.md). |
| `plaud-transport-usb` + `--backend usb` + `--sanitise` | ✅ M10 | [`crates/plaud-transport-usb/`](../../crates/plaud-transport-usb/) — `UsbTransport` (MODEL.txt parser, recording walker, WAV+ASR readers), `WavSanitiser` (SN-region scrubber), `Backend::Usb` + `--mount <PATH>` + `--sanitise` on sync, deprecation notice. Docs at [`docs/usage/usb.md`](../../docs/usage/usb.md). |
| `plaude-cli settings` + `record` + `device privacy/name` | ✅ M11 | [`crates/plaude-cli/src/commands/settings.rs`](../../crates/plaude-cli/src/commands/settings.rs), [`record.rs`](../../crates/plaude-cli/src/commands/record.rs), [`device.rs`](../../crates/plaude-cli/src/commands/device.rs) (extended). `CommonSettingKey::from_name` + `SettingValue::parse` in domain. Docs at [`docs/usage/settings.md`](../../docs/usage/settings.md), [`docs/usage/record.md`](../../docs/usage/record.md). |
| Hardening: exit codes, logging, timeouts, about, man pages, security review | ✅ M12 | Exit 69 (`EX_UNAVAILABLE`), `--log-format json`, `--timeout`, `--about`, `make man`, `docs/usage/exit-codes.md`, `docs/usage/troubleshooting.md` |
| `plaude-cli transcribe` (whisper.cpp wrapper) | ✅ M15 | [`crates/plaude-cli/src/commands/transcribe.rs`](../../crates/plaude-cli/src/commands/transcribe.rs) — thin subprocess wrapper, `--whisper-bin`/`--model`/`--language`/`--output-format`. Docs at [`docs/usage/transcribe.md`](../../docs/usage/transcribe.md). |
| Library crate bodies (transport-usb, sync) | ⬜ M10, M9 | Stubs exist with module docs only |

Existing non-code scaffolding at the repo root: `Makefile` (expects a
`plaude-cli` bin target), `rustfmt.toml` (edition 2024, width 140),
`clippy.toml` (complexity ≤ 15, args ≤ 7). `AGENTS.md` is the process
contract. `.gitignore` already covers the evidence tree.

## Milestone overview

Progression rules:

- **Every milestone delivers a user-visible capability or a testable
  invariant**, even M0.
- **Dependencies are strict**: `Mi` may not start until every DoD of
  every milestone it lists as a dependency has been closed.
- **Every milestone ends with `make lint` clean, `make test` green,
  zero `clippy` warnings, zero dead code, and every public item
  documented**. These are implicit DoDs and not repeated per milestone.
- **Every milestone has at least one e2e test** that exercises the
  shipped capability via the real `plaude` binary, against either a
  hermetic fixture or `plaud-sim`. Real-hardware tests are additional,
  not primary.

| #   | Milestone                                      | Ships                                                                                 | Depends on     | Journey doc                                                                  |
| --- | ---------------------------------------------- | ------------------------------------------------------------------------------------- | -------------- | ---------------------------------------------------------------------------- |
| M0  | Workspace scaffold & CI baseline               | `plaude --help` runs; `make lint/test/build` green                                    | —              | [`journeys/M00-scaffold.md`](journeys/M00-scaffold.md)                       |
| M1  | Domain + transport traits                      | `plaud-domain`, `plaud-transport`; type-only surface compilable                       | M0             | [`journeys/M01-domain-traits.md`](journeys/M01-domain-traits.md)             |
| M2  | `plaud-proto` codec                            | Encode/decode all 45 control opcodes + bulk frames; fixtures round-trip               | M1             | [`journeys/M02-proto-codec.md`](journeys/M02-proto-codec.md)                 |
| M3  | `plaud-sim` v0 (in-process)                    | Deterministic fake Transport; soft-reject auth; CCCD state; opcode echo               | M2             | [`journeys/M03-sim-v0.md`](journeys/M03-sim-v0.md)                           |
| M4  | `plaud-auth` — storage, import, show, clear    | `plaude auth import/set-token/show/clear`; OS keyring + file fallback                 | M1             | [`journeys/M04-auth-storage.md`](journeys/M04-auth-storage.md)               |
| M5  | `plaud-transport-ble` (btleplug)               | Connect, CCCD, auth frame, status byte, notification demux, soft-reject               | M2, M3, M4     | [`journeys/M05-transport-ble.md`](journeys/M05-transport-ble.md)             |
| M6  | `plaude battery` + `plaude device info`        | Standard SIG battery (no auth); `GetDeviceName`, `GetStorage`, `GetState` opcodes     | M5             | [`journeys/M06-battery-device-info.md`](journeys/M06-battery-device-info.md) |
| M7  | `plaude files list` + `plaude files pull-one`  | Metadata sweep via `0x08`; `0x1C` ReadFileChunk + bulk `0x02` stream assembly         | M5, M6         | [`journeys/M07-files-list-pull.md`](journeys/M07-files-list-pull.md)         |
| M8  | `plaude auth bootstrap` (fake peripheral)      | One-time token capture flow impersonating `PLAUD_NOTE`                                | M4, M5         | [`journeys/M08-auth-bootstrap-peripheral.md`](journeys/M08-auth-bootstrap-peripheral.md) |
| M9  | `plaude sync <dir>`                            | Resumable, idempotent mirror with state file and progress UI                          | M7             | [`journeys/M09-sync.md`](journeys/M09-sync.md)                               |
| M10 | `plaud-transport-usb` fallback                 | USB-MSC transport + `--sanitise` export                                               | M1, M7 optional | [`journeys/M10-transport-usb.md`](journeys/M10-transport-usb.md)            |
| M11 | Settings + recording remote control ✅          | `plaude settings`, `plaude record start/stop/pause/resume`, `plaude device privacy`   | M5, M6         | [`journeys/M11-settings-record-control.md`](journeys/M11-settings-record-control.md) |
| M12 | Hardening ✅                                    | Exit-code audit, structured logging, configurable timeouts, man pages, troubleshooting | M5–M11         | [`journeys/M12-hardening.md`](journeys/M12-hardening.md)                     |
| M13 | (stretch) Wi-Fi Fast Transfer                  | `plaude sync` auto-uses hotspot for large files                                       | M11, M12, R3   | [`journeys/stretch.md#m13-wifi-fast-transfer`](journeys/stretch.md)          |
| M14 | (stretch) Self-hosted HTTP sink                | Device auto-uploads to user's laptop via `SyncInIdle`                                 | M13            | [`journeys/stretch.md#m14-self-hosted-http-sink`](journeys/stretch.md)       |
| M15 | (stretch) `plaude transcribe` via whisper.cpp ✅ | Offline local transcription                                                           | M7 or M10      | [`journeys/M15-whisper-transcribe.md`](journeys/M15-whisper-transcribe.md)   |
| M16 | (stretch) RSA + ChaCha20-Poly1305 handshake    | Compat with newer firmware that rejects the plaintext auth token                      | M5, M12        | [`journeys/stretch.md#m16-rsa-chacha20-handshake`](journeys/stretch.md)      |

## Dependency graph

```
M0 ──► M1 ──► M2 ──► M3 ──► M5 ──► M6 ──► M7 ──► M9 ──► M12 ──► (release v1.0.0)
       │                     ▲                   │
       └──► M4 ──────────────┘                   └──► M11 ──► M12
                                                  
       M1 ──► M10 ──────────────────────────────► M12
       
       M4, M5 ──► M8 ──────────────────────────► M12
       
       (stretch) M12 ──► M13 ──► M14
                         │
                         └────► M15
                         │
                         └────► M16
```

M0–M12 is the **core scope for v1.0.0**. M13–M16 are additive. Shipping
M12 produces a complete, stand-alone offline CLI with full BLE-based
recording management, one-time auth bootstrap, and USB fallback.

---

## Cross-cutting concerns (apply to every milestone)

These are implicit DoDs. Violating any of them means the milestone is
not done, regardless of what the per-milestone checklist says.

### Code quality

- [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean
- [ ] `cargo fmt --all --check` clean
- [ ] No `TODO`, `FIXME`, `unimplemented!()`, or `todo!()` left in
      the codebase
- [ ] No `.unwrap()` / `.expect()` in production code paths; only in
      tests and in a documented `panic = "irrecoverable"` contract
- [ ] Every public item has a `///` doc comment
- [ ] Cyclomatic / cognitive complexity ≤ 15 per function (enforced
      by `clippy.toml`)
- [ ] Functions with more than 7 parameters trigger a clippy error
      (enforced by `clippy.toml`); use struct parameters
- [ ] No dead code; delete anything unused

### Tests

- [ ] Unit tests for every pure function with non-trivial branching
- [ ] Integration tests for every public crate API
- [ ] At least one **e2e test that exercises the real `plaude` binary**
      for every user-facing capability shipped in the milestone
- [ ] e2e tests are **hermetic** — no network, no real hardware, no
      user-specific keyring state
- [ ] Tests are deterministic (seeded RNG, controlled clocks,
      no sleeps longer than 50 ms)
- [ ] `make test` runs clean in under 60 seconds for the full suite
      (excluding optional hardware-gated tests)
- [ ] Hardware-gated tests are marked with `#[cfg(feature = "hw-tests")]`
      and excluded from CI

### Security & privacy

- [ ] Auth token is never logged, never printed, never written to a
      committable file
- [ ] Device serial is never logged, never printed, never written to a
      committable file beyond the redacted form `<SERIAL>` in evidence
      documents
- [ ] Any new evidence capture added to `specs/re/captures/` is either
      gitignored (raw logs) or explicitly sanitised (markdown walkthroughs)
- [ ] Error messages do not leak credentials or device identifiers

### Documentation

- [ ] Every new user-facing command is documented in `docs/usage/`
      with example input and output
- [ ] Every new protocol fact cites its evidence in
      `specs/re/captures/` or `specs/re/apk-notes/`
- [ ] `ROADMAP.md` changelog updated with a line per milestone on
      close

### Evidence discipline

- [ ] Any new fixture under `crates/*/tests/fixtures/` that contains
      protocol bytes has an accompanying `README.md` citing the
      originating evidence file
- [ ] Any new live-test script is stored under
      `specs/re/captures/ble-live-tests/scripts/` so it can be re-run

---

## Milestone specifications

Each section below gives the **stable** scope, DoR, and DoD for a
milestone. Details that depend on implementation choices (exact type
signatures, test file names, refactor decisions) live in the per-milestone
journey doc, authored at start of that milestone.

---

### M0 — Workspace scaffold & CI baseline ✅ **CLOSED 2026-04-05**

**Goal**: turn the doc-only repo into a buildable Cargo workspace with
a `plaude-cli` binary that responds to `--help` / `--version`, a green
`make lint`/`make test`/`make build` pipeline, and a CI config.

**Depends on**: —

**DoR**:
- [x] `Makefile`, `rustfmt.toml`, `clippy.toml`, `AGENTS.md` exist
- [x] `PLAN.md` §3 crate layout is decided
- [x] No prior Rust code to reconcile with

**DoD**:
- [x] Cargo workspace `Cargo.toml` at repo root with all member crate
      stubs created (one per PLAN.md §3 entry) — 10 crates under
      [`/Cargo.toml`](../../Cargo.toml)
- [x] `crates/plaude-cli` bin crate exposes `plaude-cli` binary matching
      the Makefile's `--bin plaude-cli` target
      ([`crates/plaude-cli/Cargo.toml`](../../crates/plaude-cli/Cargo.toml))
- [x] `plaude-cli --help` exits 0 with a clap-generated help page
      (verified in [`e2e_help.rs`](../../crates/plaude-cli/tests/e2e_help.rs))
- [x] `plaude-cli --version` exits 0 with `plaude-cli <semver>`
      (verified in [`e2e_version.rs`](../../crates/plaude-cli/tests/e2e_version.rs))
- [x] `plaude-cli` with no args exits 2 with a usage hint (CLIG)
      (verified in [`e2e_no_args.rs`](../../crates/plaude-cli/tests/e2e_no_args.rs))
- [x] `make build`, `make test`, `make lint` all exit 0
- [x] E2E tests exist for `--help`, `--version`, and no-args (three
      separate test files)
- [x] CI workflow runs `make lint && make test && make build` on a
      pinned recent stable Rust toolchain
      ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml))
- [x] Top-level `README.md` exists with build instructions and a link
      into `specs/plaude-cli-v1/` ([`/README.md`](../../README.md))
- [x] `docs/usage/index.md` placeholder exists
      ([`docs/usage/index.md`](../../docs/usage/index.md))
- [x] All cross-cutting concerns (top of this file) satisfied
- [x] Journey doc: [`journeys/M00-scaffold.md`](journeys/M00-scaffold.md) — authored and closed with Implementation section ✅

**Key implementation files**:
- Workspace: [`Cargo.toml`](../../Cargo.toml)
- Binary entry point: [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs)
- E2E tests: [`e2e_help.rs`](../../crates/plaude-cli/tests/e2e_help.rs), [`e2e_version.rs`](../../crates/plaude-cli/tests/e2e_version.rs), [`e2e_no_args.rs`](../../crates/plaude-cli/tests/e2e_no_args.rs)
- CI: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- User docs: [`README.md`](../../README.md), [`docs/usage/index.md`](../../docs/usage/index.md)

---

### M1 — Domain types & transport traits ✅ **CLOSED 2026-04-05**

**Goal**: publish the pure-types crate `plaud-domain` and the thin
trait crate `plaud-transport` that every downstream transport (BLE,
USB, Wi-Fi, sim) will implement. No I/O, no transport code.

**Depends on**: M0

**DoR**:
- [x] M0 closed
- [x] `docs/protocol/ble-commands.md` opcode table is up-to-date
- [x] `Constants$CommonSettings$SettingType` enum source-confirmed in
      `specs/re/apk-notes/3.14.0-620/ble-protocol.md`

**DoD**:
- [x] `plaud-domain` crate exports: [`Recording`](../../crates/plaud-domain/src/recording.rs), [`RecordingId`](../../crates/plaud-domain/src/recording.rs), [`RecordingKind`](../../crates/plaud-domain/src/recording.rs), [`DeviceInfo`](../../crates/plaud-domain/src/device.rs), [`DeviceModel`](../../crates/plaud-domain/src/device.rs), [`FirmwareVersion`](../../crates/plaud-domain/src/device.rs), [`BatteryLevel`](../../crates/plaud-domain/src/battery.rs), [`StorageStats`](../../crates/plaud-domain/src/storage.rs), [`Setting`/`SettingValue`](../../crates/plaud-domain/src/setting.rs), [`CommonSettingKey`](../../crates/plaud-domain/src/setting.rs), [`DeviceSerial`](../../crates/plaud-domain/src/device.rs) (with non-leaking `Debug`), [`DeviceCandidate`](../../crates/plaud-domain/src/discovery.rs), [`TransportHint`](../../crates/plaud-domain/src/discovery.rs), [`AuthToken`](../../crates/plaud-domain/src/auth.rs)
- [x] `plaud-transport` crate exports: [`Transport`](../../crates/plaud-transport/src/transport.rs) trait, [`DeviceDiscovery`](../../crates/plaud-transport/src/discovery.rs) trait, [`AuthStore`](../../crates/plaud-transport/src/auth_store.rs) trait, [`Error`](../../crates/plaud-transport/src/error.rs) enum (`thiserror`), `Result<T>` alias
- [x] Every public item carries a doc comment with evidence citation
      for any claim about wire format or opcode semantics
- [x] `missing_docs = "deny"` enforced at workspace level and inherited by every crate
- [x] Trait object safety verified by compile-only tests in [`trait_object_safety.rs`](../../crates/plaud-transport/tests/trait_object_safety.rs)
- [x] `DeviceSerial::fmt::Debug` output does not contain the raw
      serial — asserted in [`device_serial.rs`](../../crates/plaud-domain/tests/device_serial.rs) via a "no 8+ digit run" scan; composition invariant enforced by [`device_info.rs`](../../crates/plaud-domain/tests/device_info.rs)
- [x] Auth tokens are wrapped in `zeroize::Zeroizing` throughout (see [`auth.rs`](../../crates/plaud-domain/src/auth.rs))
- [x] `plaud-domain` does **not** depend on `tokio`, `btleplug`,
      `reqwest`, or any transport crate — verified via `cargo tree -p plaud-domain -e normal`: only `thiserror` + `zeroize`
- [x] All cross-cutting concerns satisfied: 95 tests green, clippy clean, rustfmt clean, 0 dead code, 0 unwraps in production

**Key implementation files**:
- [`crates/plaud-domain/src/lib.rs`](../../crates/plaud-domain/src/lib.rs) — module wiring + re-exports
- [`crates/plaud-domain/src/recording.rs`](../../crates/plaud-domain/src/recording.rs) + [`device.rs`](../../crates/plaud-domain/src/device.rs) + [`battery.rs`](../../crates/plaud-domain/src/battery.rs) + [`storage.rs`](../../crates/plaud-domain/src/storage.rs) + [`setting.rs`](../../crates/plaud-domain/src/setting.rs) + [`discovery.rs`](../../crates/plaud-domain/src/discovery.rs) + [`auth.rs`](../../crates/plaud-domain/src/auth.rs)
- [`crates/plaud-transport/src/lib.rs`](../../crates/plaud-transport/src/lib.rs) + [`error.rs`](../../crates/plaud-transport/src/error.rs) + [`transport.rs`](../../crates/plaud-transport/src/transport.rs) + [`discovery.rs`](../../crates/plaud-transport/src/discovery.rs) + [`auth_store.rs`](../../crates/plaud-transport/src/auth_store.rs)
- Tests: 13 integration test files under `crates/plaud-domain/tests/` and `crates/plaud-transport/tests/`
- Journey: [`journeys/M01-domain-traits.md`](journeys/M01-domain-traits.md) — authored and closed with Implementation section ✅

---

### M2 — `plaud-proto` codec (control + bulk frames) ✅ **CLOSED 2026-04-05**

**Goal**: a pure-Rust codec crate that encodes every control opcode
from the 45-opcode dictionary into wire bytes and decodes every
observed response, including the bulk `0x02`-magic stream with its
`0xFFFFFFFF` end-of-stream sentinel. Fixtures are extracted from the
committed btsnoop walkthroughs and round-trip byte-for-byte.

**Depends on**: M1

**DoR**:
- [x] M1 closed; `plaud-domain` types stable
- [x] `specs/re/apk-notes/3.14.0-620/ble-protocol.md` is the source of
      truth for opcode constructors
- [x] At least one wire example per opcode is extractable from the
      btsnoop walkthroughs

**DoD**:
- [x] [`plaud_proto::Frame`](../../crates/plaud-proto/src/frame.rs) enum with `Control`, `Bulk`, `BulkEnd`, and `Handshake` variants — the last for forward-compat detection of `0xFE11`/`0xFE12` preambles
- [x] `plaud_proto::encode::*` — [`control`](../../crates/plaud-proto/src/encode/mod.rs), [`nullary`](../../crates/plaud-proto/src/encode/mod.rs), [`auth::authenticate`](../../crates/plaud-proto/src/encode/auth.rs), [`file::read_file_chunk`](../../crates/plaud-proto/src/encode/file.rs), [`device::get_device_name`/`get_state`/`get_storage_stats`/`set_privacy`/`close_session`](../../crates/plaud-proto/src/encode/device.rs). Additional opcode wrappers land in M11 as their semantics are resolved.
- [x] [`decode::parse_notification`](../../crates/plaud-proto/src/decode.rs) handles the magic-byte demux: `0x01` → Control, `0x02` → Bulk/BulkEnd, `0xFE..` → Handshake, anything else → `DecodeError::UnknownFrameType`
- [x] [`encode::auth::authenticate`](../../crates/plaud-proto/src/encode/auth.rs) emits the V0095 wire layout (`01 01 00 02 00 00 <token>`) byte-for-byte, verified by [`auth_encode.rs`](../../crates/plaud-proto/tests/auth_encode.rs)
- [x] [`decode::auth_response`](../../crates/plaud-proto/src/decode.rs) returns `Accepted` for status `0x00`, `Rejected` for `0x01`, typed errors otherwise — verified by [`auth_decode.rs`](../../crates/plaud-proto/tests/auth_decode.rs)
- [x] Fixtures provenance table at [`tests/fixtures/README.md`](../../crates/plaud-proto/tests/fixtures/README.md) (fixtures are inlined as `const &[u8]` in each test file rather than stored as binary files, for grep-visibility and to avoid committing any raw capture bytes)
- [x] Round-trip tests for every encoded fixture go through `parse_notification` and match the original bytes
- [x] `proptest` in [`roundtrip.rs`](../../crates/plaud-proto/tests/roundtrip.rs) exercises any `(u16, Vec<u8>)` through encode → decode without panicking
- [x] Decode hot path is zero-copy via `Bytes::slice` (only the incoming buffer + the returned slice allocate)
- [x] `plaud-proto` depends only on `plaud-domain`, `bytes`, and `thiserror` — verified via `cargo tree -p plaud-proto -e normal --depth 1`
- [x] All cross-cutting concerns satisfied: 129 tests green, clippy clean, rustfmt clean, 0 dead code, 0 unwraps / expects in production, no `#[allow]` attributes

**Key implementation files**:
- Sources: [`src/lib.rs`](../../crates/plaud-proto/src/lib.rs), [`constants.rs`](../../crates/plaud-proto/src/constants.rs), [`opcode.rs`](../../crates/plaud-proto/src/opcode.rs), [`frame.rs`](../../crates/plaud-proto/src/frame.rs), [`error.rs`](../../crates/plaud-proto/src/error.rs), [`decode.rs`](../../crates/plaud-proto/src/decode.rs), [`encode/mod.rs`](../../crates/plaud-proto/src/encode/mod.rs), [`encode/auth.rs`](../../crates/plaud-proto/src/encode/auth.rs), [`encode/file.rs`](../../crates/plaud-proto/src/encode/file.rs), [`encode/device.rs`](../../crates/plaud-proto/src/encode/device.rs)
- Tests: 9 integration test files under [`crates/plaud-proto/tests/`](../../crates/plaud-proto/tests/)
- Journey: [`journeys/M02-proto-codec.md`](journeys/M02-proto-codec.md) — authored and closed with Implementation section ✅

---

### M3 — `plaud-sim` v0 (in-process fake)

**Goal**: a deterministic, in-process implementation of the `Transport`
and `DeviceDiscovery` traits that `plaud-sync`, `plaude-cli`, and every
integration test can point at. The sim models CCCD subscription, auth
with status byte, silent soft-reject on bad token, battery read, the
common opcode echo pattern, and the bulk stream state machine for
`ReadFileChunk`. It is the **CI north star** — no milestone after M3
may depend on real hardware for its mandatory tests.

**Depends on**: M2

**DoR**:
- [ ] M2 closed; encoder/decoder round-tripping on all fixtures
- [ ] We have at least one real bulk-transfer trace from the btsnoop
      walkthrough to replay through the sim

**DoD**:
- [ ] `plaud_sim::SimDevice` struct holds: list of recordings, auth
      token (configurable per test), `DeviceInfo`, battery level, a
      "soft-reject" flag, a clock (`tokio::time::MockClock` or
      equivalent)
- [ ] `SimDevice::transport()` returns `impl Transport + DeviceDiscovery`
- [ ] Auth flow:
  - Correct token → response `status = 0x00`, subsequent opcodes work
  - Wrong token → response `status = 0x01`, subsequent vendor opcodes
    drop silently, standard SIG battery still works, connection held
    indefinitely (no timeout) — matches Test 2b behaviour
- [ ] CCCD state machine: write to `0x0011` with value `0x0100` is a
      precondition for any notification traffic; writing `0x0000`
      disables notifications
- [ ] Bulk stream: a `ReadFileChunk(file_id, offset, length)` call
      produces `ceil(length / 80)` bulk frames with offsets stepping
      by 80, then a terminal `BulkEnd` frame with offset `0xFFFFFFFF`
- [ ] `SimDevice::preload_recording(wav_bytes, asr_bytes)` helper for
      tests
- [ ] Replay test: a real recording (from our captures, with audio
      zeroed for privacy) loaded into the sim is retrievable via
      `Transport::read_recording` with bytes matching the preload
- [ ] Failure-injection helpers: `SimDevice::inject_disconnect_after(n_ops)`,
      `SimDevice::inject_delay(per_op_ms)` — used by M9/M12 tests
- [ ] Deterministic: running any test suite against `plaud-sim` twice
      produces identical traces
- [ ] `plaud-sim` depends only on `plaud-domain`, `plaud-transport`,
      `plaud-proto`, `tokio`, `bytes`
- [ ] Integration test exercising the sim from outside its crate,
      pretending to be `plaude-cli`, verifying the full auth → list
      → pull happy path
- [ ] All cross-cutting concerns satisfied

---

### M4 — `plaud-auth` (storage, import, show, clear)

**Goal**: first user-visible auth surface. `plaude auth set-token`,
`plaude auth import <btsnoop.log>`, `plaude auth show`, `plaude auth clear`
all work against the OS keyring with a file fallback at
`~/.config/plaude/token`. No BLE yet; this is purely the credential
store side.

**Depends on**: M1 (AuthStore trait)

**DoR**:
- [ ] M1 closed; `AuthStore` trait exists
- [ ] `specs/re/captures/ble-live-tests/scripts/plaud-test2c.py` is
      the reference parser for `auth import` — re-implement its
      btsnoop-to-token extraction in Rust

**DoD**:
- [ ] `plaud-auth` crate exposes: `KeyringStore`, `FileStore`,
      `ChainStore` (tries keyring first, falls back to file), all
      implementing `AuthStore`
- [ ] `keyring` crate used for the keyring backend (Linux Secret
      Service, macOS Keychain, Windows Credential Manager)
- [ ] File backend writes to `~/.config/plaude/token` mode `0600`,
      parent dir `0700`
- [ ] Token-in-memory type is `zeroize::Zeroizing<String>`
- [ ] `plaude auth set-token <hex>` subcommand: validates input is
      32 ASCII hex chars, stores via `ChainStore::put_token`
- [ ] `plaude auth import <btsnoop.log>` subcommand: shells out to
      `tshark` (or uses a pure-Rust btsnoop parser if available),
      extracts the first `0x52`-opcode `handle=0x000d` value, slices
      the token field, stores it. Prints a fingerprint on success,
      never the raw value.
- [ ] `plaude auth show` prints the SHA-256 fingerprint of the stored
      token (first 16 hex chars) and the storage backend (`keyring`
      or `file`), never the raw token
- [ ] `plaude auth clear` removes the token from all backends and
      returns `0` whether or not one was present
- [ ] E2E tests using `tempfile` to sandbox `$HOME` for file backend
      tests; keyring tests are gated behind a feature flag and skipped
      in CI if no keyring daemon is present
- [ ] Existing `~/.config/plaude/token` (pre-seeded from the research
      phase) is picked up correctly by the file backend on first run
- [ ] All cross-cutting concerns satisfied

---

### M5 — `plaud-transport-ble` (session + transport + discovery) ✅ **CLOSED 2026-04-05**

**Goal**: a `Transport` implementation whose protocol logic (auth,
control round-trip, bulk reassembly, battery delegation, discovery
plumbing) is exercised entirely against hermetic in-memory channels,
so CI stays fast and BLE-hardware-free. The real `btleplug` backend
that turns the channel contract into GATT writes/notifications lands
incrementally in M6/M7/M11 as each vendor opcode becomes real.

**Depends on**: M2, M3, M4

**DoR**:
- [x] M2, M3, M4 closed
- [x] Evidence: auth status byte + soft-reject + battery-without-auth
      semantics live-tested (`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`)

**DoD**:
- [x] [`BleSession`](../../crates/plaud-transport-ble/src/session.rs) — protocol state machine over a `BleChannel`: `authenticate`, `send_control`, `read_bulk`, `is_authenticated`
- [x] `authenticate` emits the exact `plaud_proto::encode::auth::authenticate` frame bytes, awaits the auth notification within 5 s, and flips the `authenticated` flag on `AuthStatus::Accepted`
- [x] `Err(Error::AuthRejected { status: 0x01 })` returned for `AuthStatus::Rejected`; malformed responses return `Error::Protocol`; `Frame::Handshake` (future `0xFE11`/`0xFE12`) returns `Error::Unsupported { capability: "rsa-chacha20-handshake" }`
- [x] `send_control` rejects with `Error::AuthRequired` when called before `authenticate`, writes the control frame, reads the next notification, surfaces `Error::Protocol` on opcode mismatch, and returns the payload bytes on match
- [x] [`BulkReassembler`](../../crates/plaud-transport-ble/src/bulk.rs) — validates monotone offsets, validates `file_id` consistency across frames, errors on `finish` without a `BulkEnd`, concatenates payloads in-order
- [x] `BleSession::read_bulk` drives the reassembler over the channel end-to-end and returns the fully reassembled `Bytes`
- [x] [`BleTransport`](../../crates/plaud-transport-ble/src/transport.rs) implements `plaud_transport::Transport`; `battery()` delegates to an injected [`BatteryReader`](../../crates/plaud-transport-ble/src/battery.rs) (`FixedBatteryReader` for tests) **without touching session auth state**, matching Test 2b live evidence; every other method returns `Error::Unsupported { capability }` pointing at the milestone that will land it (M6/M7/M11)
- [x] [`BleDiscovery`](../../crates/plaud-transport-ble/src/discovery.rs) implements `DeviceDiscovery`; `scan` delegates to an injectable `ScanProvider` trait object (real backend lands alongside the first hardware-driven milestone), `connect` returns `Error::Unsupported` in M5 scope
- [x] [`BleChannel::loopback_pair`](../../crates/plaud-transport-ble/src/channel.rs) — hermetic `tokio::mpsc` test factory returning a `BleChannel` + `TestPeer` handle; every M5 test uses this, zero tests require hardware
- [x] All cross-cutting concerns satisfied: clippy clean (`-D warnings`), rustfmt clean, every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]` attributes, zero dead code
- [x] Deferred to M6/M7/M11 by design (documented as `Error::Unsupported` with stable capability strings, pinned by [`tests/transport_unsupported.rs`](../../crates/plaud-transport-ble/tests/transport_unsupported.rs)): `list_recordings`, `read_recording`, `device_info`, `storage_stats`, `read_setting`, `write_setting`, `start_recording`, `stop_recording`, `pause_recording`, `resume_recording`, `set_privacy`, and `BleDiscovery::connect`
- [x] Deferred to M12 hardening: opcode-tag correlation (M5 uses FIFO ordering), `AsyncWrite`-sink streaming for bulk (M5 reassembles to `Vec<u8>` which is sufficient for M7 single-recording pulls), opt-in `hw-tests` feature with real-hardware smoke
- [x] Test suite: 24 integration tests across `auth_flow.rs`, `control_roundtrip.rs`, `bulk_reassembly.rs`, `battery_transport.rs`, `transport_unsupported.rs`, `discovery.rs` — all green, all hermetic

**Key implementation files**:
- Sources: [`src/lib.rs`](../../crates/plaud-transport-ble/src/lib.rs), [`constants.rs`](../../crates/plaud-transport-ble/src/constants.rs), [`channel.rs`](../../crates/plaud-transport-ble/src/channel.rs), [`bulk.rs`](../../crates/plaud-transport-ble/src/bulk.rs), [`battery.rs`](../../crates/plaud-transport-ble/src/battery.rs), [`session.rs`](../../crates/plaud-transport-ble/src/session.rs), [`transport.rs`](../../crates/plaud-transport-ble/src/transport.rs), [`discovery.rs`](../../crates/plaud-transport-ble/src/discovery.rs)
- Tests: 6 integration test files under [`crates/plaud-transport-ble/tests/`](../../crates/plaud-transport-ble/tests/)
- Journey: [`journeys/M05-transport-ble.md`](journeys/M05-transport-ble.md) — authored and closed with Implementation section ✅

---

### M6 — `plaude-cli battery` + `plaude-cli device info` ✅ **CLOSED 2026-04-06**

**Goal**: first user-visible commands that drive a full transport
stack. Two small commands plus the reusable `TransportProvider`
abstraction every later CLI surface will plug into, plus a stable
exit-code contract for auth failures. All e2e tests run against
`plaud-sim`; the real btleplug hardware backend is still deferred.

**Depends on**: M5 (and M3 for sim, M4 for token storage)

**DoR**:
- [x] M3 closed (sim has real `device_info` / `battery` / `storage`)
- [x] M4 closed (token store + `--config-dir` sandbox flag exist)
- [x] M5 closed (stable `BleTransport` surface — not touched by M6)

**DoD**:
- [x] [`plaude-cli battery`](../../docs/usage/battery.md) subcommand: exposes the SIG-analogue battery path, works **without an auth token**, supports `--output text|json`
- [x] [`plaude-cli device info`](../../docs/usage/device-info.md) subcommand: queries `device_info` + `storage` through an authenticated transport, prints a formatted summary, supports `--output text|json`, requires a token
- [x] Global [`--backend <sim|ble>`](../../crates/plaude-cli/src/commands/backend.rs) flag selects the runtime [`TransportProvider`](../../crates/plaude-cli/src/commands/backend.rs): `SimProvider` wraps `plaud_sim::SimDevice`; `BleProvider` is a stub returning `Error::Unsupported { capability: "ble-hardware-backend" }` until the btleplug wire-up milestone
- [x] Missing-token path returns distinct exit code **`77` (`EX_NOPERM`)** with stderr pointing at `plaude-cli auth --help`
- [x] Rejected-token path returns distinct exit code **`78` (`EX_CONFIG`)** with stderr naming the status byte and pointing at `plaude-cli auth bootstrap`
- [x] `DispatchError::from_transport_error` maps every `plaud_transport::Error` variant to the right exit-code path, including a stable "ble backend not yet wired" runtime message so wrapper scripts can detect the stub without false-matching on "Unsupported" text
- [x] `PLAUDE_SIM_REJECT=1` env hook on `SimProvider::connect_authenticated` drives the rejected-token test path deterministically without needing a second sim builder entry point
- [x] E2E tests against `plaud-sim`: [`tests/e2e_battery.rs`](../../crates/plaude-cli/tests/e2e_battery.rs) (4 tests: text output, no-token-required, json schema, ble-backend stub) and [`tests/e2e_device_info.rs`](../../crates/plaude-cli/tests/e2e_device_info.rs) (5 tests: text output, json schema, missing-token → 77, rejected-token → 78, no-token-leak assertion)
- [x] Unit tests pin the `TransportProvider` contract, the JSON schemas for both commands, and the text prefixes used as mutation-kill assertions
- [x] [`docs/usage/battery.md`](../../docs/usage/battery.md) and [`docs/usage/device-info.md`](../../docs/usage/device-info.md) shipped; `docs/usage/index.md` updated with the two new exit codes and the two commands marked ✅
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green (238 tests), every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]` attributes, zero dead code
- [x] Deferred: real `BleTransport::device_info` / `storage` implementations (lands alongside the btleplug backend so they have a real channel to drive); they remain `Unsupported` stubs in M6. The CLI surface is ready to pick them up transparently via the existing `TransportProvider` abstraction.

**Key implementation files**:
- Sources: [`src/main.rs`](../../crates/plaude-cli/src/main.rs) (new exit codes + dispatch), [`src/commands/backend.rs`](../../crates/plaude-cli/src/commands/backend.rs), [`src/commands/battery.rs`](../../crates/plaude-cli/src/commands/battery.rs), [`src/commands/device.rs`](../../crates/plaude-cli/src/commands/device.rs), [`src/commands/output.rs`](../../crates/plaude-cli/src/commands/output.rs)
- Tests: [`tests/e2e_battery.rs`](../../crates/plaude-cli/tests/e2e_battery.rs), [`tests/e2e_device_info.rs`](../../crates/plaude-cli/tests/e2e_device_info.rs) + unit tests in each command module
- Docs: [`docs/usage/battery.md`](../../docs/usage/battery.md), [`docs/usage/device-info.md`](../../docs/usage/device-info.md)
- Journey: [`journeys/M06-battery-device-info.md`](journeys/M06-battery-device-info.md) — authored and closed with Implementation section ✅

---

### M7 — `plaude-cli files list` + `plaude-cli files pull-one` ✅ **CLOSED 2026-04-06**

**Goal**: the first real recording-management capability. `files list`
enumerates recordings; `files pull-one` downloads a single recording's
`.WAV` + `.ASR` pair to disk with a progress bar. Hermetic against
`plaud-sim`; real-hardware support lands with the btleplug backend.

**Depends on**: M5, M6 (and M3 for the sim fixture, M4 for auth storage).

**DoR**:
- [x] M5, M6 closed
- [x] `plaud-sim` serves preloaded recordings via `read_recording` with deterministic bytes ([`crates/plaud-sim/src/transport.rs`](../../crates/plaud-sim/src/transport.rs))

**DoD**:
- [x] [`plaude-cli files list`](../../docs/usage/files.md) prints a table of recordings (id, kind, started_at, wav_size, asr_size) with `--output text|json`
- [x] [`plaude-cli files pull-one <id>`](../../docs/usage/files.md) downloads a recording as paired `<id>.wav` + `<id>.asr` files; `-o DIR` controls the destination directory (created on demand); progress reported via `indicatif`
- [x] New trait method [`Transport::read_recording_asr`](../../crates/plaud-transport/src/transport.rs) added so the CLI can fetch ASR sidecars through the transport boundary instead of reaching into sim-only accessors; implemented for real on [`SimTransport`](../../crates/plaud-sim/src/transport.rs), stubbed as `Unsupported` on [`BleTransport`](../../crates/plaud-transport-ble/src/transport.rs) until the btleplug backend ships, contract pinned by the extended [`transport_unsupported.rs`](../../crates/plaud-transport-ble/tests/transport_unsupported.rs) matrix
- [x] `SimProvider` in the CLI backend preloads exactly one deterministic recording (`1775393534` / `WAV-BYTES-FROM-SIM` / `ASR-BYTES-FROM-SIM`) so every e2e test asserts against stable bytes
- [x] `--resume` implements **idempotent-skip** semantics: if both target files already exist at the expected byte count, the command is a no-op and emits `<id> already up to date`; if either is partial, the partial file is rewritten from scratch. Mid-offset resume (start-at-byte-N) is deferred — it requires a range-read method on the transport trait that lands alongside the btleplug backend or M12 hardening.
- [x] Unknown id → runtime error (exit 1) with a message naming the id, and no partial files left on disk
- [x] E2E tests against `plaud-sim`: [`tests/e2e_files_list.rs`](../../crates/plaude-cli/tests/e2e_files_list.rs) (3 tests: text + json + missing-token) and [`tests/e2e_files_pull.rs`](../../crates/plaude-cli/tests/e2e_files_pull.rs) (5 tests: happy path with byte-equality on both files, nested output dir creation, `--resume` skip, `--resume` rewrite, unknown id)
- [x] Unit tests in [`commands/files.rs`](../../crates/plaude-cli/src/commands/files.rs) pin the JSON schema, the text table header, and the file-extension constants as mutation-kill assertions
- [x] [`docs/usage/files.md`](../../docs/usage/files.md) added; [`docs/usage/index.md`](../../docs/usage/index.md) updated with the two new command rows
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green (251 tests), every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]`, zero dead code
- [x] **Scope-reductions (documented in the journey "Context" section)**: mid-offset resume, streaming writes without an in-memory buffer, device-side CRC validation via `tnt_get_file_crc`, and the hardware-gated smoke test are deferred to later milestones. All four require plumbing that either does not exist yet (range-reads, CRC opcode) or depends on the btleplug backend. The M7 command surface is structured so each deferral can be slotted in later without a behavioural change for existing callers.

**Key implementation files**:
- Sources: [`crates/plaud-transport/src/transport.rs`](../../crates/plaud-transport/src/transport.rs) (trait extension), [`crates/plaud-sim/src/transport.rs`](../../crates/plaud-sim/src/transport.rs), [`crates/plaud-transport-ble/src/transport.rs`](../../crates/plaud-transport-ble/src/transport.rs), [`crates/plaud-transport-ble/src/constants.rs`](../../crates/plaud-transport-ble/src/constants.rs), [`crates/plaude-cli/src/commands/files.rs`](../../crates/plaude-cli/src/commands/files.rs), [`crates/plaude-cli/src/commands/backend.rs`](../../crates/plaude-cli/src/commands/backend.rs) (preloaded recording), [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs) (new `Files` subcommand dispatch)
- Tests: [`crates/plaude-cli/tests/e2e_files_list.rs`](../../crates/plaude-cli/tests/e2e_files_list.rs), [`crates/plaude-cli/tests/e2e_files_pull.rs`](../../crates/plaude-cli/tests/e2e_files_pull.rs), [`crates/plaud-sim/tests/recordings.rs`](../../crates/plaud-sim/tests/recordings.rs) (new ASR tests), [`crates/plaud-transport-ble/tests/transport_unsupported.rs`](../../crates/plaud-transport-ble/tests/transport_unsupported.rs) (new ASR stub test)
- Docs: [`docs/usage/files.md`](../../docs/usage/files.md)
- Journey: [`journeys/M07-files-list-pull.md`](journeys/M07-files-list-pull.md) — authored and closed with Implementation section ✅

---

### M8 — `plaude-cli auth bootstrap` (fake peripheral) ✅ **CLOSED 2026-04-06**

**Goal**: the one-time onboarding command. Capture the Plaud auth
token from the user's own phone app without `adb`, `tshark`, or any
third-party tooling. M8 ships the hermetic protocol layer and the
CLI wiring; the real BlueZ GATT advertisement lands alongside the
btleplug central in a later milestone (both share D-Bus plumbing).

**Depends on**: M2, M4, M5

**DoR**:
- [x] M2, M4, M5 closed
- [x] Real-hardware crate selected: **`bluer`** (bluez-rs, D-Bus,
      actively maintained, the only mature Rust crate for
      peripheral-mode BlueZ). Wired behind a future `bluer-backend`
      feature flag; M8 does not actually pull it in since it cannot
      be CI-tested.
- [x] `AUTH_PREFIX` layout pinned by M2 evidence

**DoD**:
- [x] [`plaud_proto::decode::parse_auth_write`](../../crates/plaud-proto/src/decode.rs) — inverse of `encode::auth::authenticate`, decodes a captured phone write into an `AuthToken`; round-trip tested against both 16- and 32-char tokens and against prefix-mismatch / non-hex / short-input failure modes in [`tests/auth_write_decode.rs`](../../crates/plaud-proto/tests/auth_write_decode.rs)
- [x] [`plaud_transport_ble::bootstrap`](../../crates/plaud-transport-ble/src/bootstrap/) submodule — `BootstrapChannel` + `PhoneChannel` mpsc pair modelling the peripheral side of one GATT connection, `BootstrapSession::run(timeout)` driving the three-step handshake (receive → decode → send mock accepted), `BootstrapOutcome { token }`, `BootstrapError { Timeout, PhoneDisconnected, DecodeFailed }`, `BOOTSTRAP_DEFAULT_TIMEOUT = 120s`
- [x] [`LoopbackBootstrap` + `TestPhone`](../../crates/plaud-transport-ble/src/bootstrap/loopback.rs) — hermetic in-process implementation exercised by both the integration test and the CLI's `--backend sim` runtime path
- [x] [`plaude-cli auth bootstrap`](../../docs/usage/auth-bootstrap.md) subcommand with `--timeout <SECS>` (default 120) and the existing global `--backend sim|ble` flag
- [x] Sim runtime path: CLI spawns a `LoopbackBootstrap`, writes a deterministic 32-hex token from a fake phone task, awaits the session outcome, stores the captured token via the existing `AuthStore` chain, prints `Token captured. Fingerprint: <16-hex>`, exits 0. Running `plaude-cli auth show` immediately after returns the same fingerprint — pinned by `auth_bootstrap_sim_stores_token_so_auth_show_sees_it_afterwards`
- [x] BLE runtime path: returns a runtime error with the stable `ble-hardware-backend` capability marker, documented as deferred
- [x] Mock auth-accepted notification pushed back to the phone after a successful capture so a real phone app would not display an error (`MOCK_AUTH_ACCEPTED_FRAME = 01 01 00 00`). Pinned by `loopback_peripheral_sends_back_mock_accepted_notification_on_success`.
- [x] Hermetic e2e tests: [`tests/bootstrap_session.rs`](../../crates/plaud-transport-ble/tests/bootstrap_session.rs) (4 tests covering happy path, notification echo, timeout, decode failure) and [`tests/e2e_auth_bootstrap.rs`](../../crates/plaude-cli/tests/e2e_auth_bootstrap.rs) (4 tests covering sim success, store round-trip, ble stub, idempotent double-run)
- [x] [`docs/usage/auth-bootstrap.md`](../../docs/usage/auth-bootstrap.md) shipped with the "put your real device to sleep first" precondition and a clear explanation of the sim vs. ble backend split
- [x] `docs/usage/index.md` updated with the ✅ sim-path row
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green (264 tests), every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]`, zero dead code
- [x] **Scope-reductions (documented in the journey)**: the real BlueZ GATT advertisement + server, the opportunistic sidechannel-opcode capture for `sendHttpToken` / `sendFindMyToken` / `setSoundPlusToken` (M14), and the "second OS process acting as phone" e2e flow are deferred. The hermetic loopback IS the "second process" inside a single runtime, which gives byte-equal coverage of the protocol handshake without needing two processes.

**Key implementation files**:
- Sources: [`crates/plaud-proto/src/decode.rs`](../../crates/plaud-proto/src/decode.rs) (`parse_auth_write`), [`crates/plaud-proto/src/error.rs`](../../crates/plaud-proto/src/error.rs) (two new `DecodeError` variants), [`crates/plaud-transport-ble/src/bootstrap/mod.rs`](../../crates/plaud-transport-ble/src/bootstrap/mod.rs), [`session.rs`](../../crates/plaud-transport-ble/src/bootstrap/session.rs), [`loopback.rs`](../../crates/plaud-transport-ble/src/bootstrap/loopback.rs), [`crates/plaude-cli/src/commands/auth.rs`](../../crates/plaude-cli/src/commands/auth.rs) (`Bootstrap` variant + `bootstrap_sim` handler + `map_bootstrap_error`)
- Tests: [`crates/plaud-proto/tests/auth_write_decode.rs`](../../crates/plaud-proto/tests/auth_write_decode.rs), [`crates/plaud-transport-ble/tests/bootstrap_session.rs`](../../crates/plaud-transport-ble/tests/bootstrap_session.rs), [`crates/plaude-cli/tests/e2e_auth_bootstrap.rs`](../../crates/plaude-cli/tests/e2e_auth_bootstrap.rs)
- Docs: [`docs/usage/auth-bootstrap.md`](../../docs/usage/auth-bootstrap.md)
- Journey: [`journeys/M08-auth-bootstrap-peripheral.md`](journeys/M08-auth-bootstrap-peripheral.md) — authored and closed with Implementation section ✅

---

### M9 — `plaude-cli sync <dir>` ✅ **CLOSED 2026-04-06**

**Goal**: the headline command. Idempotent mirror of all device
recordings to a local directory, with a JSON state file, graceful
handling of deleted-on-device entries, and a dry-run mode.

**Depends on**: M7 (single-file pull primitives)

**DoR**:
- [x] M7 closed; single-file pull is reliable

**DoD**:
- [x] [`plaude-cli sync <dir>`](../../docs/usage/sync.md) subcommand
- [x] State file at [`<dir>/.plaude-sync.json`](../../crates/plaude-cli/src/commands/sync/state.rs) — versioned schema with `inventory_hash` (SHA-256 over sorted `(id, wav_size, asr_size)` triples) and per-recording `{wav_size, asr_size, pulled_at_unix_seconds}` entries. Atomic writes via `.tmp` + rename so a crash mid-write cannot truncate the file.
- [x] Idempotent: running twice in a row with no device changes is a no-op, exits 0, prints `nothing to do`. Pinned by `sync_second_run_with_same_device_is_a_noop`.
- [x] Incremental: new recordings are pulled; the existing ones are not re-downloaded. Pinned by `sync_incremental_only_pulls_the_newly_added_recording`.
- [x] Deleted-on-device: flagged in stderr (`deleted on device (still on disk): <id>`), local files left untouched, state entry removed so a future re-add is treated as new. Pinned by `sync_reports_deleted_on_device_without_removing_local_files`.
- [x] `--dry-run` prints the plan (`would pull: <id>` lines) and exits 0 without touching the state file or writing any media files. Pinned by `sync_dry_run_prints_plan_without_writing_files`.
- [x] `--concurrency N` flag is accepted and parsed (default `1`). BLE is serial so the value is not currently consulted; documented as reserved for later hardening.
- [x] File-grained resume: an interrupted run leaves successfully-pulled files on disk and a state file that reflects them, so the next run pulls only the stragglers. Byte-grained (mid-file) resume deferred to the btleplug-backend / M12 hardening milestone along with the `read_recording_range` trait extension.
- [x] `SimProvider` extended with the `PLAUDE_SIM_RECORDINGS` env var (comma-separated basename list; deterministic WAV+ASR payloads derived per basename) so CLI e2e tests can vary the fixture between runs within a single test without a second process.
- [x] E2E tests against `plaud-sim`: [`tests/e2e_sync.rs`](../../crates/plaude-cli/tests/e2e_sync.rs) — 7 tests covering empty-device, one-file, no-op re-run, incremental add, dry-run, deleted-on-device, missing-token → exit 77.
- [x] Unit tests in [`commands/sync/state.rs`](../../crates/plaude-cli/src/commands/sync/state.rs) pin `inventory_hash` stability under reordering, hash sensitivity to size changes and new recordings, JSON round-trip, and the current schema version.
- [x] [`docs/usage/sync.md`](../../docs/usage/sync.md) shipped; `docs/usage/index.md` updated with a ✅ row.
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green (277 tests), every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]`, zero dead code.
- [x] **Scope-reductions (documented in the journey)**: real `SIGINT`/`SIGTERM` → exit-130 integration + `indicatif` per-file progress bars are deferred to M12 hardening along with byte-grained resume. The file-grained resume semantic plus the JSON state file already provide safe interruption handling at file boundaries — the user restarts the command, gets the same result, no data loss. The `--concurrency` flag surface is in place so M12 can drop in eager-prefetch without a CLI-compatibility break.

**Key implementation files**:
- Sources: [`crates/plaude-cli/src/commands/sync/mod.rs`](../../crates/plaude-cli/src/commands/sync/mod.rs) (top-level dispatch + `Plan`), [`state.rs`](../../crates/plaude-cli/src/commands/sync/state.rs) (`SyncState` + `inventory_hash`), [`crates/plaude-cli/src/commands/backend.rs`](../../crates/plaude-cli/src/commands/backend.rs) (`PLAUDE_SIM_RECORDINGS` env hook), [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs) (new `Sync` subcommand)
- Tests: [`crates/plaude-cli/tests/e2e_sync.rs`](../../crates/plaude-cli/tests/e2e_sync.rs), unit tests in `commands/sync/state.rs` + `commands/sync/mod.rs`
- Docs: [`docs/usage/sync.md`](../../docs/usage/sync.md)
- Journey: [`journeys/M09-sync.md`](journeys/M09-sync.md) — authored and closed with Implementation section ✅

---

### M10 — `plaud-transport-usb` fallback ✅ **CLOSED 2026-04-06**

**Goal**: USB-MSC transport as a convenience fallback for users on
pre-deprecation firmware.

**Depends on**: M1, M6, M9

**DoR**:
- [x] M1 closed (Transport trait + domain types)
- [x] `docs/protocol/file-formats.md` WAV / ASR / MODEL.txt specs finalised

**DoD**:
- [x] [`UsbTransport`](../../crates/plaud-transport-usb/src/transport.rs) implements `Transport`: real `device_info`, `list_recordings`, `read_recording`, `read_recording_asr`; every other method returns `Error::Unsupported { capability: "usb-transport-unsupported" }`
- [x] [`model_txt::parse`](../../crates/plaud-transport-usb/src/model_txt.rs) — fixed-field parser for `MODEL.txt` (line 1 = product + firmware via `FirmwareVersion::parse_model_txt_line`, line 2 = `Serial No.:<serial>`); 7 unit tests pinning happy path, build-stamp preservation, every error variant
- [x] [`WavSanitiser`](../../crates/plaud-transport-usb/src/wav.rs) — in-place zeroing of bytes `0x2C..0x42` (the `SN:<18-digit-serial>\0` region inside the `pad ` RIFF chunk) without shifting audio offsets; validates RIFF/WAVE magic before touching any byte; idempotent; 7 unit tests
- [x] [`listing::list_recordings`](../../crates/plaud-transport-usb/src/listing.rs) — walks `{NOTES,CALLS}/<YYYYMMDD>/<unix>.{WAV,ASR}`, pairs files by basename, surfaces only complete `(WAV, ASR)` pairs, sorts by id; 4 unit tests covering NOTES, CALLS, unpaired WAV, empty root
- [x] `Backend::Usb` variant + global `--mount <PATH>` flag; `UsbProvider` wraps `UsbTransport::new(mount)` and returns the same transport for both `connect_anonymous` and `connect_authenticated` (USB has no auth split — the filesystem is world-readable); all commands (`battery`, `device info`, `files list`, `files pull-one`, `sync`) route through the backend
- [x] `--sanitise` flag on `sync`: applies `WavSanitiser` to every WAV pulled, regardless of source backend. On non-RIFF input the sanitiser is a silent no-op (graceful degradation for sim fixtures)
- [x] `.ASR` sidecars are copied through unchanged
- [x] Deprecation notice on stderr for every USB-backend run: `"Plaud has announced USB will be disabled in a future firmware update..."`
- [x] 7 integration tests in [`tests/transport_usb.rs`](../../crates/plaud-transport-usb/tests/transport_usb.rs) (device info, list, read wav, read asr, unknown id → NotFound, battery unsupported, set_privacy unsupported) against a plain tempdir fixture — no VFAT mount required
- [x] [`docs/usage/usb.md`](../../docs/usage/usb.md) shipped with per-OS mount hints, the deprecation notice, and the `--sanitise` contract
- [x] `docs/usage/index.md` updated
- [x] All cross-cutting concerns satisfied: 302 tests green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented
- [x] **Scope-reduction**: auto-discovery of the VFAT mount path (Linux/macOS/Windows block-device enumeration + VFAT-label matching) is deferred — M10 ships with the explicit `--mount <PATH>` flag. Documented in `docs/usage/usb.md` with common per-OS mount-point hints and a shell-alias recipe.

**Key implementation files**:
- Sources: [`crates/plaud-transport-usb/src/`](../../crates/plaud-transport-usb/src/) (5 modules), [`crates/plaude-cli/src/commands/backend.rs`](../../crates/plaude-cli/src/commands/backend.rs) (`Backend::Usb` + `UsbProvider`), [`crates/plaude-cli/src/commands/sync/mod.rs`](../../crates/plaude-cli/src/commands/sync/mod.rs) (`--sanitise` flag), [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs) (`--mount` flag + deprecation notice)
- Tests: [`crates/plaud-transport-usb/tests/transport_usb.rs`](../../crates/plaud-transport-usb/tests/transport_usb.rs) + unit tests in `model_txt.rs`, `wav.rs`, `listing.rs`
- Docs: [`docs/usage/usb.md`](../../docs/usage/usb.md)
- Journey: [`journeys/M10-transport-usb.md`](journeys/M10-transport-usb.md) — authored and closed with Implementation section ✅

---

### M11 — Settings + recording remote control ✅ **CLOSED 2026-04-06**

**Goal**: the long tail of vendor opcodes exposed as CLI subcommands.
`plaude settings get/set`, `plaude record start/stop/pause/resume`,
`plaude device name/privacy`. Closes out the Flutter action surface
that is reasonable to support in an offline CLI.

**Depends on**: M5, M6

**DoR**:
- [x] M5, M6 closed
- [x] `CommonSettingKey` enum complete in `plaud-domain`

**DoD**:
- [x] [`plaude-cli settings list`](../../docs/usage/settings.md) enumerates every `CommonSettingKey` with a stored value on the device, with `--output text|json`
- [x] [`plaude-cli settings get <name>`](../../docs/usage/settings.md) — one-shot read; unknown names → exit 2 (usage)
- [x] [`plaude-cli settings set <name> <value>`](../../docs/usage/settings.md) — write; value parsed as bool/u8/u32; invalid → exit 2
- [x] [`plaude-cli record start|stop|pause|resume`](../../docs/usage/record.md) — recording pipeline control with the sim's recording state machine enforcing valid transitions; invalid transitions → exit 1
- [x] [`plaude-cli device privacy on|off`](../../crates/plaude-cli/src/commands/device.rs) — opcode `0x0067 SetPrivacy` via `Transport::set_privacy`
- [x] [`plaude-cli device name`](../../crates/plaude-cli/src/commands/device.rs) — reads the device local name from `device_info().local_name`
- [x] `plaude-cli device name --set <name>` is deferred: the `SetDeviceName` opcode (`0x006B`) has not been observed on the wire and is not in the proto encoder. Read is via the existing `device_info` path.
- [x] Every command requires auth: missing token → exit 77, rejected → exit 78. Same `DispatchError::from_transport_error` mapping as M6.
- [x] E2E tests against `plaud-sim` for every subcommand: [`e2e_settings.rs`](../../crates/plaude-cli/tests/e2e_settings.rs) (7 tests), [`e2e_record.rs`](../../crates/plaude-cli/tests/e2e_record.rs) (5 tests), [`e2e_device_privacy.rs`](../../crates/plaude-cli/tests/e2e_device_privacy.rs) (6 tests)
- [x] [`docs/usage/settings.md`](../../docs/usage/settings.md) and [`docs/usage/record.md`](../../docs/usage/record.md) added; [`docs/usage/index.md`](../../docs/usage/index.md) updated
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green, every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]`, zero dead code
- [x] New domain additions: `CommonSettingKey::from_name`, `SettingValue::parse`, `SettingValue` `Display` impl, `UnknownSettingName` and `SettingValueParseError` error types — 10 new unit tests in `setting.rs`
- [x] Sim backend preloads 3 default settings (`enable-vad=true`, `mic-gain=20`, `auto-power-off=300`) so `settings list` always has deterministic data in tests

**Key implementation files**:
- Sources: [`crates/plaude-cli/src/commands/settings.rs`](../../crates/plaude-cli/src/commands/settings.rs), [`crates/plaude-cli/src/commands/record.rs`](../../crates/plaude-cli/src/commands/record.rs), [`crates/plaude-cli/src/commands/device.rs`](../../crates/plaude-cli/src/commands/device.rs) (extended with `Privacy` + `Name`), [`crates/plaud-domain/src/setting.rs`](../../crates/plaud-domain/src/setting.rs) (extended)
- Tests: [`crates/plaude-cli/tests/e2e_settings.rs`](../../crates/plaude-cli/tests/e2e_settings.rs), [`crates/plaude-cli/tests/e2e_record.rs`](../../crates/plaude-cli/tests/e2e_record.rs), [`crates/plaude-cli/tests/e2e_device_privacy.rs`](../../crates/plaude-cli/tests/e2e_device_privacy.rs), [`crates/plaud-domain/tests/setting.rs`](../../crates/plaud-domain/tests/setting.rs)
- Docs: [`docs/usage/settings.md`](../../docs/usage/settings.md), [`docs/usage/record.md`](../../docs/usage/record.md)
- Journey: [`journeys/M11-settings-record-control.md`](journeys/M11-settings-record-control.md) — authored and closed with Implementation section ✅

---

### M12 — Hardening → v1.0.0 release candidate ✅ **CLOSED 2026-04-06**

**Goal**: polish the v1 feature set into a releasable state. Exit-code
audit, structured logging, configurable timeouts, man pages, troubleshooting
docs, security review, privacy disclosure.

**Depends on**: M5–M11

**DoR**:
- [x] M5 through M11 closed
- [x] No open P0/P1 issues in backlog

**DoD**:
- [x] Retry with exponential backoff: **deferred** — the retry/backoff middleware belongs in the transport layer and needs real-hardware validation. M12 ships the configurable timeout (the prerequisite). The `Timeout` transport error variant already maps to exit 69.
- [x] All timeouts are configurable: global `--timeout <SECS>` flag and `PLAUDE_TIMEOUT` env var (default 30s). Documented in [`docs/usage/troubleshooting.md`](../../docs/usage/troubleshooting.md).
- [x] Multi-device support: **deferred** — requires real btleplug backend. The `--device` flag surface is tracked for the follow-up.
- [x] CLIG exit-code audit: every command maps to {0, 1, 2, 69, 77, 78} per `sysexits(3)`, documented in [`docs/usage/exit-codes.md`](../../docs/usage/exit-codes.md). New exit code 69 (`EX_UNAVAILABLE`) for transport-layer failures (BLE not wired, connection dropped, timeout). `DispatchError::Unavailable` variant added.
- [x] Man pages: `make man` target generates man pages via `help2man` from the compiled binary; `make install` copies them to `~/.local/share/man/man1/`.
- [x] `docs/usage/` has a page per top-level subcommand plus [`troubleshooting.md`](../../docs/usage/troubleshooting.md) covering every known error path.
- [x] Structured JSON logging via `tracing-subscriber`: `RUST_LOG=info plaude-cli ...` produces human-readable text on stderr; `--log-format json` emits JSON lines. Logs go to stderr, stdout stays clean. No tracing calls exist in the codebase yet (this is infrastructure, not instrumentation), but the subscriber is initialised so any future `tracing::info!()` calls are routed correctly.
- [x] Security review: exhaustive grep of all `.rs` source files for token/serial/secret leaks. Findings: `AuthToken` has redacting Debug, `DeviceSerial` has redacting Debug, `.as_str()`/`.reveal()` calls are all in legitimate contexts (storage, wire encoding, user-requested output). No credentials in any error message. No `dbg!()` macros. Zero issues found.
- [x] Privacy disclosure in [`README.md`](../../README.md) and in the output of `plaude-cli --about`: cleartext BLE traffic, forensic serial watermark in WAVs, long-lived credential.
- [x] Release checklist: deferred to the human release manager. `CHANGELOG.md` and version bump to `1.0.0-rc.1` are manual steps outside the agent's scope.
- [x] All cross-cutting concerns satisfied: `make lint` clean, `make test` green (334 tests), every public item documented, zero `unwrap`/`expect` in production, zero `#[allow]`, zero dead code.

**Key implementation files**:
- Sources: [`crates/plaude-cli/src/main.rs`](../../crates/plaude-cli/src/main.rs) (new `LogFormat` enum, `init_logging`, `--about`/`--timeout`/`--log-format` flags, `DispatchError::Unavailable` variant, `EXIT_UNAVAILABLE = 69`)
- Sources: [`crates/plaude-cli/src/commands/auth.rs`](../../crates/plaude-cli/src/commands/auth.rs) (BLE bootstrap → `Unavailable` instead of `Runtime`)
- Tests: [`crates/plaude-cli/tests/e2e_exit_codes.rs`](../../crates/plaude-cli/tests/e2e_exit_codes.rs) (4 tests: BLE unavailable for battery/device info, about flag, missing token)
- Docs: [`docs/usage/exit-codes.md`](../../docs/usage/exit-codes.md), [`docs/usage/troubleshooting.md`](../../docs/usage/troubleshooting.md)
- Build: [`Makefile`](../../Makefile) (`man` + updated `install` targets)
- Journey: [`journeys/M12-hardening.md`](journeys/M12-hardening.md) — authored and closed with Implementation section ✅

---

### M13–M16 — Stretch goals

These are deferred beyond v1.0.0. Stub specs only, to be expanded
into full milestone sections and journey docs when the work begins.

- **M13 — Wi-Fi Fast Transfer transport**: requires one `re-wifi-probe`
  session to finalise the hotspot wire format. Implementation adds
  `plaud-transport-wifi` and modifies `plaude sync` to auto-switch
  for files above a size threshold. DoR: R3 evidence complete.
- **M14 — Self-hosted HTTP sink**: embeds an `axum` server in the
  CLI, pushes the user's laptop SSID + HTTP token to the device
  via the `SyncInIdle` opcodes, the device uploads to us forever
  after. Requires M13 and the sidechannel capture from M8 (for
  the HTTP token). DoR: M13 closed.
- **M15 — `plaude transcribe` via whisper.cpp** ✅ **CLOSED 2026-04-06**:
  thin wrapper invoking a user-supplied `whisper.cpp` CLI binary
  (`whisper-cli` on `$PATH` by default, overridable via `--whisper-bin`
  or `PLAUDE_WHISPER_BIN`). Requires `--model` (or `PLAUDE_WHISPER_MODEL`)
  pointing at a GGML model file. Supports `--language` hint and
  `--output-format txt|srt|vtt`. 6 e2e tests with mock shell-script
  whisper binaries. Docs at [`docs/usage/transcribe.md`](../../docs/usage/transcribe.md).
  See [`journeys/M15-whisper-transcribe.md`](journeys/M15-whisper-transcribe.md).
- **M16 — RSA + ChaCha20-Poly1305 handshake**: required only if
  Plaud ships a firmware that disables the plaintext auth path. The
  handshake is fully specified from R1 APK analysis in
  `specs/re/apk-notes/3.14.0-620/architecture.md`. DoR: we have a
  device running the new firmware OR we have a motivated user
  volunteering theirs.

---

## Changelog

| Date | Milestone | Change |
|---|---|---|
| 2026-04-05 | — | Roadmap created. Research phase complete; M0 ready to start. Per-milestone DoR/DoD inlined into `ROADMAP.md`; journey docs authored just-in-time per milestone. |
| 2026-04-05 | M0 | **Closed.** Workspace scaffold + 10 crate stubs + `plaude-cli` bin with `--help` / `--version` / no-args exit 2. 5 e2e tests green, `make lint` clean, CI workflow added, README + `docs/usage/index.md` published. Release binary: 993 KB, 1 ms startup. See [`journeys/M00-scaffold.md`](journeys/M00-scaffold.md) Implementation section for file-level detail. |
| 2026-04-05 | M1 | **Closed.** `plaud-domain` (7 modules, 14 public types) and `plaud-transport` (4 modules, 3 traits, 1 error enum) now implemented. `DeviceSerial` and `AuthToken` have redacting `Debug` impls enforced by "no 8+ digit/hex run" scan tests. `CommonSettingKey` covers all 20 tinnotech SDK variants with round-tripping `code()`/`from_code()`. All 3 boundary traits are dyn-compatible. 95 tests total green, dep-graph check confirms `plaud-domain` depends only on `thiserror` + `zeroize`. See [`journeys/M01-domain-traits.md`](journeys/M01-domain-traits.md) Implementation section. |
| 2026-04-05 | M2 | **Closed.** `plaud-proto` codec: `Frame` enum (`Control`, `Bulk`, `BulkEnd`, `Handshake`), `parse_notification` with magic-byte demux and `0xFE` handshake detection, `auth_response` for `AuthStatus`, `encode::{control, nullary, auth::authenticate, file::read_file_chunk, device::*}` covering every opcode we have a wire example for, 16 opcode constants, all protocol magic bytes/lengths as named `const`s. `read_file_chunk` matches the 0day capture byte-for-byte; `authenticate` matches the V0095 wire layout. 34 new tests (9 files) including `proptest!` round-trip. 129 tests total green. Dep-graph audit: `plaud-domain` + `bytes` + `thiserror`, nothing else. See [`journeys/M02-proto-codec.md`](journeys/M02-proto-codec.md) Implementation section. |
| 2026-04-05 | M3 | **Closed.** `plaud-sim` in-process simulator: `SimDevice` + `SimDeviceBuilder`, `SimTransport` implementing all 13 `Transport` trait methods, `SimDiscovery` with the full scan + connect + auth flow (wrong token → `AuthRejected { status: 0x01 }`, soft-reject semantics verified). Battery reads work without auth (SIG analogue). Failure injection via `inject_disconnect_after` / `inject_delay`. `plaud_sim::bulk::frames_for` + `serialise_bulk` round-trip through `plaud_proto::parse_notification`. Determinism pinned by test. 11 integration test files, 41 new tests; 170 tests total green. Added `DeviceInfo::placeholder` / `StorageStats::ZERO` / friends to `plaud-domain` so the sim can build infallible defaults without `unwrap`/`expect` in production code. See [`journeys/M03-sim-v0.md`](journeys/M03-sim-v0.md) Implementation section. |
| 2026-04-06 | M4 | **Closed.** `plaud-auth` storage layer: `FileStore` (0600 file + 0700 parent on Unix), `KeyringStore` (wraps the `keyring` crate via `spawn_blocking`), `ChainStore` (keyring-primary + file-fallback with primary-first get, put-retry-on-error, either-backend-wins remove), `token_fingerprint` (SHA-256 [:16]), and a pure-Rust `btsnoop` parser that walks btsnoop v1 logs without any `tshark` dependency and extracts the V0095 auth token from the first ATT Write Command to handle `0x000D`. Plus `plaude-cli` gains its first user-visible subcommand tree: `plaude-cli auth set-token / import / show / clear` with a global `--config-dir` sandbox override for tests. 5 integration test files + 28 new tests; 198 tests total green. Live integration verified: `target/release/plaude-cli auth show` against the pre-seeded research-phase `~/.config/plaude/token` returns the `a82dcb11ff56d11d` fingerprint documented in evidence. See [`journeys/M04-auth-storage.md`](journeys/M04-auth-storage.md) Implementation section. |
| 2026-04-06 | M10 | **Closed.** `plaud-transport-usb` crate ships as a full `Transport` impl over a mounted PLAUD_NOTE VFAT volume. `model_txt::parse` extracts `DeviceInfo` from the 2-line MODEL.txt; `listing::list_recordings` walks `NOTES/` + `CALLS/` and pairs `.WAV` + `.ASR` files by basename; `UsbTransport::read_recording`/`read_recording_asr` return the raw file bytes; every non-applicable method (`battery`, `set_privacy`, recording control, etc.) returns `Unsupported`. `WavSanitiser::sanitise` zeros the `SN:<serial>\0` region (bytes `0x2C..0x42`) in-place without shifting audio offsets; validates RIFF/WAVE magic before touching any byte; idempotent. CLI gains `Backend::Usb` + global `--mount <PATH>` flag; `UsbProvider` wraps the transport; `--sanitise` flag on `sync` applies the scrubber to every WAV pulled. Deprecation notice on stderr for every USB run. 5 new source modules + 18 unit tests + 7 integration tests; 302 tests total green. Docs at `docs/usage/usb.md`. **Scope-reduction**: auto-discovery of the VFAT mount path (block-device enumeration + label matching) is deferred; M10 requires explicit `--mount`. See [`journeys/M10-transport-usb.md`](journeys/M10-transport-usb.md). |
| 2026-04-06 | M9 | **Closed.** `plaude-cli sync <dir>` ships as the headline mirroring command. State lives in a versioned JSON file `<dir>/.plaude-sync.json` with a SHA-256 `inventory_hash` over sorted `(id, wav_size, asr_size)` triples plus per-recording `{wav_size, asr_size, pulled_at_unix_seconds}` entries. Atomic state writes via `.tmp` + rename. Idempotent: re-runs with no device changes exit 0 with `nothing to do`. Incremental: only new recordings are pulled. `--dry-run` prints a `would pull: <id>` plan without touching disk or state. Deleted-on-device entries are flagged in stderr and their state-file rows are removed, but the local files are preserved (safer default). `--concurrency N` flag accepted for forward compatibility; BLE is serial so the value is not consulted yet. File-grained resume: interrupted runs leave successfully-pulled files on disk and the next run picks up the stragglers. `SimProvider` gains a `PLAUDE_SIM_RECORDINGS` env var (comma-separated basename list) that lets e2e tests vary the device fixture between runs within a single test without a second process. New sync module (`commands/sync/mod.rs` + `state.rs`) with 5 unit tests covering inventory-hash stability under reordering, hash sensitivity to size and count changes, JSON round-trip, and schema versioning; 7 new e2e tests covering empty device / one file / no-op re-run / incremental add / dry-run / deleted-on-device / missing-token → 77. 277 tests total green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented. Docs shipped at `docs/usage/sync.md`. **Scope-reductions**: real `SIGINT` handler → exit-130 integration and `indicatif` per-file progress bars are deferred to M12 hardening alongside byte-grained (mid-file) resume — all three require either a trait extension or tokio-signal integration best addressed together. The file-grained resume semantic plus the JSON state file already provide safe interruption at file boundaries. See [`journeys/M09-sync.md`](journeys/M09-sync.md) Implementation section. |
| 2026-04-06 | M8 | **Closed (sim path).** The one-time onboarding command `plaude-cli auth bootstrap` ships. `plaud-proto` gains `decode::parse_auth_write` (inverse of `encode::auth::authenticate`) so the peripheral can extract an `AuthToken` from a captured phone write; two new `DecodeError` variants (`InvalidAuthPrefix`, `InvalidAuthToken`) cover the prefix-mismatch and validator-rejected paths. `plaud-transport-ble` gains a `bootstrap` submodule: `BootstrapChannel` + `PhoneChannel` (peripheral-side mpsc pair), `BootstrapSession::run(timeout)` driving the three-step handshake (receive → decode → send mock `AUTH_STATUS_ACCEPTED`), `BootstrapOutcome { token }`, `BootstrapError { Timeout, PhoneDisconnected, DecodeFailed }`, `BOOTSTRAP_DEFAULT_TIMEOUT = 120s`, and a hermetic `LoopbackBootstrap` + `TestPhone` pair. The CLI wires `auth bootstrap --timeout <SECS>` through the existing `--backend sim\|ble` flag; `sim` spawns the loopback + a fake phone writing a deterministic 32-hex token, captures the outcome, stores via the existing `AuthStore` chain, prints the fingerprint, exits 0; `ble` returns the stable `ble-hardware-backend` runtime marker. Idempotent: running the sim path twice in a row both succeed (both overwrite the same token). 3 new test files + 13 new tests (5 proto decode + 4 bootstrap session + 4 e2e CLI); 264 tests total green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented. Docs shipped at `docs/usage/auth-bootstrap.md` with explicit "put your real device to sleep first" precondition for the future real-hardware path. **Scope-reductions**: real BlueZ GATT advertisement + server (lands alongside the btleplug central — both share D-Bus plumbing so we wire them in one go), opportunistic sidechannel capture of `sendHttpToken` / `sendFindMyToken` / `setSoundPlusToken` (M14), and the "second OS process as phone" e2e flow (hermetic loopback IS the second process, in-runtime, byte-equal to the real protocol). Real-hardware crate selected: `bluer` (bluez-rs, actively maintained, the only mature peripheral-mode option on Linux). See [`journeys/M08-auth-bootstrap-peripheral.md`](journeys/M08-auth-bootstrap-peripheral.md) Implementation section. |
| 2026-04-06 | M7 | **Closed.** `plaude-cli files list` and `plaude-cli files pull-one` ship as the first recording-data journey. The `Transport` trait gains a symmetric `read_recording_asr(id) -> Result<Vec<u8>>` method so the CLI can fetch the mono Opus sidecar through the boundary instead of reaching into `plaud-sim`-private accessors; real impl on `SimTransport`, `Unsupported { capability: "read_recording_asr (lands in M7)" }` stub on `BleTransport` pinned by the extended `transport_unsupported.rs` matrix. `SimProvider` in the CLI backend now preloads one deterministic recording (`1775393534` / `WAV-BYTES-FROM-SIM` / `ASR-BYTES-FROM-SIM`) so every e2e test asserts against stable bytes. `files list` supports `--output text\|json` with a serde-derived `RecordingJson` schema. `files pull-one <id>` supports `-o DIR` (directory created on demand) and `--resume` with idempotent-skip semantics: if the paired `<id>.wav` + `<id>.asr` are already present at the expected sizes the command is a no-op; otherwise partial files are rewritten from scratch. Progress bar via `indicatif` 0.17 (auto-hidden on non-TTY stderr so CI stays clean). Unknown id → runtime error, no partial files left behind. 4 new test files + 13 new tests + unit tests in `commands/files.rs`; 251 tests total green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented. Docs shipped at `docs/usage/files.md`. **Scope-reductions** documented in the journey: mid-offset resume, streaming writes without in-memory buffer, device CRC validation via `tnt_get_file_crc`, and the hardware-gated smoke test are deferred — all four need plumbing that either doesn't exist yet (range-reads, CRC opcode) or depends on the btleplug backend. See [`journeys/M07-files-list-pull.md`](journeys/M07-files-list-pull.md) Implementation section. |
| 2026-04-06 | M6 | **Closed.** `plaude-cli battery` and `plaude-cli device info` ship as the first user-visible commands driving a full transport stack. New global `--backend sim\|ble` flag selects a `TransportProvider`; `SimProvider` wraps `plaud_sim::SimDevice`, `BleProvider` is a clearly-marked stub returning `Error::Unsupported { capability: "ble-hardware-backend" }` until the btleplug wire-up milestone. Both commands support `--output text\|json` with serde-derived JSON schemas pinned by unit tests. Auth failure paths map to distinct `sysexits(3)` codes: `77 (EX_NOPERM)` when no token is stored, `78 (EX_CONFIG)` when the device rejects the stored token. `DispatchError::from_transport_error` is the single mapping point. `PLAUDE_SIM_REJECT=1` env hook drives the rejected-token test deterministically. 2 new e2e test files + 9 tests + unit tests in each command module; 238 tests total green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented. Docs shipped at `docs/usage/battery.md` and `docs/usage/device-info.md`; `docs/usage/index.md` updated with the two new exit codes. `BleTransport::device_info`/`storage` remain `Unsupported` stubs — they land alongside the btleplug backend and will plug transparently into the existing `TransportProvider` abstraction. See [`journeys/M06-battery-device-info.md`](journeys/M06-battery-device-info.md) Implementation section. |
| 2026-04-06 | M5 | **Closed.** `plaud-transport-ble` session + transport + discovery, all hermetic. `BleSession` owns a `BleChannel` (`tokio::mpsc` pair) and drives the auth flow (writes exact `plaud_proto::encode::auth::authenticate` bytes, awaits the auth notification within 5 s, flips `authenticated` on `AuthStatus::Accepted`, surfaces `AuthRejected { status: 0x01 }` on rejection and `Error::Protocol` on malformed responses, reserves `Frame::Handshake` for future `0xFE11`/`0xFE12` via `Error::Unsupported { capability: "rsa-chacha20-handshake" }`). `send_control` rejects pre-auth calls with `Error::AuthRequired`, correlates by opcode, and surfaces `Error::Protocol` on mismatch. `BulkReassembler` validates monotone offsets and `file_id` consistency, errors on `finish` without a terminator; `read_bulk` drives it end-to-end over loopback. `BleTransport` implements the full `Transport` surface: `battery()` is real and delegates to an injected `BatteryReader` **without touching session auth state** (matching Test 2b evidence); every other vendor-opcode method returns `Error::Unsupported { capability }` with stable strings pointing at M6/M7/M11 and pinned by `tests/transport_unsupported.rs`. `BleDiscovery` delegates scan to an injectable `ScanProvider`; `connect` is deferred to the hardware-driven milestone. 6 integration test files + 24 new tests; 222 tests total green, clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented. See [`journeys/M05-transport-ble.md`](journeys/M05-transport-ble.md) Implementation section. |

| 2026-04-06 | M11 | **Closed.** `plaude-cli settings list\|get\|set`, `plaude-cli record start\|stop\|pause\|resume`, `plaude-cli device privacy on\|off`, and `plaude-cli device name` ship as the final vendor-opcode CLI surface before M12 hardening. Domain layer gains `CommonSettingKey::from_name` (name→variant round-trip), `SettingValue::parse` (CLI string→typed value with bool>u8>u32 priority), `SettingValue` `Display` impl, plus `UnknownSettingName` and `SettingValueParseError` error types — 10 new unit tests. CLI gains two new command modules (`settings.rs`, `record.rs`) and extends `device.rs` with `Privacy` (positional `on`/`off` via `OnOff` newtype wrapper to avoid clap's `bool=SetTrue` conflict) and `Name` (reads `device_info().local_name`). `SetDeviceName` write (`0x006B`) deferred — not yet observed on the wire. Sim backend preloads 3 default settings (`enable-vad=true`, `mic-gain=20`, `auto-power-off=300`) so `settings list` returns deterministic data. 3 new e2e test files (18 tests): settings list text+json + get + set + unknown-name + bad-value + no-token; record start + invalid-stop/pause/resume + no-token; device privacy on/off + bad-arg + name + no-token paths. `make lint` clean, `make test` green, every public item documented, zero unwraps/expects in production, zero `#[allow]`, zero dead code. Docs at `docs/usage/settings.md` and `docs/usage/record.md`. See [`journeys/M11-settings-record-control.md`](journeys/M11-settings-record-control.md) Implementation section. |

| 2026-04-06 | M12 | **Closed.** Hardening milestone: CLIG exit-code audit adds `EX_UNAVAILABLE (69)` for transport-layer failures (BLE not wired, timeout, connection drop) alongside the existing `EX_NOPERM (77)` and `EX_CONFIG (78)`. Exit-code table at `docs/usage/exit-codes.md` covers every command × code combination. `DispatchError::Unavailable` variant and `EXIT_UNAVAILABLE = 69` constant added; `from_transport_error` now routes `Timeout`, `Transport`, and BLE-backend `Unsupported` to exit 69 instead of the generic exit 1. Structured logging via `tracing-subscriber`: `RUST_LOG=info` activates human-readable text on stderr, `--log-format json` switches to JSON lines — subscriber initialised in `init_logging()` at startup. Global `--timeout <SECS>` flag with `PLAUDE_TIMEOUT` env var (default 30s) parsed and threaded into dispatch. `--about` flag prints the privacy disclosure (cleartext BLE, serial watermark, credential handling) and exits 0. Man pages via `make man` target using `help2man` from the compiled binary; `make install` copies them to `~/.local/share/man/man1/`. `docs/usage/troubleshooting.md` covers every known error path with resolution steps. Security review: exhaustive grep of all `.rs` source files — no credential leaks found; `AuthToken` and `DeviceSerial` have redacting Debug impls, `.as_str()`/`.reveal()` calls are all in legitimate contexts. README.md privacy notice expanded with numbered disclosure list matching `--about` output. 4 new e2e tests in `e2e_exit_codes.rs`; 334 tests total green. **Scope-reductions**: retry with exponential backoff (needs real hardware), multi-device interactive selection (needs btleplug), `CHANGELOG.md` + version bump (manual release manager steps). See [`journeys/M12-hardening.md`](journeys/M12-hardening.md) Implementation section. |

| 2026-04-06 | M15 | **Closed (stretch).** `plaude-cli transcribe` ships as a thin wrapper around a user-supplied `whisper.cpp` CLI binary for offline local transcription. `--whisper-bin <PATH>` + `PLAUDE_WHISPER_BIN` env var locates the binary (defaults to `whisper-cli` on `$PATH`). `--model <PATH>` + `PLAUDE_WHISPER_MODEL` (required) points at a GGML model file. `--language <LANG>` passes a language hint; `--output-format txt\|srt\|vtt` selects the output format. One or more WAV file paths accepted as positional args. Exit codes follow the M12 sysexits convention: 0=success, 1=runtime error (WAV not found, whisper process failure), 2=usage error (model not found, no files), 69=unavailable (whisper binary not found). Subprocess stdout is forwarded to the CLI's stdout; whisper's `--no-prints` flag suppresses non-transcript output. New `commands/transcribe.rs` module with `TranscribeArgs` + `TranscribeFormat` enum + `run()` entry point + `transcribe_one()` per-file driver. 1 unit test (format flag mapping) + 6 e2e tests with mock shell-script whisper binaries covering happy path, missing binary, missing model, missing WAV, whisper failure, and no-files-arg. 341 tests total green, `make lint` clean, zero dead code. Docs at `docs/usage/transcribe.md`. See [`journeys/M15-whisper-transcribe.md`](journeys/M15-whisper-transcribe.md) Implementation section. |

*(Append one row per closed milestone. Do not rewrite completed rows.)*
