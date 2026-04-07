# M08 — `plaude-cli auth bootstrap` (fake peripheral)

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M8 |
| **Journey name** | "I can run `plaude-cli auth bootstrap`, have my Plaud phone app connect to my laptop thinking it is a Plaud Note, watch the CLI capture the auth token from the phone's first write, and end up with the token stored in the keyring ready to use." |
| **Primary actor** | First-time CLI user onboarding a new device. |
| **Dependencies** | M2 (plaud-proto auth layout), M4 (AuthStore chain), M5 (BleChannel loopback). |
| **Blocks** | Every command that needs a stored token on a fresh install (M6, M7, M9, M11). |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M8 row. |

## Context

The design goal of plaude-cli, as signed off by the user, is a
one-time onboarding command that captures the Plaud auth token
from the user's own phone app **without relying on `adb bugreport`,
`tshark`, or any third-party tooling**. The mechanic is:

1. The CLI spins up a local BLE peripheral advertising
   `PLAUD_NOTE` with Nordic manufacturer id `0x0059`.
2. The user's Plaud phone app, looking for a nearby device to pair
   with, connects to the laptop instead of the real pen.
3. The phone writes a standard V0095 auth frame
   (`01 01 00 02 00 00 <token>`) to vendor characteristic `0x2BB1`.
4. Our fake peripheral captures the write, parses the token out of
   the 6-byte prefix, sends back a mock auth-accepted notification
   so the phone does not error, stores the token via the existing
   `AuthStore` chain, tears down, and exits.

### The CI reality

The full happy path above needs a real BlueZ peripheral GATT server
plus a second process acting as the phone — and **no CI runner has
a Bluetooth adapter**. The same dilemma we faced with M5. The same
solution applies:

**M8 ships the hermetic protocol layer now, and leaves a trait seam
for a real BlueZ backend to drop in later without behavioural
change.** The protocol layer is:

- A `BootstrapPeripheral` trait with a single `run(timeout)` method
  returning a captured `AuthToken` (or a timeout error).
- `LoopbackBootstrap` — reuses M5's `BleChannel` as the "radio",
  waits for a write that decodes via `plaud_proto::parse_auth_write`,
  sends back a mock `AUTH_STATUS_ACCEPTED` notification, and yields
  the outcome. Exposes a `TestPhone` handle that integration tests
  and the CLI's `--backend sim` runtime drive.
- A `BluerBootstrap` placeholder module that selects `bluer` as the
  future D-Bus crate (satisfying the DoR "crate selected" bullet)
  but does not wire a real advertisement yet. Lands alongside the
  btleplug central in a later milestone where it can share system
  deps.

### Scope-reductions vs. literal DoD

1. **Real BlueZ advertisement + GATT server**: deferred to the same
   milestone that ships the btleplug central. M8's CLI `--backend ble`
   path returns `Error::Unsupported { capability: "ble-hardware-backend" }`
   like every other device-talking command until that milestone.
2. **Opportunistic sidechannel capture** (sendHttpToken, etc.):
   deferred to M14. The protocol layer has an extension point but
   M8 only stores the primary auth token.
3. **E2E test with a second process acting as the phone**: the
   hermetic loopback IS the "second process" inside a single
   runtime. This gives us byte-equal coverage of the handshake
   without needing two OS processes.

## Customer journey (CJM)

### Phase 1 — Sim dogfooding (what CI actually runs)

**Action**: `plaude-cli --backend sim auth bootstrap`.

**Expected**: exit 0. Stdout prints `Token captured. Fingerprint:
<16-hex>`. The stored token is the deterministic one the sim fake
phone writes. Running `plaude-cli auth show` immediately after
returns the same fingerprint.

### Phase 2 — Real hardware (future)

**Action**: `plaude-cli auth bootstrap` on a Linux box with BlueZ.

**Expected**: exit 0 after the phone completes its handshake. The
command prints the fingerprint of the real device's token.
**(Deferred — requires btleplug/bluer backend milestone.)**

### Phase 3 — Timeout

**Action**: run with `--timeout 1` against a sim that never writes.

**Expected**: exit 1 after 1 second, stderr names the timeout.

### Phase 4 — Already-bootstrapped

**Action**: run bootstrap twice in a row on the sim.

**Expected**: both runs exit 0; the second run overwrites the
token, which is fine since the captured value is the same
deterministic sim token.

## Scope

**In scope:**

- [`plaud_proto::decode::parse_auth_write`] — already landed in the
  test-first loop that opened M8; decodes a captured write into an
  `AuthToken`.
- `plaud_transport_ble::bootstrap` submodule with
  `BootstrapPeripheral` trait, `BootstrapOutcome`, `LoopbackBootstrap`
  (+ its `TestPhone` counterpart), `BOOTSTRAP_DEFAULT_TIMEOUT = 120s`.
- `plaude-cli auth bootstrap` subcommand with `--timeout` and the
  existing global `--backend sim|ble` flag.
- Sim runtime path: CLI spawns the loopback peripheral + a local
  `TestPhone` task that writes a deterministic token, awaits the
  outcome, stores via `AuthStore`, prints the fingerprint.
- BLE runtime path: `Error::Unsupported { capability: "ble-hardware-backend" }`.
- `docs/usage/auth-bootstrap.md`.

**Out of scope:**

- Real BlueZ GATT server / advertisement — btleplug-backend milestone.
- Sidechannel opcode capture (M14).
- Multi-device disambiguation; M8 captures one token per run.

## Test plan

| Path | Focus | Proves |
|---|---|---|
| `crates/plaud-proto/tests/auth_write_decode.rs` | `parse_auth_write` | round-trip with `encode::auth::authenticate`, prefix mismatch, non-hex token, too-short input |
| `crates/plaud-transport-ble/tests/bootstrap_session.rs` | `LoopbackBootstrap::run` | fake phone writes → session decodes → mock auth-accepted written back → outcome yields the exact token; no-write-before-timeout path |
| `crates/plaude-cli/tests/e2e_auth_bootstrap.rs` | CLI sim path | `--backend sim` exits 0 with a fingerprint line; the just-captured token is then visible via `auth show`; `--backend ble` is `Unsupported`; `--timeout 0` surfaces the runtime error path |

Target coverage: ≥ 90 % on new code. All tests hermetic.

## Definition of Ready

- [x] M2 closed (`AUTH_PREFIX` constant, `encode::auth::authenticate`)
- [x] M4 closed (`AuthStore` chain + `token_fingerprint`)
- [x] M5 closed (`BleChannel::loopback_pair` as the hermetic "radio")
- [x] Real-hardware backend crate selected: **`bluer`** (actively maintained by the bluez-rs org, D-Bus based, Linux-only; the only mature Rust crate for peripheral-mode BlueZ). Wired behind a future `bluer-backend` feature flag; M8 does not actually pull it in.

## Definition of Done

Mirror of the M8 DoD in `specs/plaude-cli-v1/ROADMAP.md`, adjusted
for the three scope-reductions in **Context**. Updated at close.

## Implementation (closed 2026-04-06)

### Sources (new)

- [`crates/plaud-transport-ble/src/bootstrap/mod.rs`](../../../crates/plaud-transport-ble/src/bootstrap/mod.rs) — submodule root, re-exports.
- [`crates/plaud-transport-ble/src/bootstrap/session.rs`](../../../crates/plaud-transport-ble/src/bootstrap/session.rs) — `BootstrapChannel`, `PhoneChannel`, `BootstrapSession::run`, `BootstrapOutcome`, `BootstrapError { Timeout, PhoneDisconnected, DecodeFailed }`, `MOCK_AUTH_ACCEPTED_FRAME`, `BOOTSTRAP_DEFAULT_TIMEOUT`.
- [`crates/plaud-transport-ble/src/bootstrap/loopback.rs`](../../../crates/plaud-transport-ble/src/bootstrap/loopback.rs) — `LoopbackBootstrap::new/split`, `TestPhone::write/receive_notification`, `TestPhoneError`.

### Sources (modified)

- [`crates/plaud-proto/src/decode.rs`](../../../crates/plaud-proto/src/decode.rs) — new `parse_auth_write(&[u8]) -> Result<AuthToken, DecodeError>` inverse decoder.
- [`crates/plaud-proto/src/error.rs`](../../../crates/plaud-proto/src/error.rs) — two new `DecodeError` variants: `InvalidAuthPrefix`, `InvalidAuthToken { reason }`.
- [`crates/plaud-proto/src/lib.rs`](../../../crates/plaud-proto/src/lib.rs) — re-export `parse_auth_write`.
- [`crates/plaud-transport-ble/src/lib.rs`](../../../crates/plaud-transport-ble/src/lib.rs) — new `bootstrap` module + re-exports.
- [`crates/plaude-cli/src/commands/auth.rs`](../../../crates/plaude-cli/src/commands/auth.rs) — new `AuthCommand::Bootstrap(BootstrapArgs)`, `run` signature takes `Backend`, `bootstrap` / `bootstrap_sim` / `map_bootstrap_error` handlers, `SIM_BOOTSTRAP_TOKEN`, `DEFAULT_BOOTSTRAP_TIMEOUT_SECS`, `CAP_BLE_BOOTSTRAP_BACKEND`.
- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — pass `cli.backend` into `commands::auth::run`.
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — new deps `plaud-transport-ble`, `plaud-proto`.

### Tests

- [`crates/plaud-proto/tests/auth_write_decode.rs`](../../../crates/plaud-proto/tests/auth_write_decode.rs) — 5 tests: round-trip long + short tokens, prefix mismatch, too-short input, non-hex token.
- [`crates/plaud-transport-ble/tests/bootstrap_session.rs`](../../../crates/plaud-transport-ble/tests/bootstrap_session.rs) — 4 tests: happy capture, mock-accepted notification echo, timeout, decode failure.
- [`crates/plaude-cli/tests/e2e_auth_bootstrap.rs`](../../../crates/plaude-cli/tests/e2e_auth_bootstrap.rs) — 4 tests: sim fingerprint print, sim→auth show round-trip, ble-backend stub, idempotent double-run.

### Docs

- [`docs/usage/auth-bootstrap.md`](../../../docs/usage/auth-bootstrap.md) — onboarding flow, synopsis, options, backend note, preconditions, exit codes, privacy note.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — ✅ row for the sim path.

### Deferred

- **Real BlueZ GATT advertisement + server.** The `bluer` crate is selected; wiring lands alongside the btleplug central so both share D-Bus plumbing.
- **Sidechannel opcode capture** (`sendHttpToken`, `sendFindMyToken`, `setSoundPlusToken`). M14.
- **Two-process e2e test.** The hermetic `LoopbackBootstrap` IS the "second process" inside a single runtime and gives byte-equal coverage of the protocol handshake.

### Quality gates

- `cargo test` — 264 tests pass, 0 fail across the workspace.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- Zero `unwrap`/`expect` in production code.
- Zero `#[allow]` attributes.
- Zero dead code.
- Every public item documented.
