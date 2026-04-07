# M02 — `plaud-proto` wire codec

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M2 |
| **Journey name** | "I can parse any notification the Plaud device sends and emit a wire-correct auth or vendor opcode write, purely in Rust, with no I/O" |
| **Primary actor** | Engineer implementing M3 (`plaud-sim`) and M5 (`plaud-transport-ble`) |
| **Dependencies** | M1 |
| **Blocks** | M3, M5 |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md#m2` |

## Context

M1 published the domain vocabulary. M2 puts that vocabulary onto the
wire: a pure-Rust, allocation-lean codec that translates between
`Frame` values and the raw bytes Plaud devices emit/consume on the
BLE vendor notify / write characteristics.

The codec is the single source of truth for the wire format. Every
downstream crate — the simulator, the real BLE transport, the Wi-Fi
transport once it lands — consumes this API. No ad-hoc byte twiddling
allowed outside `plaud-proto`.

Evidence inputs:

- Frame layout — `docs/protocol/ble-commands.md` §1 "Control frame" and §2 "Bulk data frame".
- 16 observed opcodes with example bytes — `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` and `.../2026-04-05-plaud-0day-pair.md`.
- V0095 auth frame byte layout — `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md` (live-tested by replay).
- Handshake type values (`0xFE11`/`0xFE12`) — `specs/re/apk-notes/3.14.0-620/architecture.md` §Mode B.
- End-of-stream sentinel `0xFFFFFFFF` — `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md`.

## Customer journey (CJM)

### Phase 1 — M5 engineer decodes an incoming BLE notification

**Action**: `let frame = plaud_proto::decode::parse_notification(bytes)?;`

**Expected**: one of `Frame::Control { opcode, payload }`,
`Frame::Bulk { file_id, offset, payload }`,
`Frame::BulkEnd { file_id, payload }`,
`Frame::Handshake { handshake_type, payload }` — demuxed solely by
inspecting the first one or two bytes. Unknown frame types surface
as a typed `DecodeError::UnknownFrameType`, never as a panic, never
as an opaque string.

### Phase 2 — M5 engineer writes an auth frame

**Action**: `let bytes = plaud_proto::encode::auth::authenticate(&stored_token);`

**Expected**: the byte-for-byte sequence the tinnotech SDK's
`C9555a0.mo35317b()` produces for the V0095 firmware path — `01 01 00 02 00 00`
followed by the token's ASCII bytes. No intermediate allocations
beyond the returned `Bytes`.

### Phase 3 — M5 engineer reads an auth response

**Action**: `let status = plaud_proto::decode::auth_response(&frame)?;`

**Expected**: `AuthStatus::Accepted` for response status byte `0x00`,
`AuthStatus::Rejected` for `0x01`, typed error for any other shape.

### Phase 4 — M3 engineer replays captured bytes through the sim

**Action**: loads one of the binary fixtures under
`crates/plaud-proto/tests/fixtures/` and feeds it through
`parse_notification`, then round-trips through the matching encoder.

**Expected**: **exact byte-equality** between the fixture and the
re-encoded output for every captured control frame and every
constructible bulk frame.

## Engineer journey (micro-TDD)

1. **Plan** — "`parse_notification` rejects empty input with
   `DecodeError::Empty`."
2. **Test-RED** — one assertion driving `parse_notification(Bytes::new()).is_err()`.
3. **Code-GREEN** — four lines of early-return.
4. Continue for every frame-type branch: control happy path, control
   too-short, bulk happy path, bulk end-of-stream sentinel, handshake
   detection, unknown-type rejection.
5. For the encoder side, each opcode builder gets:
   - one "expected bytes" example test with a hand-written literal
     sourced from a walkthrough,
   - a round-trip test that parses the encoder's output and gets
     back a matching `Frame::Control`.
6. Proptest: any valid `(opcode, payload)` tuple encoded and then
   parsed yields back the same opcode and payload — no crashes, no
   truncation.

## Scope (this milestone)

**In scope:**

- `Frame` enum with four variants: `Control`, `Bulk`, `BulkEnd`, `Handshake`.
- `AuthStatus` enum (`Accepted`, `Rejected`).
- `Opcode` constants for every one of the 16 opcodes observed on the wire.
- `DecodeError` enum with one variant per distinct failure mode.
- `decode::parse_notification(Bytes) -> Result<Frame>` — the magic-byte demux.
- `decode::auth_response(&Frame) -> Result<AuthStatus>`.
- `encode::control(opcode, payload) -> Bytes` — low-level builder shared by every opcode wrapper.
- `encode::auth::authenticate(&AuthToken) -> Bytes` — the critical V0095-compatible auth encoder.
- Typed wrappers for **the observed opcodes** that have concrete evidence:
  `get_device_name`, `get_state`, `get_storage_stats`, `read_file_chunk`,
  `set_privacy`, `close_session`, a handful of nullary probes.
- Binary fixtures under `tests/fixtures/` with a `README.md` citing
  the btsnoop walkthrough entry each fixture came from.
- Round-trip property test + example tests per fixture.

**Out of scope (deferred to M11):**

- Typed wrappers for the ~30 SDK opcodes we have never exercised on the
  wire. They get a numeric constant in `opcode.rs` but no `encode::*`
  helper until M11 probes their semantics.
- Variable-length `packInt` (the tinnotech native helper's exact
  encoding rules). We empirically match the V0095 auth wire format
  with a fixed prefix; full reimplementation is a stretch for M16
  when the RSA/ChaCha20 path becomes relevant.
- Bulk frame **encoding** — the device sends bulk, the CLI only
  decodes. A `decode::parse_notification` path exists for
  `Frame::Bulk` and `Frame::BulkEnd`; there is no `encode::bulk`.

## Test plan

| Test file | Focus | Coverage |
|---|---|---|
| `tests/control_decode.rs` | `parse_notification` for control frames, short/empty/unknown-type error paths | `decode::parse_notification` happy + error |
| `tests/control_encode.rs` | `encode::control` byte-for-byte against a fixture; nullary opcode wrappers | `encode::control` + wrappers |
| `tests/auth_encode.rs` | `encode::auth::authenticate` produces the exact V0095 layout | `encode::auth` |
| `tests/auth_decode.rs` | `decode::auth_response` for both status bytes + error cases | `decode::auth_response` |
| `tests/bulk_decode.rs` | `parse_notification` for bulk happy path + end-of-stream sentinel | `decode` bulk branch |
| `tests/handshake_decode.rs` | `parse_notification` detects `0xFE11`/`0xFE12` preambles | `decode` handshake branch |
| `tests/file_encode.rs` | `encode::file::read_chunk` byte layout matches the 0day capture wire bytes | `encode::file` |
| `tests/device_encode.rs` | `encode::device::get_device_name`, `set_privacy` wire layout | `encode::device` |
| `tests/roundtrip.rs` | `proptest!` — any `(opcode, payload)` encoded then decoded round-trips | property test |

Target coverage for `plaud-proto`: ≥ 90 %.

## Definition of Ready

- [x] M1 closed (`plaud-domain::AuthToken` available)
- [x] `docs/protocol/ble-commands.md` §§ 1–2 are the authoritative wire spec
- [x] At least one btsnoop walkthrough entry exists per opcode we will encode

## Definition of Done

Mirror of the M2 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.

## Implementation (closed 2026-04-05)

### Files created — `plaud-proto` source

- [`crates/plaud-proto/Cargo.toml`](../../../crates/plaud-proto/Cargo.toml) — deps: `plaud-domain`, `bytes`, `thiserror`; dev-dep `proptest`.
- [`crates/plaud-proto/src/lib.rs`](../../../crates/plaud-proto/src/lib.rs) — module tree + re-exports.
- [`crates/plaud-proto/src/constants.rs`](../../../crates/plaud-proto/src/constants.rs) — every wire-format magic byte, length, offset, and status code as a named `const`. Zero bare literals in the rest of the crate reference these.
- [`crates/plaud-proto/src/opcode.rs`](../../../crates/plaud-proto/src/opcode.rs) — 16 opcode constants, one per opcode observed on the wire, each citing its source walkthrough in its doc comment.
- [`crates/plaud-proto/src/frame.rs`](../../../crates/plaud-proto/src/frame.rs) — `Frame` enum (`Control`, `Bulk`, `BulkEnd`, `Handshake`) and `AuthStatus` enum (`Accepted`, `Rejected`).
- [`crates/plaud-proto/src/error.rs`](../../../crates/plaud-proto/src/error.rs) — `DecodeError` enum (`Empty`, `UnknownFrameType`, `TooShort`, `NotAuthResponse`, `UnknownAuthStatus`).
- [`crates/plaud-proto/src/decode.rs`](../../../crates/plaud-proto/src/decode.rs) — `parse_notification(Bytes) -> Result<Frame>` with magic-byte demux (including `0xFE` detection for handshake preambles), plus `auth_response(&Frame) -> Result<AuthStatus>`.
- [`crates/plaud-proto/src/encode/mod.rs`](../../../crates/plaud-proto/src/encode/mod.rs) — low-level `control(opcode, payload)` builder + `nullary(opcode)` helper.
- [`crates/plaud-proto/src/encode/auth.rs`](../../../crates/plaud-proto/src/encode/auth.rs) — `authenticate(&AuthToken)` producing the V0095-compatible wire layout.
- [`crates/plaud-proto/src/encode/file.rs`](../../../crates/plaud-proto/src/encode/file.rs) — `read_file_chunk(file_id, offset, length)` producing the exact wire bytes from the 0day session-C capture.
- [`crates/plaud-proto/src/encode/device.rs`](../../../crates/plaud-proto/src/encode/device.rs) — `get_device_name`, `get_state`, `get_storage_stats`, `set_privacy(bool)`, `close_session`.

### Files created — tests (9 files)

Every file carries a `// Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md` header comment.

- [`control_decode.rs`](../../../crates/plaud-proto/tests/control_decode.rs) — 5 tests: empty, too-short, unknown type, nullary parse, payload preservation.
- [`control_encode.rs`](../../../crates/plaud-proto/tests/control_encode.rs) — 3 tests: byte layout, nullary, round-trip.
- [`auth_encode.rs`](../../../crates/plaud-proto/tests/auth_encode.rs) — 3 tests: 32-char V0095 layout, 16-char legacy layout, header pin.
- [`auth_decode.rs`](../../../crates/plaud-proto/tests/auth_decode.rs) — 6 tests: accepted, rejected, unknown status, empty payload, non-auth opcode, non-control frame.
- [`bulk_decode.rs`](../../../crates/plaud-proto/tests/bulk_decode.rs) — 5 tests: data frame, offset 80, end-of-stream sentinel, too-short, zero-byte payload.
- [`handshake_decode.rs`](../../../crates/plaud-proto/tests/handshake_decode.rs) — 3 tests: `0xFE12`, `0xFE11`, single-byte non-handshake rejection.
- [`file_encode.rs`](../../../crates/plaud-proto/tests/file_encode.rs) — 2 tests: byte-for-byte match to the 0day session-C capture wire bytes, round-trip.
- [`device_encode.rs`](../../../crates/plaud-proto/tests/device_encode.rs) — 6 tests: each device-level encoder's exact wire bytes.
- [`roundtrip.rs`](../../../crates/plaud-proto/tests/roundtrip.rs) — `proptest!` round-trip for any `(u16, Vec<u8>)` through `encode::control` → `parse_notification`.
- [`tests/fixtures/README.md`](../../../crates/plaud-proto/tests/fixtures/README.md) — provenance table linking every fixture pattern back to its source btsnoop walkthrough.

### Files modified

- [`Cargo.toml`](../../../Cargo.toml) (workspace root) — added `bytes = "1.8"` to `[workspace.dependencies]`.

### Dependency discipline

```
plaud-proto v0.1.0
├── bytes v1.11.1
├── plaud-domain v0.1.0
└── thiserror v2.0.18
```

Exactly the three crates the M2 DoD allows: `plaud-domain`, `bytes`,
`thiserror`. No `tokio`, no transport crates, no `async-trait`.
The codec is pure and synchronous.

### Results

- **`make lint`** — clippy `-D warnings` clean on all targets, rustfmt clean, audit swallowed by the pre-existing Makefile `|| true`.
- **`make test`** — **129 tests total, 0 failed** (95 from M0/M1 + 34 new from M2 across 9 test files).
- **`make build`** — release binary rebuild incremental finish 0.01 s, workspace still builds clean.

### Cross-cutting concerns

- [x] Every public item has a doc comment with an evidence citation where applicable.
- [x] No bare literals in business logic — every magic byte, opcode, and length is a named `const` in [`constants.rs`](../../../crates/plaud-proto/src/constants.rs) or [`opcode.rs`](../../../crates/plaud-proto/src/opcode.rs).
- [x] No `#[allow(...)]` attributes and no `clippy.toml` changes.
- [x] No `unwrap()` / `expect()` in production code paths (verified in `src/`).
- [x] No `TODO` / `FIXME` / `unimplemented!()`.
- [x] Decoder is zero-copy via `Bytes::slice` for all payload fields; the only heap allocation is the `BytesMut` in the encoder and the incoming `Bytes` buffer the caller owns.

### Traceability

- [x] This journey has an Implementation section (you're reading it).
- [x] [`ROADMAP.md`](../ROADMAP.md) M2 row is flipped to ✅ with links to every key file.
- [x] Every test file header begins with `// Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md`.
