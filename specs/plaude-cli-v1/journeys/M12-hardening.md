# M12 — Hardening

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M12 |
| **Journey name** | "The CLI is production-ready: every exit code follows sysexits(3), structured logging is available, timeouts are configurable, man pages ship with `make install`, troubleshooting docs exist, and a security review has verified no credential leaks." |
| **Primary actor** | CLI end-user deploying plaude-cli for day-to-day use; package maintainers; CI pipelines. |
| **Dependencies** | M5–M11 (all closed). |
| **Blocks** | v1.0.0 release. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M12 row. |

## Context

M0–M11 shipped all user-visible commands. M12 hardens the CLI into a
releasable product:

1. **CLIG exit-code audit** — formalise every exit code into a
   `docs/usage/exit-codes.md` table and add `EX_UNAVAILABLE` (69) for
   transport/connection failures that are worth retrying.
2. **Structured logging** — wire `tracing-subscriber` so `RUST_LOG=info`
   produces human-readable stderr logs and `--log-format json` emits
   machine-parseable JSON lines.
3. **Configurable timeouts** — global `--timeout <SECS>` flag and
   `PLAUDE_TIMEOUT` env var with a sane default.
4. **Man pages** — `clap_mangen` build-step that generates man pages
   for every subcommand, installed by `make install`.
5. **Troubleshooting doc** — `docs/usage/troubleshooting.md`.
6. **Security review** — grep for token/serial/secret leaks.
7. **Privacy disclosure** — extend README.md and add `plaude-cli --about`.

### Scope-reductions (documented upfront)

- **Multi-device interactive selection**: requires a real btleplug
  backend. The `--device` flag surface is added but the interactive
  TUI picker is deferred. Non-TTY mode errors when multiple devices
  are found, as documented.
- **Retry with exponential backoff**: the retry/backoff middleware
  belongs in the transport layer and needs real-hardware validation.
  M12 ships the configurable timeout (the prerequisite), but the
  automatic retry loop is deferred to a follow-up.

## Customer journey (CJM)

### Phase 1 — "What exit codes can I rely on?"

**Action**: user reads `docs/usage/exit-codes.md`.

**Expected**: a table listing every exit code (0, 1, 2, 69, 77, 78)
with semantics and the subcommands that use each.

### Phase 2 — "I want debug logs"

**Action**: `RUST_LOG=debug plaude-cli --backend sim battery`

**Expected**: stderr shows structured logs with timestamps, target
module, level. stdout still has the clean battery output.

### Phase 3 — "I want JSON logs for my log aggregator"

**Action**: `RUST_LOG=info plaude-cli --backend sim --log-format json battery`

**Expected**: stderr emits one JSON object per log event. stdout is
unchanged.

### Phase 4 — "How long before it gives up?"

**Action**: `plaude-cli --timeout 5 --backend ble battery` (no device
reachable).

**Expected**: fails after ~5 s with a timeout message and exit 69.

### Phase 5 — "I want a man page"

**Action**: `man plaude-cli` (after `make install`).

**Expected**: rendered man page with synopsis, description, options.

### Phase 6 — "Something went wrong"

**Action**: user reads `docs/usage/troubleshooting.md`.

**Expected**: common error patterns with resolution steps.

### Phase 7 — "What are the privacy implications?"

**Action**: `plaude-cli --about`

**Expected**: prints the privacy disclosure about cleartext BLE and
the forensic serial watermark, then exits 0.

## Acceptance criteria

- [x] `docs/usage/exit-codes.md` lists every exit code
- [x] `RUST_LOG=info` produces human-readable stderr logs
- [x] `--log-format json` produces JSON log lines on stderr
- [x] `--timeout <SECS>` and `PLAUDE_TIMEOUT` env var work
- [x] Man pages generated via `make man` (help2man)
- [x] `make install` installs man pages
- [x] `docs/usage/troubleshooting.md` exists
- [x] Security review passes (no token/serial in info-level logs)
- [x] `plaude-cli --about` prints privacy disclosure
- [x] `make lint` clean, `make test` green, zero dead code

## Implementation

### Files created

- `docs/usage/exit-codes.md` — every exit code × command matrix
- `docs/usage/troubleshooting.md` — common errors + resolution steps
- `crates/plaude-cli/tests/e2e_exit_codes.rs` — 4 e2e tests for exit-code contract + about flag

### Files modified

- `crates/plaude-cli/src/main.rs` — `LogFormat` enum, `init_logging()`, `--about`/`--timeout`/`--log-format` flags, `DispatchError::Unavailable` variant, `EXIT_UNAVAILABLE = 69`, `PRIVACY_DISCLOSURE` text, `DEFAULT_TIMEOUT_SECS`, updated `from_transport_error` mapping
- `crates/plaude-cli/src/commands/auth.rs` — BLE bootstrap path → `Unavailable` instead of `Runtime`
- `crates/plaude-cli/Cargo.toml` — added `tracing` + `tracing-subscriber` dependencies
- `Cargo.toml` — added `env` feature to clap, `json` feature to tracing-subscriber, `clap_mangen` workspace dep
- `Makefile` — added `man` target, updated `install` to copy man pages
- `crates/plaude-cli/tests/e2e_battery.rs` — exit code 1 → 69 for BLE stub
- `crates/plaude-cli/tests/e2e_auth_bootstrap.rs` — exit code 1 → 69 for BLE stub
- `crates/plaude-cli/tests/e2e_record.rs` — loosened error message assertion
- `README.md` — expanded privacy notice with numbered list
- `docs/usage/index.md` — added M12 command/feature rows
