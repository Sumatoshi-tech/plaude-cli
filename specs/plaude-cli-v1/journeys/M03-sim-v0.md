# M03 — `plaud-sim` v0 (in-process device simulator)

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M3 |
| **Journey name** | "I can run a full CLI happy-path test against a deterministic fake Plaud device, with no BLE adapter and no hardware" |
| **Primary actor** | Engineer implementing M5–M12 — every milestone from here on uses this sim as its test fixture |
| **Dependencies** | M2 |
| **Blocks** | M5, M7, M9 |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md#m3` |

## Context

M0–M2 delivered the workspace, the domain vocabulary, and the wire-format
codec. M3 puts them together into a fake device that **implements every
method of [`plaud_transport::Transport`] in-process**, with deterministic
state, realistic auth semantics, and failure-injection knobs.

This is the **CI north star**: from M3 onward, no milestone may depend on
physical hardware for its mandatory test suite. Real-hardware smoke tests
are allowed but are gated behind a feature flag and excluded from the
default `make test` run.

Evidence inputs for behaviour correctness:

- Auth soft-reject semantics — `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md` Test 2b:
  "Vendor commands issued after a failed auth are silently ignored: no
  response, no error frame, no disconnect."
- Battery without auth — same document; Test 2b reads battery `0x64 = 100 %`
  via the standard SIG service in an unauthenticated session.
- Control frame + bulk stream layout — `docs/protocol/ble-commands.md`
  §§ 1–2 and `plaud-proto`.
- Setting enumeration — `plaud_domain::CommonSettingKey::all()`.

## Customer journey (CJM)

### Phase 1 — M5 engineer spins up a sim in a `#[tokio::test]`

**Action**:

```rust
let sim = SimDevice::builder()
    .with_expected_token(token.clone())
    .preload_recording(sample_recording(), wav_bytes, asr_bytes)
    .with_battery(BatteryLevel::new(87).unwrap())
    .build();
let transport = sim.authenticated_transport();
```

**Expected**: zero BLE setup, zero sleeps, zero flaky timing. A working
`Box<dyn Transport>` in three method calls.

### Phase 2 — M5 engineer exercises the auth flow end-to-end

**Action**:

```rust
let discovery = sim.discovery(valid_token);
let candidate = discovery.scan(Duration::from_millis(1)).await?[0].clone();
let transport = discovery.connect(&candidate).await?;
```

**Expected**: valid token → `Ok(transport)`; wrong token → `Err(Error::AuthRejected { status: 0x01 })`.

### Phase 3 — M6 engineer verifies battery works without auth

**Action**: builds a sim with **no** auth short-circuit, calls
`SimDevice::unauthenticated_transport()`, and calls `transport.battery()`.

**Expected**: `Ok(BatteryLevel { percent: … })`. Any vendor op on the same
transport returns `Err(Error::AuthRequired)`.

### Phase 4 — M7 engineer reads a preloaded recording

**Action**: preload `sample_recording` with known WAV bytes, call
`transport.read_recording(&id)`, compare the returned bytes to the preload.

**Expected**: byte-for-byte equality with the preloaded WAV.

### Phase 5 — M9 engineer stress-tests partial-failure recovery

**Action**: build a sim with `inject_disconnect_after(3)`, run a list →
pull → list sequence, expect the third op to fail with a transport
error; verify state is sane on retry after a fresh transport.

**Expected**: first two ops succeed; third op returns
`Err(Error::Transport(..))`; op counter observable.

## Engineer journey (micro-TDD)

1. **Plan** — "Sim's `battery()` returns the configured value without
   touching auth state."
2. **Test-RED** — one assertion: `transport.battery().await.unwrap() == BatteryLevel::new(50).unwrap()`.
3. **Code-GREEN** — minimal `impl Transport for SimTransport` with just
   the `battery()` method; others return `todo!()`… wait, AGENTS.md
   bans `todo!()`. Use `Err(Unsupported)` as the skeleton return.
4. Iterate: `device_info`, `storage`, `list_recordings`, `read_recording`,
   `delete_recording`, `read_setting`, `write_setting`, `start_recording`,
   `stop_recording`, `pause_recording`, `resume_recording`, `set_privacy`,
   then the discovery flow, then failure injection, then determinism check.

## Scope (this milestone)

**In scope:**

- `SimDevice` struct + `SimDeviceBuilder` for test setup.
- `SimTransport` implementing every [`plaud_transport::Transport`] method.
- `SimDiscovery` implementing [`plaud_transport::DeviceDiscovery`].
- Auth state machine: `Unauthenticated` → `Accepted` | `SoftRejected`.
- Battery readable regardless of auth state (standard SIG analogue).
- Recording preload via `builder.preload_recording(rec, wav, asr)`.
- `read_recording` returns the preloaded WAV bytes (ASR stays internal
  to the sim; M7 will widen the trait if needed).
- Record control state machine (Idle / Recording / Paused) with the
  obvious transitions.
- `CommonSettingKey` settings store backed by a `HashMap`.
- Failure injection: `inject_disconnect_after(n_ops)`,
  `inject_delay(per_op)`.
- `plaud_sim::bulk::frames_for` helper — given a file id and a byte
  slice, produces the sequence of `plaud_proto::Frame::Bulk` frames
  (plus a terminating `Frame::BulkEnd`) a real device would emit for
  a `ReadFileChunk` that covers those bytes. Used by M5's encoder
  round-trip tests.
- Deterministic: two independent sim runs with the same inputs
  produce bit-identical traces.

**Out of scope (deferred):**

- Literal BLE wire-format simulation (parsing incoming auth-frame
  bytes, emitting control-frame response bytes). The sim operates at
  the `Transport` trait level, not the bytes level. A BLE-level
  mock peripheral lands with M5/M8 when it is actually needed.
- RSA + ChaCha20-Poly1305 handshake (Mode B). Deferred to M16.
- A full `AuthStore` integration — M3 stores the expected token
  directly on `SimDevice`. M4 lands the real auth store.

## Test plan

| Path | Scope | What it proves |
|---|---|---|
| `tests/device_info.rs` | integration | `device_info` returns the preloaded identity |
| `tests/battery.rs` | integration | battery works with and without auth, returns preloaded level |
| `tests/storage.rs` | integration | `storage` returns the preloaded `StorageStats` |
| `tests/auth.rs` | integration | correct token → Accepted; wrong token → `AuthRejected { status: 1 }`; vendor ops post-reject return `AuthRejected` while battery still works |
| `tests/recordings.rs` | integration | `list_recordings`, `read_recording`, `delete_recording` happy paths + missing-id error |
| `tests/settings.rs` | integration | read default, write, round-trip; unknown key behaviour (N/A — enum is exhaustive) |
| `tests/record_control.rs` | integration | `start`, `pause`, `resume`, `stop` state transitions; idempotency + illegal transitions |
| `tests/privacy.rs` | integration | `set_privacy(true)`, `set_privacy(false)` persist across reads |
| `tests/discovery.rs` | integration | `scan` returns the configured candidate; `connect` with matching / mismatching token |
| `tests/injection.rs` | integration | `inject_disconnect_after(n)` produces `Error::Transport` from the `(n+1)`th op onward; `inject_delay` actually sleeps |
| `tests/bulk_frames.rs` | integration | `bulk::frames_for` produces N data frames + 1 `BulkEnd`, offsets step by 80, round-trip through `plaud_proto::decode::parse_notification` |
| `tests/determinism.rs` | integration | two sims built from identical builder chains produce identical traces |

Target coverage for `plaud-sim`: ≥ 90 %.

## Definition of Ready (DoR)

- [x] M1 closed; `Transport` / `DeviceDiscovery` / `Error` types stable
- [x] M2 closed; `plaud_proto::Frame` and encoders available for bulk helper

## Definition of Done

Mirror of the M3 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.
