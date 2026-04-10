# Journey L6: Transcript Auto-Correction via LLM

**Roadmap item:** L6 — Transcript auto-correction via LLM
**Spec:** `specs/llm-features/SPEC.md` §3

## Persona

**Dmitriy** — has raw transcripts from whisper.cpp that contain
misheard technical terms, broken punctuation, and filler words.
Wants a cleaned-up version before sharing or summarizing.

## Trigger

User runs `plaude correct ./recordings/1712345678.txt` to produce
a corrected transcript without modifying the original.

## Phases

### Phase 1: Basic Correction
- **Action:** `plaude correct <transcript>`
- **System:** Chunks text, sends each through LLM with correction prompt
- **Success:** `<id>.corrected.txt` written alongside original
- **Pain:** LLM not reachable → clear error

### Phase 2: Glossary Injection
- **Action:** `plaude correct <transcript> --glossary terms.txt`
- **System:** Loads glossary terms, injects into system prompt
- **Success:** Domain terms spelled correctly in output
- **Pain:** Missing glossary file → clear error

## CLI Interface

```bash
plaude correct <path>                       # Correct with default prompt
plaude correct <path> --glossary terms.txt  # With domain glossary
plaude correct <path> --model gpt-4o        # Override model
plaude correct <path> --no-stream           # Suppress streaming
```

## Tests

### Unit Tests (plaud-llm/src/correct.rs)
- `correction_prompt_contains_instructions`
- `correction_prompt_includes_glossary`
- `correction_prompt_without_glossary`
- `corrected_filename_convention`

### E2E Tests
- `correct_help_exits_zero`
- `correct_missing_file_errors`
- `correct_no_args_shows_usage`

## Implementation

### Files Created
- `crates/plaud-llm/src/correct.rs` — correction prompt builder, glossary loading, correction pipeline, atomic output write, 8 unit tests
- `crates/plaude-cli/src/commands/correct.rs` — `plaude correct` CLI command with `--glossary`, `--model`, `--no-stream`
- `crates/plaude-cli/tests/e2e_correct.rs` — 4 E2E tests

### Files Modified
- `crates/plaud-llm/src/lib.rs` — added `pub mod correct`
- `crates/plaude-cli/src/commands/mod.rs` — added `correct` module (feature-gated)
- `crates/plaude-cli/src/main.rs` — added `Correct` to `Commands` enum + dispatch
