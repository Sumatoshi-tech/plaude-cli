# M05 — `plaud-transport-ble` (btleplug + protocol session)

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M5 |
| **Journey name** | "I can instantiate a `BleTransport`, authenticate, read a vendor opcode response, and read a bulk-stream reassembly — all against a fake in-memory device in CI, and against real hardware behind a feature flag" |
| **Primary actor** | Engineer implementing M6 (battery + device info) and M7 (file pull) |
| **Dependencies** | M2, M3, M4 |
| **Blocks** | M6, M7, M8, M9, M11 |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md#m5` |

## Context

M1 defined the `Transport` trait. M2 shipped the wire-format codec.
M3 shipped the in-process sim. M4 shipped credential storage. **M5 is
the first milestone that produces a `Transport` implementation that
can talk to real hardware** — with the critical caveat that its
**core protocol logic is tested entirely against in-memory channels**
so CI remains hermetic.

The split M5 lands is:

1. **`BleSession`** — the protocol state machine. Owns a pair of
   `tokio::mpsc` channels (outbound writes, inbound notifications).
   Implements the auth flow, request/response correlation, bulk
   reassembly, and all the `plaud_proto` integration. **100 % of
   M5's tests hit this layer**; nothing in the test suite requires
   BLE hardware.
2. **`BleTransport`** — wraps a `BleSession` + a `BatteryReader`
   trait object. Implements `plaud_transport::Transport`. In M5 only
   two methods are real: `battery()` (delegates to `BatteryReader`)
   and the infrastructure that M6+ will call for vendor opcodes.
   Every other method returns `Error::Unsupported { capability: … }`
   pointing at the milestone that will fill it in.
3. **`btleplug` backend** — thin wrapper that turns a real BLE
   adapter into the channel-based contract `BleSession` needs. Gated
   behind a `btleplug-backend` cargo feature, **on by default** but
   excludable from CI if system BlueZ headers are missing. The
   hardware smoke test is gated behind a separate `hw-tests` feature.

Evidence inputs:

- Frame layout — `docs/protocol/ble-commands.md` §§ 1–2 (we already
  tested this in M2).
- Live BLE semantics — `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`:
  auth status byte, soft-reject behaviour, battery-without-auth.
- GATT map — `docs/protocol/ble-gatt.md`: vendor service `0x1910`,
  write char `0x2BB1`, notify char `0x2BB0`, CCCD `0x0011`, battery
  char `0x2A19`.

## Customer journey (CJM)

### Phase 1 — M6 engineer wants a battery read

**Action**:

```rust
let battery_reader = Arc::new(FixedBatteryReader::new(BatteryLevel::new(87)?));
let (session, _peer) = test_session_pair();
let transport = BleTransport::from_parts(Arc::new(Mutex::new(session)), battery_reader);
let level = transport.battery().await?;
assert_eq!(level.percent(), 87);
```

**Expected**: battery succeeds without ever touching the Session's
auth state. Matches the live evidence from Test 2b.

### Phase 2 — M7 engineer drives a bulk read

**Action**: creates a `test_session_pair`, spawns a peer task that
responds to the `ReadFileChunk` opcode with a sequence of bulk frames
ending in `BulkEnd`, then calls `session.read_bulk(trigger).await`.

**Expected**: returns the reassembled bytes, validates that offsets
were monotone, fails cleanly if a frame arrives out of order.

### Phase 3 — CLI user against real hardware (opt-in)

**Action**: `cargo test -p plaud-transport-ble --features hw-tests`
on a Linux box with a BLE adapter and a real Plaud Note in range.

**Expected**: one smoke test that scans, finds `PLAUD_NOTE`,
connects, authenticates with the stored token, reads battery, and
disconnects. Skipped on systems without hardware.

## Scope

**In scope:**

- `BleChannel` — the `(mpsc::Sender<Bytes>, mpsc::Receiver<Bytes>)`
  pair that represents a connected BLE vendor-characteristic session.
- `BleSession` — protocol state machine with
  `authenticate(token)`, `send_control(frame, expected_opcode)`,
  `read_bulk(trigger_frame)` methods.
- `BulkReassembler` — validates monotone offsets, concatenates
  payloads, terminates on `BulkEnd`.
- `BatteryReader` trait + `FixedBatteryReader` test helper.
- `BleTransport` implementing `plaud_transport::Transport` with
  `battery` real and every vendor-opcode method returning
  `Error::Unsupported { capability: … }` (those land in M6/M7/M11).
- `BleDiscovery` implementing `plaud_transport::DeviceDiscovery`.
  Scan is delegated to an injectable `ScanProvider` trait; tests
  supply a fake.
- `test_session_pair()` — hermetic test factory that returns a
  ready-to-drive `BleSession` plus a `TestPeer` handle for the
  other end.
- `btleplug-backend` cargo feature with a minimal `BtleplugBackend`
  that exposes the pieces `BleTransport` needs. Compiles on
  ubuntu-latest without extra system packages; hardware tests are
  gated separately.

**Out of scope (deferred):**

- Real vendor-opcode method implementations on `BleTransport`
  (`device_info`, `list_recordings`, `read_recording`, settings,
  record control, `set_privacy`). These land in M6, M7, M11.
- RSA + ChaCha20-Poly1305 handshake (Mode B) — M16.
- Request correlation by opcode (M5 uses FIFO ordering: the next
  notification is the response to the last request). Opcode-tag
  correlation is a nice-to-have for M12 hardening.
- Bulk streaming to an `AsyncWrite` sink — M5 reassembles into an
  in-memory `Vec<u8>` which is fine for M7's single-recording pulls
  and will be replaced in M9 if memory becomes an issue.
- Real-hardware CI. The `hw-tests` feature exists but is off by
  default; CI does not run it.

## Test plan

| Path | Focus | What it proves |
|---|---|---|
| `tests/auth_flow.rs` | `BleSession::authenticate` | wire bytes match `plaud_proto::encode::auth::authenticate`; accepted response → `Ok(())`; rejected response → `Err(AuthRejected { status: 1 })`; timeout → `Err(Timeout)`; invalid response payload → `Err(Protocol)` |
| `tests/control_roundtrip.rs` | `BleSession::send_control` | writes the supplied frame to the out channel; reads the next notification from the in channel; returns its payload; errors on timeout |
| `tests/bulk_reassembly.rs` | `BulkReassembler` + `BleSession::read_bulk` | ordered `Bulk` frames reassemble byte-for-byte; a `BulkEnd` terminates the stream; an out-of-order offset yields `Error::Protocol` |
| `tests/battery_transport.rs` | `BleTransport::battery` | delegates to `BatteryReader` without consulting session auth state |
| `tests/transport_unsupported.rs` | every M5-unsupported method on `BleTransport` | returns `Error::Unsupported { capability }` with a stable capability string pointing at the future milestone |
| `tests/discovery.rs` | `BleDiscovery` | scan returns candidates produced by the injected `ScanProvider` |

Target coverage for the session/transport logic: ≥ 90 %. The btleplug
backend is not covered by tests (its compile-only).

## Definition of Ready

- [x] M2 closed (codec)
- [x] M3 closed (sim)
- [x] M4 closed (auth store)
- [x] Evidence: auth status byte + soft-reject semantics live-tested

## Definition of Done

Mirror of the M5 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.

## Implementation (closed 2026-04-06)

**New crate**: `plaud-transport-ble` — protocol-layer session, transport,
and discovery, fully hermetic (zero BLE hardware required for any test).

### Sources

- [`crates/plaud-transport-ble/Cargo.toml`](../../../crates/plaud-transport-ble/Cargo.toml) — deps `plaud-domain`, `plaud-transport`, `plaud-proto`, `bytes`, `thiserror`, `async-trait`, `tokio`. Features: `btleplug-backend` (reserved, off), `hw-tests` (reserved, off).
- [`src/lib.rs`](../../../crates/plaud-transport-ble/src/lib.rs) — module tree and re-exports.
- [`src/constants.rs`](../../../crates/plaud-transport-ble/src/constants.rs) — `AUTH_RESPONSE_TIMEOUT`, `CONTROL_RESPONSE_TIMEOUT`, `BULK_FRAME_TIMEOUT`, `DEFAULT_CHANNEL_CAPACITY`, `AUTH_STATUS_REJECTED`, and the `CAP_*` capability strings used by `Error::Unsupported` stubs.
- [`src/channel.rs`](../../../crates/plaud-transport-ble/src/channel.rs) — `BleChannel` + `TestPeer` + `BleChannel::loopback_pair()`; the hermetic test factory every integration test uses.
- [`src/bulk.rs`](../../../crates/plaud-transport-ble/src/bulk.rs) — `BulkReassembler` with `feed` / `finish` / `FeedStatus`; validates monotone offsets and `file_id` consistency.
- [`src/battery.rs`](../../../crates/plaud-transport-ble/src/battery.rs) — `BatteryReader` trait + `FixedBatteryReader` test helper.
- [`src/session.rs`](../../../crates/plaud-transport-ble/src/session.rs) — `BleSession` with `authenticate`, `send_control`, `read_bulk`, `is_authenticated`.
- [`src/transport.rs`](../../../crates/plaud-transport-ble/src/transport.rs) — `BleTransport` implementing all 13 `Transport` methods; `battery()` is real, the rest return `Error::Unsupported { capability: CAP_* }`.
- [`src/discovery.rs`](../../../crates/plaud-transport-ble/src/discovery.rs) — `BleDiscovery` + injectable `ScanProvider` trait.

### Tests (all hermetic, 24 total)

- [`tests/auth_flow.rs`](../../../crates/plaud-transport-ble/tests/auth_flow.rs) — 4 tests: exact-wire-bytes, flag flip, rejected→`AuthRejected { status: 1 }`, malformed→`Protocol`.
- [`tests/control_roundtrip.rs`](../../../crates/plaud-transport-ble/tests/control_roundtrip.rs) — 3 tests: opcode match returns payload, pre-auth→`AuthRequired`, opcode mismatch→`Protocol`.
- [`tests/bulk_reassembly.rs`](../../../crates/plaud-transport-ble/tests/bulk_reassembly.rs) — 5 tests: happy-path reassembly, non-monotone offset rejection, mismatched `file_id` rejection, `finish`-without-`BulkEnd` rejection, end-to-end `read_bulk` over loopback.
- [`tests/battery_transport.rs`](../../../crates/plaud-transport-ble/tests/battery_transport.rs) — 1 test: battery read without authentication (matches Test 2b live evidence).
- [`tests/transport_unsupported.rs`](../../../crates/plaud-transport-ble/tests/transport_unsupported.rs) — 10 tests: every M5-unsupported method returns `Error::Unsupported` with a stable capability string pointing at M6/M7/M11.
- [`tests/discovery.rs`](../../../crates/plaud-transport-ble/tests/discovery.rs) — 2 tests: scan delegates to injected `ScanProvider`; `connect` returns `Unsupported`.

### Deferred (by design)

- Real `btleplug` backend and `hw-tests` smoke — lands incrementally
  alongside M6/M7/M11 as each vendor opcode becomes real; placeholder
  features reserved in `Cargo.toml`.
- `AsyncWrite`-sink streaming for bulk — M12 hardening; M5 reassembles
  to `Vec<u8>` which is sufficient for M7 single-recording pulls.
- Opcode-tag correlation across concurrent requests — M12 hardening;
  M5 uses strict FIFO ordering on the notification channel.
- RSA + ChaCha20-Poly1305 handshake (`Frame::Handshake`,
  `0xFE11`/`0xFE12`) — M16; M5 reserves the error path via
  `Error::Unsupported { capability: "rsa-chacha20-handshake" }`.

### Quality gates (all green at close)

- `cargo test` — 222 tests pass, 0 fail across the workspace.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- Zero `unwrap`/`expect` in production code.
- Zero `#[allow]` attributes.
- Zero dead code.
- Every public item documented.
