# Journey L4: Single-Recording Summarization Pipeline

**Roadmap item:** L4 — Single-recording summarization with streaming output
**Spec:** `specs/llm-features/SPEC.md` §4

## Persona

**Dmitriy** — has Ollama running, transcripts from `plaude transcribe`.
Wants to summarize a single recording's transcript.

## Trigger

User runs `plaude summarize ./recordings/1712345678.txt` or
`plaude summarize ./recordings/` (directory with transcript files).

## Phases

### Phase 1: Transcript Discovery
- **Action:** User points at a file or directory
- **System:** Locates `.txt`/`.json`/`.srt`/`.vtt` transcript
- **Success:** Transcript loaded into memory
- **Pain:** No transcript → error with `plaude transcribe` suggestion

### Phase 2: Summarization
- **Action:** System chunks transcript, sends to LLM
- **System:** Template as system message, transcript as user message
- **Success:** LLM returns summary text, streamed to stderr
- **Pain:** LLM unreachable → clear connection error

### Phase 3: Output
- **Action:** System writes summary file
- **System:** Atomic write (`.tmp` → rename), YAML front matter
- **Success:** `<id>.summary.<template>.md` created alongside transcript
- **Pain:** None — transparent

## CLI Interface

```bash
plaude summarize <path>                    # Summarize with default template
plaude summarize <path> --template brief   # Use specific template
plaude summarize <path> --model gpt-4o     # Override model
plaude summarize <path> --no-stream        # Suppress streaming output
plaude summarize <path> --json             # JSON metadata to stdout
```

## Tests

### Unit Tests (plaud-llm/src/summarize.rs)
- `discover_transcript_finds_txt` — finds .txt file
- `discover_transcript_finds_json` — finds .json file
- `discover_transcript_prefers_json` — .json wins over .txt
- `discover_transcript_missing_returns_error` — no transcript → error
- `front_matter_format` — YAML front matter is well-formed
- `summary_filename` — naming convention is correct

### E2E Tests (plaude-cli)
- `summarize_missing_transcript_suggests_transcribe`
- `summarize_with_path_flag_works` (when LLM not available, tests error path)

## Implementation

### Files Created
- `crates/plaud-llm/src/summarize.rs` — `SummarizeError`, `SummaryResult`, `SummarizeOptions`, `discover_transcript()`, `summary_filename()`, `format_front_matter()`, `run_pipeline()`, `write_summary()`, streaming + non-streaming LLM calls, 10 unit tests

### Files Modified
- `crates/plaud-llm/src/lib.rs` — added `pub mod summarize`
- `crates/plaud-llm/Cargo.toml` — added `jiff`, `futures`, `serde_json` deps
- `crates/plaude-cli/src/commands/summarize.rs` — extended with `--template`, `--model`, `--no-stream`, `--json`, `<PATH>` positional arg, full pipeline integration
- `crates/plaude-cli/src/main.rs` — dispatch now uses `runtime.block_on()` for async summarize
- `crates/plaude-cli/tests/e2e_summarize.rs` — extended to 10 E2E tests (template + pipeline)
