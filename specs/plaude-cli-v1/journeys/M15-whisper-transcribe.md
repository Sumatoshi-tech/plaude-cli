# M15 — `plaude transcribe` via whisper.cpp

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M15 |
| **Journey name** | "I can transcribe a Plaud recording WAV file to text using a local whisper.cpp binary, entirely offline." |
| **Primary actor** | CLI end-user who has pulled WAV files and wants transcription without cloud. |
| **Dependencies** | M7 or M10 (WAV file production — both closed). |
| **Blocks** | Nothing — this is a stretch goal. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M15 row. |

## Context

M7 and M10 ship WAV files to disk. Users want offline transcription.
`whisper.cpp` is the de-facto open-source local speech-to-text engine.
M15 wraps a user-supplied `whisper.cpp` CLI binary (`main` or
`whisper-cli`) as a subprocess, passing the WAV file and a configurable
model path.

The command does **not** embed or download whisper.cpp — it is a thin
wrapper that invokes an externally-installed binary. This keeps the
CLI simple and avoids bundling a large model or native library.

### Design decisions

- **`--whisper-bin <PATH>`** flag + `PLAUDE_WHISPER_BIN` env var to
  locate the whisper.cpp binary. Defaults to `whisper-cli` on `$PATH`.
- **`--model <PATH>`** flag + `PLAUDE_WHISPER_MODEL` env var for the
  GGML model file. Required (no default — models are large).
- **`--language <LANG>`** optional flag to pass to whisper.
- **`--output-format <txt|srt|vtt>`** controls whisper's `--output-*` flag.
- The command accepts one or more WAV file paths as positional args.
- stdout gets the transcript text; stderr gets progress/errors.
- Exit code follows the sysexits convention from M12.

## Customer journey (CJM)

### Phase 1 — "Transcribe a single file"

**Action**: `plaude-cli transcribe --model ~/models/ggml-base.bin recording.wav`

**Expected**: exit 0, stdout contains the transcript text.

### Phase 2 — "Whisper binary not found"

**Action**: `plaude-cli transcribe --whisper-bin /nonexistent --model m.bin f.wav`

**Expected**: exit 69 (EX_UNAVAILABLE), stderr says binary not found.

### Phase 3 — "Model file doesn't exist"

**Action**: `plaude-cli transcribe --model /nonexistent f.wav`

**Expected**: exit 2 (usage error), stderr says model not found.

### Phase 4 — "WAV file doesn't exist"

**Action**: `plaude-cli transcribe --model m.bin /nonexistent.wav`

**Expected**: exit 1 (runtime error), stderr says file not found.

### Phase 5 — "Whisper process fails"

**Action**: whisper binary exits with nonzero code.

**Expected**: exit 1 (runtime error), stderr forwards whisper's stderr.

## Acceptance criteria

- [x] `plaude-cli transcribe --model <path> <wav>...` invokes whisper
- [x] `--whisper-bin` + `PLAUDE_WHISPER_BIN` configure the binary path
- [x] `--model` + `PLAUDE_WHISPER_MODEL` configure the model path
- [x] `--language` optional language hint
- [x] `--output-format txt|srt|vtt` controls output format
- [x] Missing binary → exit 69
- [x] Missing model → exit 2
- [x] Missing WAV → exit 1
- [x] Whisper failure → exit 1 with stderr forwarded
- [x] E2e tests with a mock whisper binary (shell script)
- [x] `docs/usage/transcribe.md` added
- [x] `make lint` clean, `make test` green

## Implementation

### Files created

- `crates/plaude-cli/src/commands/transcribe.rs` — `TranscribeArgs`, `TranscribeFormat` enum, `run()`, `transcribe_one()`, 1 unit test
- `crates/plaude-cli/tests/e2e_transcribe.rs` — 6 e2e tests with mock shell-script whisper binaries
- `docs/usage/transcribe.md` — user documentation with examples

### Files modified

- `crates/plaude-cli/src/commands/mod.rs` — added `transcribe` module
- `crates/plaude-cli/src/main.rs` — added `Transcribe` variant to `Commands` enum + dispatch arm
- `docs/usage/index.md` — added M15 row
