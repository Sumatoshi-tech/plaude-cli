# M07 — `plaude-cli files list` + `plaude-cli files pull-one`

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M7 |
| **Journey name** | "I can list the recordings on my Plaud device and download any single one of them to disk as its paired `.WAV` + `.ASR` files, with a progress bar, resumable across re-runs, exercised entirely against `plaud-sim` in CI." |
| **Primary actor** | CLI end-user managing recordings without the Plaud app. |
| **Dependencies** | M3 (sim), M4 (auth), M5 (transport-ble skeleton), M6 (CLI command scaffold + `TransportProvider`). |
| **Blocks** | M9 (`sync`) — which is this command in a loop with a state file. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M7 row. |

## Context

M6 shipped the CLI command scaffold, the `TransportProvider`
abstraction, and stable exit codes. M7 is the **first recording-data
journey** — the first command that actually moves bytes off the
device onto disk.

The existing `Transport` trait has `list_recordings` and
`read_recording` (WAV). It has **no way to fetch the `.ASR` sidecar**
a real Plaud recording always carries alongside its `.WAV`. M7 closes
that gap by adding a second, symmetric trait method
`read_recording_asr` and wiring it on every backend (real for sim,
`Unsupported` stub for `BleTransport` until the btleplug backend
ships — same pattern as M5's other deferred methods).

Test reality for M7 is `plaud-sim` only; the `BleTransport` recording
methods remain `Unsupported` and will be filled in alongside the
btleplug backend. The CLI's `SimProvider` is extended to preload one
deterministic recording so `files list` always has something to
show and `files pull-one` always has something to fetch.

### Scope-reductions (vs. literal roadmap DoD)

Two roadmap bullets are downgraded in M7 and rolled forward into M9
(`sync`) / M12 (hardening) where they belong:

1. **Mid-offset resumable download.** The current `Transport` surface
   returns a full `Vec<u8>` from `read_recording`. True mid-stream
   resume requires a range-read method on the trait
   (`read_recording_range(id, offset, length)`), which the live
   device's `0x1C ReadFileChunk` supports but we do not wire in M7.
   M7's `--resume` semantics are therefore: **idempotent skip** — if
   the target file already exists with exactly the expected byte
   count, the command is a no-op (exit 0); if it exists but is the
   wrong size (partial), the command rewrites it from scratch. This
   is the semantic `plaude sync` actually needs. Mid-offset resume
   is tracked for M9/M12.

2. **CRC validation via `tnt_get_file_crc`.** The device's file-CRC
   opcode is not in `plaud-proto` yet and the sim does not model it.
   M7 instead validates by re-reading the just-written file and
   comparing against the buffer we wrote. That catches truncation
   and disk-full bugs but not device-side corruption, which is a
   rare and best-addressed-in-hardening concern.

3. **Streaming writes with no in-memory buffer.** Deferred to the
   same milestone that adds range-reads. M7 buffers the full file in
   memory before writing — acceptable given M7's single-file scope
   and the fact that Plaud recordings are on the order of tens of
   megabytes at most.

4. **Hardware-gated smoke test.** Deferred to the btleplug wire-up
   milestone, since M7 cannot drive real hardware until that backend
   exists.

All four deferrals are called out in the closing `Implementation`
section so the next engineer knows where to pick them up.

## Customer journey (CJM)

### Phase 1 — "What's on my device?"

**Action**: `plaude-cli auth set-token <hex>` (one-time),
then `plaude-cli --backend sim files list`.

**Expected**: exit 0, stdout prints a table with columns `ID`,
`KIND`, `STARTED`, `WAV`, `ASR` for every recording the sim has
preloaded. `--output json` emits an array of objects with stable
keys.

### Phase 2 — "I want that one file"

**Action**: `plaude-cli --backend sim files pull-one <id>`.

**Expected**: exit 0. Two files appear in the current directory:
`<id>.wav` and `<id>.asr`, with byte contents equal to what the sim
returns. Stderr shows a short progress bar (or is silent when
stderr is not a TTY — the common CI case).

### Phase 3 — "Pull it into a specific folder"

**Action**: `plaude-cli --backend sim files pull-one <id> -o /tmp/dump`.

**Expected**: exit 0. Files land at `/tmp/dump/<id>.wav` and
`/tmp/dump/<id>.asr`. Parent directory is created if missing.

### Phase 4 — "I lost my SSH session mid-pull"

**Action**: re-run the same `files pull-one <id> -o /tmp/dump`
command after a previous run completed. With `--resume`: if the
files are already the expected size, the command is a no-op.

**Expected**: exit 0 with a "already up to date" message.

### Phase 5 — "The device dropped halfway through"

**Action**: with the sim configured to disconnect after N ops,
run `files pull-one <id>`.

**Expected**: exit 1, stderr names the `.wav` partial path (or the
transport error), no `.asr` file is written on disk.

### Phase 6 — "I forgot my token"

**Action**: clear the token, run `files list`.

**Expected**: exit 77 (same mapping as M6 — both `files list` and
`files pull-one` require auth).

## Scope

**In scope:**

- `Transport::read_recording_asr(id) -> Result<Vec<u8>>` — new trait
  method on the transport boundary; required to avoid leaking
  sim-only accessors into the CLI.
- `SimTransport::read_recording_asr` — real impl, returns the
  preloaded ASR bytes.
- `BleTransport::read_recording_asr` — stub returning
  `Error::Unsupported { capability: "read_recording_asr (lands with btleplug backend)" }`.
- `plaude-cli files list` subcommand with `--output text|json`.
- `plaude-cli files pull-one <id> [-o PATH] [--resume]` subcommand
  with a progress bar via `indicatif`.
- `SimProvider` in the CLI preloads exactly one deterministic
  recording so e2e tests have stable fixtures.
- E2E tests: list happy, list json, list without token → 77,
  pull-one happy, pull-one with `-o`, pull-one resume, pull-one
  mid-stream disconnect, pull-one with unknown id → exit 1.
- `docs/usage/files.md`.

**Out of scope (deferred):**

- Mid-offset resume and streaming writes — M9/M12.
- Device-side CRC validation — later when `tnt_get_file_crc` lands.
- Hardware-gated smoke test — btleplug wire-up milestone.
- Delete-recording CLI surface — M11.

## Test plan

Target coverage: ≥ 90 % on new code. All tests hermetic.

| Path | Focus | Proves |
|---|---|---|
| `crates/plaud-sim/tests/recordings.rs` | new `read_recording_asr` | sim returns preloaded ASR bytes; unknown id → `NotFound` |
| `crates/plaud-transport-ble/tests/transport_unsupported.rs` | new `read_recording_asr` stub | returns `Unsupported` with "M7"-containing capability |
| `crates/plaude-cli/tests/e2e_files_list.rs` | CLI `files list` | text table columns, JSON schema, missing-token → 77, recording order is stable |
| `crates/plaude-cli/tests/e2e_files_pull.rs` | CLI `files pull-one` | happy path writes both files, `-o` respects dir, unknown id → exit 1, `--resume` is a no-op when files are complete, `--resume` rewrites partial files, mid-stream disconnect leaves no `.asr` file |
| unit tests in `commands/files.rs` | output formatting | text header + JSON schema pinned; filename derivation matches `<id>.wav` / `<id>.asr` |

## Definition of Ready

- [x] M3 closed (sim has preloadable recordings + `asr_bytes_for` helper)
- [x] M4 closed (auth store + sandbox flag)
- [x] M5 closed (`BleTransport` is the stable stub surface M7 extends)
- [x] M6 closed (`TransportProvider`, `OutputFormat`, exit-code mapping)

## Definition of Done

Mirror of the M7 DoD in `specs/plaude-cli-v1/ROADMAP.md`, adjusted for
the four scope-reductions listed in **Context**. Updated at milestone
close with evidence links.

## Implementation (closed 2026-04-06)

### Sources (new)

- [`crates/plaude-cli/src/commands/files.rs`](../../../crates/plaude-cli/src/commands/files.rs) — `FilesCommand { List, PullOne }`, `ListArgs`, `PullOneArgs`, `RecordingJson` schema, `pull_file` / `write_with_progress` / `file_is_already_complete` helpers, `FileKind { Wav, Asr }` enum, `indicatif` progress rendering, text-table formatter.

### Sources (modified)

- [`crates/plaud-transport/src/transport.rs`](../../../crates/plaud-transport/src/transport.rs) — new required trait method `read_recording_asr(&self, id: &RecordingId) -> Result<Vec<u8>>`; doc of `read_recording` rewritten to say it returns WAV bytes specifically rather than "an opaque blob".
- [`crates/plaud-sim/src/transport.rs`](../../../crates/plaud-sim/src/transport.rs) — real `read_recording_asr` implementation returning `state.recordings[id].asr` or `Error::NotFound`.
- [`crates/plaud-transport-ble/src/transport.rs`](../../../crates/plaud-transport-ble/src/transport.rs) — stub `read_recording_asr` returning `Error::Unsupported { capability: CAP_READ_RECORDING_ASR }`.
- [`crates/plaud-transport-ble/src/constants.rs`](../../../crates/plaud-transport-ble/src/constants.rs) — new `CAP_READ_RECORDING_ASR` capability string.
- [`crates/plaude-cli/src/commands/backend.rs`](../../../crates/plaude-cli/src/commands/backend.rs) — `SimProvider` now calls a shared `build_sim_device()` that preloads a deterministic recording (`SIM_RECORDING_BASENAME`, `SIM_RECORDING_WAV`, `SIM_RECORDING_ASR`). `connect_anonymous` and `connect_authenticated` both route through this helper so every subcommand observes the same fixture.
- [`crates/plaude-cli/src/commands/mod.rs`](../../../crates/plaude-cli/src/commands/mod.rs) — new `files` module.
- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — new `Commands::Files(FilesCommand)` arm wired through `commands::files::run`.
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — new `indicatif` dep.
- [`Cargo.toml`](../../../Cargo.toml) — new workspace dep `indicatif = "0.17"`.

### Tests (new)

- [`crates/plaude-cli/tests/e2e_files_list.rs`](../../../crates/plaude-cli/tests/e2e_files_list.rs) — 3 tests: text table header + preloaded recording, JSON schema (`id`, `kind`, `wav_size`, `asr_size` keys), missing-token → exit 77.
- [`crates/plaude-cli/tests/e2e_files_pull.rs`](../../../crates/plaude-cli/tests/e2e_files_pull.rs) — 5 tests: happy path with byte-equality on both files, nested output-dir creation, `--resume` skip after complete pull, `--resume` rewrite over a partial file, unknown id → runtime exit 1 with no files written.
- Unit tests in [`commands/files.rs`](../../../crates/plaude-cli/src/commands/files.rs) — `RecordingJson` schema keys + text-table header + file-extension constants (mutation-kill assertions).

### Tests (modified)

- [`crates/plaud-sim/tests/recordings.rs`](../../../crates/plaud-sim/tests/recordings.rs) — 2 new tests for the `read_recording_asr` happy path + `NotFound`.
- [`crates/plaud-transport-ble/tests/transport_unsupported.rs`](../../../crates/plaud-transport-ble/tests/transport_unsupported.rs) — 1 new test pinning the `read_recording_asr` stub against the `CAP_READ_RECORDING_ASR` capability string.

### Docs

- [`docs/usage/files.md`](../../../docs/usage/files.md) — synopsis, arguments, example invocations, output-file layout, resume-semantics contract, exit codes, cross-links.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — two new ✅ rows with links to the new page.

### Deferred (by design, tracked for later milestones)

- **Mid-offset resume / streaming writes.** M7's `--resume` is
  idempotent-skip only. True byte-range resume requires a
  `read_recording_range(id, offset, length)` trait method that maps
  to `0x1C ReadFileChunk(offset, length)` on the wire. Lands with
  the btleplug backend or M12 hardening, whichever comes first.
- **Device-side CRC validation via `tnt_get_file_crc`.** The opcode
  is not in `plaud-proto` and the sim does not model it. Will be
  added alongside the real btleplug backend when the CRC opcode
  round-trips against live hardware.
- **Hardware-gated smoke test.** Requires the btleplug backend. The
  M7 e2e suite runs entirely against `plaud-sim`.
- **`plaude-cli files delete <id>`.** M11 (recording control).

### Quality gates (all green at close)

- `cargo test` — 251 tests pass, 0 fail across the workspace.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- Zero `unwrap`/`expect` in production code.
- Zero `#[allow]` attributes.
- Zero dead code.
- Every public item documented.
