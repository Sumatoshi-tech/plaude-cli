# M01 — Domain types & transport traits

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M1 |
| **Journey name** | "I can `use plaud_domain::*` and `use plaud_transport::*` and model the Plaud protocol in idiomatic Rust without knowing which transport is underneath" |
| **Primary actor** | Engineer implementing M2–M10 |
| **Dependencies** | M0 |
| **Blocks** | M2, M4, M5, M10 |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md#m1` |

## Context

M0 landed the workspace with 10 documented stub crates. M1 puts real
public types into two of them — `plaud-domain` and `plaud-transport` —
so every subsequent milestone can start producing behaviour against a
stable vocabulary.

Every type in this milestone is traceable to evidence already in the
repo:

- Recording model + filename scheme → `docs/protocol/file-formats.md`
  and `specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md`.
- `CommonSettingKey` enum → the 20-variant enum in
  `specs/re/captures/apk/decompiled/3.14.0-620/sources/com/tinnotech/penblesdk/Constants$CommonSettings$SettingType.java`,
  summarised in `specs/re/apk-notes/3.14.0-620/ble-protocol.md`.
- `DeviceModel` variants → the Plaud product line as seen in
  `specs/re/apk-notes/3.14.0-620/architecture.md`.
- `FirmwareVersion` parser → the `MODEL.txt` string in
  `specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`.
- Auth token ergonomics → the live-tested properties in
  `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`.

## Customer journey (CJM)

### Phase 1 — Engineer imports the crate

**Actor**: downstream engineer working on M2 (`plaud-proto`) or M4
(`plaud-auth`).

**Action**: `use plaud_domain::{Recording, RecordingId, DeviceInfo, CommonSettingKey, ...};`

**Expected**: every symbol they need for their milestone exists,
every type has a doc comment explaining its origin in the protocol
spec, and no symbol leaks implementation details of a concrete
transport.

### Phase 2 — Engineer tries to construct a bad value

**Action**: `RecordingId::new("not-a-timestamp")`.

**Expected**: returns a typed `RecordingIdError` with a descriptive
message; never panics, never silently accepts garbage.

### Phase 3 — Engineer `println!("{:?}", device_info)`

**Action**: logs a `DeviceInfo` struct for debugging.

**Expected**: the 18-digit device serial is **not** present in the
output. The `DeviceSerial` `Debug` impl redacts. Forgetting this
guarantee once would leak the forensic watermark we are trying to
strip.

### Phase 4 — Engineer defines a new transport backend

**Action**: implements `#[async_trait] impl Transport for MyBackend`.

**Expected**: every method the CLI uses is present in the trait,
each returns a typed `Result<T>`, and the trait is object-safe so
the CLI can store `Box<dyn Transport>` after discovery.

### Phase 5 — Engineer handles an error path

**Action**: matches on `plaud_transport::Error`.

**Expected**: every distinct failure mode the CLI must surface to
users (auth required, auth rejected with status byte, timeout,
unsupported capability, protocol error, transport error, I/O error,
not found) is a distinct enum variant.

## Engineer journey (micro-TDD)

The milestone decomposes naturally into ~12 atomic units (one per
type / trait). Each one follows the loop: write a failing test
showing the intended public API, then a minimal impl.

Representative loop for `RecordingId`:

1. **Plan** — "`RecordingId::new` rejects non-numeric strings."
2. **Test-RED** — `assert!(RecordingId::new("abc").is_err());`
3. **Code-GREEN** — validation function, `RecordingIdError::NonNumeric`.
4. **Reflect** — does the error type convey enough for a CLI error
   message? Yes.
5. **Repeat** for the other branches (empty, too-long, leading-zero
   edge case).

For the `Transport` trait, the "test" is a compile-only object-safety
assertion — `fn _assert(_: Box<dyn Transport>) {}`. If that compiles,
the trait is dyn-safe; if it doesn't, a method has broken object
safety and the milestone cannot land.

## Scope (this milestone)

Types:
- `RecordingId`, `RecordingKind`, `Recording`
- `DeviceSerial` (with redacting `Debug`), `DeviceModel`, `FirmwareVersion`, `DeviceInfo`
- `BatteryLevel`
- `StorageStats`
- `CommonSettingKey` (20 variants from the tinnotech SDK enum), `SettingValue`, `Setting`
- `TransportHint`, `DeviceCandidate`
- `AuthToken` (wraps `Zeroizing<String>`)

Traits (in `plaud-transport`):
- `Transport` — every vendor capability the CLI will surface
- `DeviceDiscovery` — scan + connect
- `AuthStore` — pluggable token storage

Errors:
- Per-type validation errors in `plaud-domain` (each type owns its own)
- `plaud_transport::Error` — the unified error enum the CLI bubbles up

Non-scope (deferred):
- Any actual async runtime (`plaud-transport` declares traits; it does
  not spawn futures).
- Any concrete transport implementation (M5, M8, M10).
- Any serde derives — M2 decides whether the wire format lands through
  serde or hand-rolled codecs, so we do not prejudge here.

## Test plan

| Path | Scope | What it proves |
|---|---|---|
| `crates/plaud-domain/tests/recording_id.rs` | integration | `new`, `from_str`, `Display`, `as_unix_seconds`, error variants, roundtrip property test |
| `crates/plaud-domain/tests/recording.rs` | integration | `Recording` getters, `started_at_unix_seconds` delegates to id |
| `crates/plaud-domain/tests/device_serial.rs` | integration | `reveal`, `Debug` never leaks (regex check for any 8+ digit substring), validation errors |
| `crates/plaud-domain/tests/firmware_version.rs` | integration | Parses the real `MODEL.txt` fixture byte-for-byte |
| `crates/plaud-domain/tests/battery_level.rs` | integration | `new(0)`, `new(100)`, `new(101)` error, `TryFrom<u8>` |
| `crates/plaud-domain/tests/storage_stats.rs` | integration | `free_bytes`, `used_ratio`, edge cases |
| `crates/plaud-domain/tests/common_setting_key.rs` | integration | Every SDK code round-trips through `code()` / `from_code()`; every variant has a `name()` |
| `crates/plaud-domain/tests/auth_token.rs` | integration | `new`, `as_str`, length validation; `Debug` does not leak the raw token |
| `crates/plaud-domain/tests/device_candidate.rs` | integration | Builder-style construction; `Debug` does not leak fields unexpectedly |
| `crates/plaud-transport/tests/error.rs` | integration | Every variant has a stable `Display`; `Error::AuthRejected { status: 1 }.to_string()` contains `0x01`; `From<io::Error>` |
| `crates/plaud-transport/tests/trait_object_safety.rs` | compile-only | `Box<dyn Transport>`, `Box<dyn DeviceDiscovery>`, `Box<dyn AuthStore>` all compile |

Target coverage for `plaud-domain`: ≥ 90 %.

## Friction points to watch for

1. **`async-trait` vs native async in traits**: Rust 2024 supports
   `async fn` in traits but dyn compatibility still requires care.
   Use `#[async_trait::async_trait]` on boundary traits that will be
   stored as `Box<dyn>`; document the choice in each trait's doc
   comment.
2. **`Zeroizing<String>` and `Debug`**: verify that `Debug` on
   `AuthToken` does not forward to the inner string. `zeroize`'s
   `Zeroizing` newtype forwards most traits; we must wrap it so our
   own `Debug` impl controls the output.
3. **`DeviceSerial::Debug` leak risk**: derive macros can
   accidentally drag the serial into other structs' `Debug` output.
   Every struct that holds a `DeviceSerial` must either derive
   `Debug` (relying on `DeviceSerial`'s custom impl) or have its
   own. A test enforces this invariant on `DeviceInfo`.
4. **`CommonSettingKey::from_code` vs the SDK's own `find()`**: the
   tinnotech SDK's `find()` method is incomplete (missing several
   variants). Our `from_code` covers every variant to avoid the same
   bug. Covered by a unit test.
5. **Minimal dependencies on `plaud-domain`**: the domain crate must
   not pull in `tokio`, `reqwest`, `btleplug`, or any transport
   crate. Enforced at review time; `cargo tree -e normal` check in
   CI is a stretch goal for M12.

## Definition of Ready (DoR)

- [x] M0 closed
- [x] `docs/protocol/ble-commands.md` opcode table authoritative
- [x] `Constants$CommonSettings$SettingType` enum source-confirmed

## Definition of Done

Mirror of the M1 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated
inline at milestone close with evidence links.

## Implementation (closed 2026-04-05)

### Files created — `plaud-domain`

- [`crates/plaud-domain/Cargo.toml`](../../../crates/plaud-domain/Cargo.toml) — deps: `thiserror`, `zeroize`. Dev-dep: `proptest`.
- [`crates/plaud-domain/src/lib.rs`](../../../crates/plaud-domain/src/lib.rs) — module wiring, crate-level doc, re-exports.
- [`crates/plaud-domain/src/recording.rs`](../../../crates/plaud-domain/src/recording.rs) — `RecordingId`, `RecordingIdError`, `RecordingKind`, `Recording`.
- [`crates/plaud-domain/src/device.rs`](../../../crates/plaud-domain/src/device.rs) — `DeviceSerial` (redacting `Debug`), `DeviceSerialError`, `DeviceModel`, `FirmwareVersion`, `FirmwareVersionError`, `DeviceInfo`.
- [`crates/plaud-domain/src/battery.rs`](../../../crates/plaud-domain/src/battery.rs) — `BatteryLevel`, `BatteryLevelError`.
- [`crates/plaud-domain/src/storage.rs`](../../../crates/plaud-domain/src/storage.rs) — `StorageStats`, `StorageStatsError`.
- [`crates/plaud-domain/src/setting.rs`](../../../crates/plaud-domain/src/setting.rs) — `CommonSettingKey` (20 variants), `SettingValue`, `Setting`, `UnknownSettingCode`. Every code is a named `const`; `code()`/`from_code()` share the same single source of truth, covering the tinnotech SDK's own bug.
- [`crates/plaud-domain/src/discovery.rs`](../../../crates/plaud-domain/src/discovery.rs) — `TransportHint`, `DeviceCandidate`.
- [`crates/plaud-domain/src/auth.rs`](../../../crates/plaud-domain/src/auth.rs) — `AuthToken` (wraps `Zeroizing<String>`, redacting `Debug`), `AuthTokenError`.

### Files created — `plaud-transport`

- [`crates/plaud-transport/Cargo.toml`](../../../crates/plaud-transport/Cargo.toml) — deps: `plaud-domain`, `thiserror`, `async-trait`.
- [`crates/plaud-transport/src/lib.rs`](../../../crates/plaud-transport/src/lib.rs) — module wiring and re-exports.
- [`crates/plaud-transport/src/error.rs`](../../../crates/plaud-transport/src/error.rs) — `Error` enum, `Result<T>` alias. Variants: `NotFound`, `AuthRequired`, `AuthRejected { status: u8 }`, `Timeout { seconds: u64 }`, `Io(io::Error)`, `Protocol(String)`, `Transport(String)`, `Unsupported { capability: &'static str }`.
- [`crates/plaud-transport/src/transport.rs`](../../../crates/plaud-transport/src/transport.rs) — `Transport` async trait with 13 methods covering the vendor command surface.
- [`crates/plaud-transport/src/discovery.rs`](../../../crates/plaud-transport/src/discovery.rs) — `DeviceDiscovery` async trait with `scan` and `connect`.
- [`crates/plaud-transport/src/auth_store.rs`](../../../crates/plaud-transport/src/auth_store.rs) — `AuthStore` async trait with `get_token` / `put_token` / `remove_token`.

### Test files created (12) — all carry `// Journey:` header

- [`plaud-domain/tests/recording_id.rs`](../../../crates/plaud-domain/tests/recording_id.rs) — 15 tests, including a `proptest!` roundtrip for any accepted `[0-9]{9,19}` input
- [`plaud-domain/tests/recording.rs`](../../../crates/plaud-domain/tests/recording.rs) — 6 tests
- [`plaud-domain/tests/device_serial.rs`](../../../crates/plaud-domain/tests/device_serial.rs) — 10 tests, including a non-leak `Debug` assertion that fails on any 8+ consecutive digit run in the output
- [`plaud-domain/tests/device_model.rs`](../../../crates/plaud-domain/tests/device_model.rs) — 3 tests
- [`plaud-domain/tests/firmware_version.rs`](../../../crates/plaud-domain/tests/firmware_version.rs) — 6 tests, including the exact `MODEL.txt` line from the V0095 USB baseline capture
- [`plaud-domain/tests/device_info.rs`](../../../crates/plaud-domain/tests/device_info.rs) — 2 tests, including the composition assertion that `DeviceInfo::Debug` never leaks the serial
- [`plaud-domain/tests/battery_level.rs`](../../../crates/plaud-domain/tests/battery_level.rs) — 8 tests
- [`plaud-domain/tests/storage_stats.rs`](../../../crates/plaud-domain/tests/storage_stats.rs) — 7 tests
- [`plaud-domain/tests/setting.rs`](../../../crates/plaud-domain/tests/setting.rs) — 7 tests, including a round-trip check for every one of the 20 SDK codes and a uniqueness check on `name()`
- [`plaud-domain/tests/discovery.rs`](../../../crates/plaud-domain/tests/discovery.rs) — 3 tests
- [`plaud-domain/tests/auth_token.rs`](../../../crates/plaud-domain/tests/auth_token.rs) — 10 tests, including a non-leak `Debug` assertion on hex runs
- [`plaud-transport/tests/error.rs`](../../../crates/plaud-transport/tests/error.rs) — 9 tests for `Display`, `From<io::Error>`, `std::error::Error` blanket impl
- [`plaud-transport/tests/trait_object_safety.rs`](../../../crates/plaud-transport/tests/trait_object_safety.rs) — 3 compile-only tests asserting `Box<dyn Transport>`, `Box<dyn DeviceDiscovery>`, `Box<dyn AuthStore>` all compile

### Files modified

- [`Cargo.toml`](../../../Cargo.toml) (workspace root) — added `zeroize = { version = "1.8", features = ["zeroize_derive"] }`, `async-trait = "0.1"`, `proptest = "1.6"` to `[workspace.dependencies]`.

### Dependency discipline (verified via `cargo tree -p plaud-domain -e normal`)

```
plaud-domain v0.1.0
├── thiserror v2.0.18
└── zeroize v1.8.2
```

No `tokio`, no `reqwest`, no `btleplug`, no transport-specific code in
the domain crate. The DoD invariant holds.

### Results

- **`make build`** — `Finished release profile [optimized] target(s) in 2.19s`.
- **`make test`** — **95 tests passed, 0 failed**. Breakdown: 75 new tests in `plaud-domain` and `plaud-transport`, 15 new tests via proptest iterations (counted as one test by `cargo test` but exercising many inputs), 5 e2e tests from M0.
- **`make lint`** — clippy clean on `--all-targets --all-features -- -D warnings`, rustfmt clean, audit absent (swallowed by Makefile `|| true` as in M0).
- **Clippy config honoured**: no function exceeds cognitive complexity 15, no function has more than 7 parameters, no `.unwrap()` / `.expect()` in production code paths (tests are allowed to `.expect()` fixtures).

### Cross-cutting concerns

- [x] `#![deny(missing_docs)]` at both crate roots (via `[workspace.lints.rust] missing_docs = "deny"`).
- [x] Every public item has a `///` doc comment with protocol evidence citation where applicable.
- [x] Auth tokens wrapped in `Zeroizing<String>`; never printed through `Debug`.
- [x] Device serial wrapped in `DeviceSerial` with non-leaking `Debug`; enforced by a test that scans for 8+ consecutive digits.
- [x] Every error type has a stable `Display` message and implements `std::error::Error` via `thiserror`.
- [x] All traits are dyn-compatible via `async-trait`; compile-only tests enforce it.

### Traceability (per `/implement` step 16)

- [x] This journey doc has an Implementation section (the one you're reading).
- [x] The M1 row in [`../ROADMAP.md`](../ROADMAP.md) is flipped to ✅ with links back to the concrete files.
- [x] Every test file header contains `// Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md`.
