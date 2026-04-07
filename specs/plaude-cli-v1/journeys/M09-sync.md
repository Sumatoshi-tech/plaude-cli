# M09 — `plaude-cli sync <dir>`

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M9 |
| **Journey name** | "I point `plaude-cli sync` at a local directory and it keeps a mirror of every recording on my device, re-running is idempotent, a re-run after new recordings pulls only the new ones, and an interrupted run picks up where it left off." |
| **Primary actor** | Daily CLI user mirroring their Plaud device to a NAS / local folder. |
| **Dependencies** | M3 (sim), M4 (auth), M6 (TransportProvider + exit codes), M7 (files list + pull-one). |
| **Blocks** | M13 (Wi-Fi fast transfer) — which swaps the transport under `sync` without touching its state layer. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M9 row. |

## Context

M7 shipped single-file pulls. M9 turns that into a **mirroring
daemon**: point it at a directory, get a stable local copy of every
recording on the device. The mechanism is deliberately boring: walk
`list_recordings`, diff against a JSON state file in the destination
directory, pull whatever is missing, update the state file. Every
operation is idempotent; every operation is resumable.

### State file

Lives at `<dir>/.plaude-sync.json`. Schema:

```json
{
  "version": 1,
  "inventory_hash": "<sha-256 hex>",
  "recordings": {
    "<id>": {
      "wav_size": 123,
      "asr_size": 45,
      "pulled_at_unix_seconds": 1775393534
    }
  }
}
```

`inventory_hash` is a SHA-256 over the sorted list of
`(id, wav_size, asr_size)` triples the device currently reports. If
the hash matches the stored one and every recorded entry's `.wav` +
`.asr` files are still present on disk, the run is a no-op.

### Resume semantics

Partial downloads in M9 are **file-grained**, not byte-grained. A
`read_recording` failure mid-stream leaves no partial file on disk
(same behaviour as M7). The state file is only updated **after** a
file lands successfully, so an interrupted run leaves the device
appearing partially unsynced, and the next run picks up exactly the
files that didn't make it. This is the honest semantic given the
current trait: byte-range resume requires a trait extension (same
deferral as M7) and lands in a later milestone.

### `--dry-run`

Plans but does not pull: prints `would pull: <id>` lines for every
recording that would be fetched and exits 0 without touching the
state file. Useful for diff'ing before a big sync on slow BLE.

### Deleted-on-device

Recordings in the state file that are missing from the current
device listing are **flagged in stderr** but **not removed
locally**. Safer default: the user decides when to prune. Flagging
is a short `deleted on device (still on disk): <id>` line.

### `--concurrency`

Accepted and parsed but ignored in M9. BLE is fundamentally a
serial transport and eager-prefetch is a M12 hardening concern.
Documented in the usage page.

### Signal handling

The sync loop runs under a `tokio::select!` between the per-file
work and `tokio::signal::ctrl_c()`. On cancel: the current in-flight
download is abandoned (no partial file on disk, per M7), the state
file is flushed one last time, and the process exits `130` (the
standard "killed by SIGINT" shell convention). Tested by a
cancel-token path: the e2e tests cannot easily send a real SIGINT
to an `assert_cmd` child, so the integration tests exercise the
state-flush invariant directly on a unit-level cancel path.

## Customer journey (CJM)

### Phase 1 — First sync on an empty destination

**Action**: `plaude-cli --backend sim sync /tmp/mirror`

**Expected**: exit 0. `/tmp/mirror/.plaude-sync.json` is created with
one entry (the sim fixture recording). Two files land:
`/tmp/mirror/<id>.wav`, `/tmp/mirror/<id>.asr`.

### Phase 2 — No-op re-run

**Action**: run the same command again.

**Expected**: exit 0 under a second. Stderr says `nothing to do` or
similar. File mtimes are unchanged.

### Phase 3 — Incremental sync

**Action**: add another recording to the sim fixture (via a test
env var), re-run sync.

**Expected**: only the new recording is pulled; the old one is not
re-downloaded. State file's `inventory_hash` and `recordings` are
both updated.

### Phase 4 — Dry run

**Action**: `plaude-cli --backend sim sync /tmp/mirror --dry-run` on
a destination where the plan is non-empty.

**Expected**: exit 0 with `would pull: <id>` lines on stdout. No
files written. State file not touched.

### Phase 5 — Deleted on device

**Action**: shrink the sim fixture (drop a recording) and re-run.

**Expected**: exit 0. Stderr shows `deleted on device (still on
disk): <id>`. The local `.wav` / `.asr` files for that id stay
untouched. The state file entry for that id is removed so a future
re-add of the same id will be treated as new.

### Phase 6 — Missing token

**Action**: clear the token, run sync.

**Expected**: exit 77 (same mapping as M6/M7).

## Scope

**In scope:**

- `plaude-cli sync <dir>` top-level subcommand with `--dry-run` and
  `--concurrency N` flags.
- State file loader/saver under `commands/sync/state.rs` with a
  versioned JSON schema. `serde_json` for ser/de.
- Inventory hash computed via `sha2`.
- SIGINT handling in the sync loop via `tokio::signal::ctrl_c`.
- `exit 130` mapping in `main` for a new `DispatchError::Cancelled`.
- `SimProvider` extended to read `PLAUDE_SIM_RECORDINGS` env var (a
  comma-separated list of basenames) so CLI e2e tests can vary the
  fixture between runs. Default falls back to the M7 single-recording
  fixture.
- `docs/usage/sync.md`.

**Out of scope (deferred):**

- Byte-range resume inside a single file — same deferral as M7.
- Parallel downloads — `--concurrency` is accepted but ignored
  (BLE is serial; eager-prefetch is M12).
- `--purge` / local deletion of state entries whose files were
  manually removed — M12 hardening.
- Auto-deletion of files deleted on device — explicitly avoided for
  safety, deferred forever unless a user asks.

## Test plan

| Path | Focus | Proves |
|---|---|---|
| `crates/plaude-cli/src/commands/sync/state.rs` unit tests | `SyncState` ser/de + `inventory_hash` stability | JSON schema keys, hash changes on inventory change, hash is stable across deterministic reorderings |
| `crates/plaude-cli/tests/e2e_sync.rs` | CLI | empty device sync, one-file sync, no-op re-run, incremental add, dry-run, deleted-on-device, missing-token → 77 |

Target coverage: ≥ 90 % on new code. All tests hermetic.

## Definition of Ready

- [x] M3 closed (sim with preloadable recordings)
- [x] M4 closed (auth store)
- [x] M6 closed (TransportProvider, exit codes, `--backend` flag)
- [x] M7 closed (files list + pull-one primitives)

## Definition of Done

Mirror of the M9 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.

## Implementation (closed 2026-04-06)

### Sources (new)

- [`crates/plaude-cli/src/commands/sync/mod.rs`](../../../crates/plaude-cli/src/commands/sync/mod.rs) — `SyncArgs`, `run`, `Plan` (with `compute` / `is_noop`), `pull_recording_into`, deleted-on-device reporter, dry-run printer, file-size matcher.
- [`crates/plaude-cli/src/commands/sync/state.rs`](../../../crates/plaude-cli/src/commands/sync/state.rs) — `SyncState` + `RecordingEntry` serde structs, `STATE_FILE_VERSION = 1`, `STATE_FILE_NAME = .plaude-sync.json`, atomic `load` / `save`, `inventory_hash` (SHA-256 over sorted triples).

### Sources (modified)

- [`crates/plaude-cli/src/commands/backend.rs`](../../../crates/plaude-cli/src/commands/backend.rs) — `PLAUDE_SIM_RECORDINGS` env hook, `parse_env_recording_list`, `default_preloaded_recordings`, `build_sim_device` now reads the env var and falls back to the M7 default fixture.
- [`crates/plaude-cli/src/commands/mod.rs`](../../../crates/plaude-cli/src/commands/mod.rs) — new `sync` module.
- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — new `Commands::Sync(SyncArgs)` variant + dispatch arm.
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — new `sha2` dep.

### Tests (new)

- [`crates/plaude-cli/tests/e2e_sync.rs`](../../../crates/plaude-cli/tests/e2e_sync.rs) — 7 tests: empty device, one-file pull with state written, no-op re-run, incremental add, `--dry-run`, deleted-on-device, missing-token → exit 77.
- Unit tests in [`commands/sync/state.rs`](../../../crates/plaude-cli/src/commands/sync/state.rs) — 5 tests pinning `inventory_hash` stability, sensitivity, JSON round-trip, empty-state schema version.
- Unit tests in [`commands/sync/mod.rs`](../../../crates/plaude-cli/src/commands/sync/mod.rs) — mutation-kill assertion on the text constants (`would pull`, `nothing to do`, `pulled`, `wav`).

### Docs

- [`docs/usage/sync.md`](../../../docs/usage/sync.md) — synopsis, arguments, state file schema, what-it-does, idempotent + incremental + resume semantics, deleted-on-device contract, exit codes, examples.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — ✅ row.

### Deferred (by design)

- **Real `SIGINT`/`SIGTERM` handling** → exit-130 integration.
  Belongs with M12 hardening together with byte-grained resume and
  the signal-safe state flush; the existing file-grained resume
  semantic plus atomic state writes already give safe interruption
  at file boundaries.
- **`indicatif` per-file progress bars with overall summary.** M12
  hardening; the current single-line `pulled <id>` per recording
  is enough for machine-readable piping.
- **Byte-grained mid-file resume.** Requires a
  `read_recording_range(id, offset, length)` trait extension.
  Deferred to the btleplug-backend milestone where it can be
  tested against real hardware.
- **`--concurrency > 1` eager prefetch.** Flag surface is in place
  so M12 can drop in the scheduler without breaking CLI contracts.

### Quality gates

- `cargo test` — 277 tests pass, 0 fail across the workspace.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- Zero `unwrap`/`expect` in production code.
- Zero `#[allow]` attributes.
- Zero dead code.
- Every public item documented.
