# M06 — `plaude battery` + `plaude device info`

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M6 |
| **Journey name** | "I can run `plaude battery` and see a percentage, and run `plaude device info` and see a formatted summary. Both commands pick up a stored token, surface a clear missing-token error, and surface a clear rejected-token error, all exercised against `plaud-sim` in CI." |
| **Primary actor** | CLI end-user (the only human milestone so far with a visible TUI output besides `auth`). |
| **Dependencies** | M4 (auth storage), M5 (BLE transport skeleton), M3 (sim backend). |
| **Blocks** | M7 (files), M11 (settings/record), and every later CLI surface that needs a `TransportProvider`. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M6 row. |

## Context

M5 shipped the hermetic protocol layer but left every vendor-opcode
method on `BleTransport` as `Error::Unsupported`. M3 shipped `plaud-sim`
with full, deterministic implementations of every `Transport` method.
**M6 is the first milestone that lets a user type `plaude <cmd>` and
see live device data.**

Because no real-hardware BLE backend exists yet (btleplug wire-up is
deferred to a later milestone), M6's runtime backend is `plaud-sim`:
the CLI now accepts a global `--backend sim|ble` flag. `sim` talks to
an in-process deterministic `SimDevice`; `ble` is reserved for the
future real-hardware backend and currently returns a clear
`Error::Unsupported { capability: "ble-hardware-backend" }` pointing
at that future milestone. Every CLI integration test in M6 runs with
`--backend sim`.

The value M6 ships is therefore:

1. A stable **CLI command surface** (`battery`, `device info`) with
   stable `--output json|text` semantics, pinned by e2e tests.
2. A reusable **`TransportProvider`** abstraction that later
   milestones will plug new backends into.
3. A stable **error-to-exit-code mapping** for auth failures: missing
   token → `77 EX_NOPERM`, rejected token → `78 EX_CONFIG`. Distinct
   codes so wrapper scripts can branch on them.
4. **Documentation pages** under `docs/usage/` for both commands.

## Customer journey (CJM)

### Phase 1 — "What's my device battery?"

**Action**: `plaude-cli --backend sim battery`

**Expected**: exit 0, stdout prints a single line like `Battery: 100%`.
No token required. Runs against the sim's default `BatteryLevel`.

### Phase 2 — "What device am I talking to?"

**Action**: set a token with `plaude-cli auth set-token <hex>`, then
`plaude-cli --backend sim device info`.

**Expected**: exit 0, stdout prints a multi-line formatted summary
including local name, model, firmware, serial fingerprint, and storage
stats. The serial is printed via its redacting `Debug` impl so no raw
device serial leaks to the terminal.

### Phase 3 — "I forgot to set a token"

**Action**: clear the token, then `plaude-cli --backend sim device info`.

**Expected**: exit 77 (EX_NOPERM), stderr contains a message pointing
the user at `plaude-cli auth --help`. Stdout is empty.

### Phase 4 — "My token was rejected"

**Action**: set the sim into soft-reject mode (via
`PLAUDE_SIM_REJECT=1`), run `plaude-cli --backend sim device info`
with a token stored.

**Expected**: exit 78 (EX_CONFIG), stderr contains a message pointing
the user at `plaude-cli auth bootstrap` / re-import, and naming the
status byte. Stdout is empty.

### Phase 5 — "I want JSON"

**Action**: `plaude-cli --backend sim battery --output json` and
`plaude-cli --backend sim device info --output json`.

**Expected**: stdout is a single JSON object per command, stable
schema, no trailing text. Suitable for `jq` piping.

## Scope

**In scope:**

- Two new subcommands under `plaude-cli`: `battery` and `device info`.
  Both accept `--output <text|json>` (default `text`).
- A global `--backend <sim|ble>` flag with default `ble`. The `ble`
  backend returns `Error::Unsupported { capability: "ble-hardware-backend" }`
  at connect-time. Tests always pass `--backend sim`.
- A new internal `TransportProvider` trait with two impls:
  - `SimTransportProvider` — wraps a `plaud_sim::SimDevice`.
  - `BleTransportProvider` — stub that errors until the btleplug
    backend ships.
- Env-var hooks on the sim provider for test determinism:
  - `PLAUDE_SIM_REJECT=1` drives the sim into `AuthRejected`.
  - Every other sim parameter uses `SimDeviceBuilder::default()`.
- Error-to-exit-code mapping extended with two new codes:
  - `EXIT_AUTH_REQUIRED = 77` — missing token.
  - `EXIT_AUTH_REJECTED = 78` — device rejected the token.
- Two docs pages: `docs/usage/battery.md`, `docs/usage/device-info.md`.

**Out of scope (deferred):**

- Real btleplug backend. A later milestone turns `--backend ble` from
  a stub into a real GATT central.
- Replacing `BleTransport::device_info` / `BleTransport::storage`
  with real `send_control`-driven impls. Those land alongside the
  btleplug backend so they have a real channel to drive.
- Multi-device routing / `--device` flag. One token per CLI for now.
- Auto-reconnect, retry/backoff — M12 hardening.
- Full `sysexits(3)` exit-code table — M12 hardening.

## Test plan

Target coverage: ≥ 90 % on new code. All tests hermetic
(`plaud-sim` only, no hardware).

| Path | Focus | Proves |
|---|---|---|
| `tests/e2e_battery.rs` | `plaude-cli battery` | text output, json output, exit 0 |
| `tests/e2e_device_info.rs` | `plaude-cli device info` | text output includes model + firmware + serial fingerprint, json output schema, missing-token → exit 77, rejected-token → exit 78 |
| unit tests in `commands/backend.rs` | `Backend::from_flag` parsing; `BleTransportProvider::connect_*` returns `Unsupported` | flag parse round-trip, ble-backend stub contract |
| unit tests in `commands/device.rs` | output formatters | text + JSON schema pinned |

Mutation targets: exit code changes for auth paths, JSON key renames,
missing-field omissions in text output.

## Definition of Ready

- [x] M3 closed (sim has `DeviceInfo`, `StorageStats`, `BatteryLevel`)
- [x] M4 closed (auth store available to the CLI process)
- [x] M5 closed (`BleTransport` exists as a stable surface; M6 does
      not touch its internals)

## Definition of Done

Mirror of the M6 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.

## Implementation (closed 2026-04-06)

### Sources (new)

- [`crates/plaude-cli/src/commands/backend.rs`](../../../crates/plaude-cli/src/commands/backend.rs) — `Backend` enum, `TransportProvider` trait, `SimProvider` (wraps `SimDevice`), `BleProvider` (stub returning `Error::Unsupported { capability: CAP_BLE_HARDWARE_BACKEND }`), `ENV_SIM_REJECT` env hook for the rejected-token test path.
- [`crates/plaude-cli/src/commands/battery.rs`](../../../crates/plaude-cli/src/commands/battery.rs) — `BatteryCommand` (clap args), `run`, `print_battery` (text + JSON), `BatteryJson` schema struct with `#[derive(Serialize)]`.
- [`crates/plaude-cli/src/commands/device.rs`](../../../crates/plaude-cli/src/commands/device.rs) — `DeviceCommand::Info`, `DeviceInfoArgs`, `run`, `load_token`, `fetch`, `print_summary`, `DeviceInfoJson` + `StorageJson` schema structs.
- [`crates/plaude-cli/src/commands/output.rs`](../../../crates/plaude-cli/src/commands/output.rs) — shared `OutputFormat { Text, Json }` enum.

### Sources (modified)

- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — two new `DispatchError` variants (`AuthRequired`, `AuthRejected { status }`), two new exit codes (`EXIT_AUTH_REQUIRED = 77`, `EXIT_AUTH_REJECTED = 78`), global `--backend` flag, dispatch arms for `Battery` and `Device`, `DispatchError::from_transport_error` central mapping.
- [`crates/plaude-cli/src/commands/mod.rs`](../../../crates/plaude-cli/src/commands/mod.rs) — module tree updated.
- [`crates/plaude-cli/src/commands/auth.rs`](../../../crates/plaude-cli/src/commands/auth.rs) — `build_store` visibility widened to `pub(crate)` so `device::info` can reuse the same `AuthStore` chain as the `auth` subcommands without duplicating configuration logic.
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — new deps `plaud-sim`, `serde`, `serde_json`, `async-trait`.
- [`Cargo.toml`](../../../Cargo.toml) — workspace additions `serde` and `serde_json`.

### Tests

- [`crates/plaude-cli/tests/e2e_battery.rs`](../../../crates/plaude-cli/tests/e2e_battery.rs) — 4 tests: text output, no-token-required happy path, JSON schema, ble-backend stub exits 1 with `ble-hardware-backend` marker.
- [`crates/plaude-cli/tests/e2e_device_info.rs`](../../../crates/plaude-cli/tests/e2e_device_info.rs) — 5 tests: text output contains model + firmware + serial + storage markers, JSON schema has stable keys, missing-token → exit 77, rejected-token (via `PLAUDE_SIM_REJECT=1`) → exit 78, no-token-leak assertion on stdout + stderr.
- Unit tests in [`commands/backend.rs`](../../../crates/plaude-cli/src/commands/backend.rs) — `BleProvider` contract (both connect methods return `Unsupported`), `SimProvider::connect_anonymous` yields a battery-capable transport, `Backend::provider` factory smoke.
- Unit tests in [`commands/battery.rs`](../../../crates/plaude-cli/src/commands/battery.rs) — JSON key round-trip, text prefix pinned as a mutation-kill assertion.
- Unit tests in [`commands/device.rs`](../../../crates/plaude-cli/src/commands/device.rs) — JSON schema has `local_name`, `model`, `firmware`, `serial`, `storage.{total_bytes,used_bytes,free_bytes,recording_count}`.

### Test suite update

- [`crates/plaude-cli/tests/e2e_help.rs`](../../../crates/plaude-cli/tests/e2e_help.rs) — the `-h == --help` byte-equality test was relaxed to two independent existence-of-Usage-header assertions. Clap splits the two output modes once any flag has a multi-paragraph doc comment, which is a stable clap behaviour and not something M6 should work around.

### Docs

- [`docs/usage/battery.md`](../../../docs/usage/battery.md) — synopsis, options, examples (text + JSON), exit codes, backend note, privacy note.
- [`docs/usage/device-info.md`](../../../docs/usage/device-info.md) — synopsis, options, examples, full 77/78 exit-code explanation, privacy note.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — two commands marked ✅ with doc links, exit-code table extended.

### Quality gates (all green at close)

- `cargo test` — 238 tests pass, 0 fail across the workspace.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- Zero `unwrap`/`expect` in production code.
- Zero `#[allow]` attributes.
- Zero dead code.
- Every public item documented.

### Deferred (by design)

- Real `BleTransport::device_info` / `storage` implementations — land
  alongside the btleplug backend. The M6 `TransportProvider` + command
  layer will pick them up transparently once the `BleProvider` stub
  is replaced.
- Multi-device routing / `--device` flag — M12 hardening.
- `plaude devices scan` subcommand — lands with the btleplug backend.
