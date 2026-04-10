# LLM Integration — Roadmap

Master implementation checklist for multi-provider LLM integration:
summarization with templates, summary management, transcript
correction, and sync automation.

**Spec:** [`SPEC.md`](SPEC.md)

## Current state

`plaude transcribe` produces text/srt/vtt/json transcripts via
in-process whisper-rs with speaker diarization. Sync mirrors
recordings to a local directory with idempotent state tracking.
No LLM integration exists yet. The `genai` crate (v0.5) is the
chosen multi-provider client (Ollama, OpenAI, Anthropic, +others).

## Dependency on existing code

- `crates/plaude-cli/src/commands/transcribe.rs` — produces transcript files consumed by summarization
- `crates/plaude-cli/src/commands/sync/` — will be extended with `--auto-summarize` (Phase 3)
- `crates/plaude-cli/src/main.rs` — `Commands` enum, dispatch (add new commands)
- `crates/plaud-domain/src/recording.rs` — `RecordingId` used to identify summaries
- `crates/plaud-auth/src/lib.rs` — config directory pattern reused for `llm.toml`

---

### L1 — `plaud-llm` crate: config loading and provider connection ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L1-llm-config-provider.md`](../journeys/JOURNEY-L1-llm-config-provider.md)
**Docs:** [`docs/usage/llm.md`](../../docs/usage/llm.md)

**Description:** Create the `plaud-llm` workspace crate with TOML-based
configuration and a `genai::Client` wrapper. The user can write
`~/.config/plaude/llm.toml` to set their model and optionally override
the provider endpoint / API key env var. With no config file, the crate
defaults to Ollama at `localhost:11434` with `llama3.2:3b`. Running
`plaude llm check` prints the resolved provider, model, and whether the
endpoint is reachable — giving the user immediate confirmation that
their LLM setup works before they try summarization.

**DoR (Definition of Ready):**
- Spec reviewed and approved
- `genai ^0.5` builds on the developer's system
- `toml ^0.8` available

**DoD (Definition of Done):**
- [x] `crates/plaud-llm/` created with `Cargo.toml`, `src/lib.rs`
- [x] `plaud-llm` added to workspace members in root `Cargo.toml`
- [x] `genai ^0.5` and `toml ^0.8` added to `[workspace.dependencies]`
- [x] `LlmConfig` struct: `model: String`, optional `provider.kind`, `provider.base_url`, `provider.api_key_env`
- [x] `LlmConfig::load(config_dir: &Path) -> Result<Self>` reads `llm.toml` with fallback to defaults
- [x] Env var overrides: `PLAUDE_LLM_MODEL` overrides `model` field
- [x] API key loaded from env var named in `provider.api_key_env` (never logged, never persisted)
- [x] `LlmProvider` struct wraps `genai::Client`, constructed from `LlmConfig`
- [x] `LlmProvider::check() -> Result<ProviderStatus>` — resolves provider, pings endpoint, returns model name + provider kind + reachable bool
- [x] `plaude llm check` command added to CLI — prints resolved config and connectivity
- [x] Unit tests: config parsing (full, minimal, missing file → defaults), invalid TOML error (6 tests)
- [x] Unit test: `LlmConfig::default()` returns Ollama + `llama3.2:3b`
- [x] E2e test: `plaude llm check` with no config prints default model (5 e2e tests)
- [x] `cargo build --all` and `cargo clippy --all` pass
- [x] Feature-gated: `llm` feature in `plaude-cli/Cargo.toml` (default-on)

**Files likely affected:**
- `Cargo.toml` (workspace members + deps)
- `crates/plaud-llm/Cargo.toml` (new)
- `crates/plaud-llm/src/lib.rs` (new)
- `crates/plaud-llm/src/config.rs` (new)
- `crates/plaud-llm/src/provider.rs` (new)
- `crates/plaude-cli/Cargo.toml` (feature + dep)
- `crates/plaude-cli/src/commands/llm.rs` (new — `check` subcommand)
- `crates/plaude-cli/src/commands/mod.rs` (add module)
- `crates/plaude-cli/src/main.rs` (add to `Commands` enum + dispatch)

---

### L2 — Template system: built-in and user-editable summarization prompts ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L2-template-system.md`](../journeys/JOURNEY-L2-template-system.md)

**Description:** Implement template loading with 5 built-in templates
compiled into the binary and a user override directory at
`~/.config/plaude/templates/`. Templates are Markdown files containing
a system prompt. Resolution order: user dir → built-in fallback.
`plaude summarize --list-templates` prints available templates.
`plaude summarize --export-template <name>` writes the built-in
to stdout so the user can pipe it to a file and customize.

**DoR (Definition of Ready):**
- L1 complete (crate exists, config loads)

**DoD (Definition of Done):**
- [x] `plaud-llm/src/template.rs` module with `Template` struct (name, system prompt text)
- [x] 5 built-in templates as `const &str`: `default`, `meeting-notes`, `action-items`, `key-decisions`, `brief`
- [x] `default` — structured summary with key points, action items, decisions (omit empty sections)
- [x] `meeting-notes` — meeting-oriented: attendees, agenda, discussion, outcomes
- [x] `action-items` — focused extraction: task, owner, deadline, priority
- [x] `key-decisions` — decision log: decision, context, alternatives considered, rationale
- [x] `brief` — 2-3 sentence executive summary, no structure
- [x] `TemplateRegistry::load(config_dir: &Path) -> Self` scans user dir + built-ins
- [x] `TemplateRegistry::get(name: &str) -> Option<&Template>` — user file overrides built-in of same name
- [x] `TemplateRegistry::list() -> Vec<TemplateInfo>` — name, source (built-in/user), first line preview
- [x] `--list-templates` prints table to stdout
- [x] `--export-template <name>` writes template content to stdout
- [x] Unit tests: resolution order (user overrides built-in), list includes both sources, unknown name returns None (13 tests)
- [x] Unit test: each built-in template is non-empty and valid Markdown
- [x] E2e test: `plaude summarize --list-templates` shows 5 templates (7 e2e tests)
- [x] E2e test: `plaude summarize --export-template default` outputs template content

**Files likely affected:**
- `crates/plaud-llm/src/template.rs` (new)
- `crates/plaud-llm/src/lib.rs` (re-export)
- `crates/plaude-cli/src/commands/summarize.rs` (new — list/export flags)
- `crates/plaude-cli/src/commands/mod.rs` (add module)
- `crates/plaude-cli/src/main.rs` (add to `Commands` enum)

---

### L3 — Transcript chunking with adaptive overlap ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L3-transcript-chunking.md`](../journeys/JOURNEY-L3-transcript-chunking.md)

**Description:** Long transcripts (1hr+ recordings) may exceed model
context windows. Implement a chunking strategy that splits transcripts
into overlapping windows, preserving speaker turns and sentence
boundaries. Chunk size adapts to a configurable token budget
(default 4096 tokens, ~3000 words). Overlap is 10% of chunk size.
This module is pure logic — no LLM calls — so it's independently
testable with deterministic inputs.

**DoR (Definition of Ready):**
- L1 complete (config provides token budget defaults)

**DoD (Definition of Done):**
- [x] `plaud-llm/src/chunk.rs` module with `Chunker` struct
- [x] `Chunker::new(max_tokens: usize, overlap_pct: f32)` — configurable
- [x] `Chunker::chunk(text: &str) -> Vec<Chunk>` — returns ordered chunks
- [x] `Chunk` struct: `index: usize`, `text: String`, `start_line: usize`, `end_line: usize`
- [x] Splitting heuristics: prefer paragraph breaks > sentence ends > speaker turn boundaries > word boundaries
- [x] Overlap: last N lines of previous chunk prepended to next chunk
- [x] Short transcripts (<= max_tokens): returned as single chunk, no splitting
- [x] Token estimation: 1 token ≈ 4 chars (simple heuristic, no tokenizer dependency)
- [x] Unit tests: single chunk for short text, multi-chunk for long text, overlap correctness, speaker turn preservation (14 tests)
- [x] Unit test: edge cases — empty input, single line, exact boundary
- [x] `cargo clippy` passes

**Files likely affected:**
- `crates/plaud-llm/src/chunk.rs` (new)
- `crates/plaud-llm/src/lib.rs` (re-export)

---

### L4 — Single-recording summarization with streaming output ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L4-summarization-pipeline.md`](../journeys/JOURNEY-L4-summarization-pipeline.md)

**Description:** Implement the core summarization pipeline: locate
transcript file → load template → chunk if needed → call LLM per
chunk → merge if multi-chunk → stream tokens to terminal → write
summary file. The command is `plaude summarize <path>` where `<path>`
is either a directory containing transcript files or a direct path
to a `.txt`/`.srt`/`.vtt`/`.json` transcript. Summary is written to
`<id>.summary.<template-name>.md` alongside the transcript.

**DoR (Definition of Ready):**
- L1 complete (provider connects)
- L2 complete (templates load)
- L3 complete (chunking works)

**DoD (Definition of Done):**
- [x] `plaud-llm/src/summarize.rs` — pipeline functions (discover, chunk, request, merge, write)
- [x] Pipeline: `load_transcript() → chunk() → per_chunk_request() → merge_if_needed() → stream_response()`
- [x] Transcript discovery: given dir, find `<id>.txt` or `<id>.json` (prefer json for speaker info)
- [x] If no transcript found: error message suggesting `plaude transcribe`
- [x] `ChatRequest` construction: system message = template, user message = transcript chunk
- [x] Multi-chunk merge: final LLM pass with "Merge these partial summaries into one" system prompt
- [x] Streaming: tokens printed to stderr in real-time via `exec_chat_stream()`
- [x] Summary output written atomically: `.tmp` → rename to `<id>.summary.<template>.md`
- [x] YAML front matter in summary file: `model`, `template`, `created_at`, `token_count`
- [x] `--template <name>` flag (default: `default`)
- [x] `--model <name>` flag overrides config file model
- [x] `--json` flag outputs structured metadata to stdout (model, template, tokens, duration)
- [x] `--no-stream` flag suppresses streaming output (for scripting)
- [x] Running again with same template overwrites existing summary
- [x] Post-summary hint: "Tip: try --template action-items for a focused summary"
- [x] Unit tests: transcript discovery, filename convention, front matter, atomic write (10 tests)
- [x] E2e test: missing transcript → error with `plaude transcribe` suggestion (10 e2e tests total)
- [x] `cargo clippy` and `cargo test --all` pass

**Files likely affected:**
- `crates/plaud-llm/src/summarize.rs` (new)
- `crates/plaud-llm/src/lib.rs` (re-export)
- `crates/plaude-cli/src/commands/summarize.rs` (new or extended)
- `crates/plaude-cli/src/main.rs` (dispatch)
- `crates/plaude-cli/tests/e2e_summarize.rs` (new)

---

### L5 — Summary management: list, show, delete ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L5-summary-management.md`](../journeys/JOURNEY-L5-summary-management.md)

**Description:** Add `plaude summaries` command group for managing
summaries. `list <path>` shows all summaries for a recording
(template name, model, date, size). `show <path> [--template <name>]`
prints summary content to stdout. `delete <path> --template <name>`
removes a specific summary file. All commands work with the
filesystem-based naming convention from L4.

**DoR (Definition of Ready):**
- L4 complete (summary files exist on disk with naming convention)

**DoD (Definition of Done):**
- [x] `plaude summaries list <path>` — scans for `<id>.summary.*.md` files, prints table: template, model (from front matter), created date, file size
- [x] `plaude summaries show <path>` — prints most recent summary (or `--template` specific one)
- [x] `plaude summaries show <path> --template <name>` — prints specific summary content to stdout
- [x] `plaude summaries delete <path> --template <name>` — deletes specific summary file, requires `--template` (no accidental mass delete)
- [x] `--json` flag on `list` outputs JSON array of summary metadata
- [x] No summaries found: helpful message "No summaries found. Run: plaude summarize <path>"
- [x] Parse YAML front matter from summary files for metadata display
- [x] E2e test: create summary (via fixture file), then list → shows entry (11 e2e tests total)
- [x] E2e test: show prints summary content
- [x] E2e test: delete removes file
- [x] E2e test: list with no summaries shows helpful message
- [x] `cargo clippy` passes

**Files likely affected:**
- `crates/plaude-cli/src/commands/summaries.rs` (new)
- `crates/plaude-cli/src/commands/mod.rs` (add module)
- `crates/plaude-cli/src/main.rs` (add to `Commands` enum + dispatch)
- `crates/plaude-cli/tests/e2e_summaries.rs` (new)

---

### L6 — Transcript auto-correction via LLM ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L6-transcript-correction.md`](../journeys/JOURNEY-L6-transcript-correction.md)

**Description:** Add `plaude correct <path>` that sends a transcript
through the LLM to fix common speech-to-text errors: misheard
technical terms, broken punctuation, run-on sentences, filler word
removal, proper noun capitalization. Output is a new file
`<id>.corrected.txt` — the original transcript is never modified.
Supports optional `--glossary <file>` with domain-specific terms
injected into the correction prompt.

**DoR (Definition of Ready):**
- L1 complete (provider connects)
- L3 complete (chunking works for long transcripts)

**DoD (Definition of Done):**
- [x] `plaud-llm/src/correct.rs` — correction prompt builder, glossary loading, pipeline
- [x] Built-in correction system prompt: fix punctuation, proper nouns, filler words, run-on sentences; preserve meaning
- [x] `--glossary <file>` — plain text file with one term per line, injected into system prompt as "known terms"
- [x] Chunking reused from L3 for long transcripts
- [x] Per-chunk correction → concatenate results (no merge pass needed — corrections are local)
- [x] Output written to `<id>.corrected.txt` atomically
- [x] Streaming output to stderr during correction
- [x] `--model <name>` override
- [x] Original transcript never modified
- [x] Unit test: correction prompt includes glossary terms when provided (8 unit tests)
- [x] E2e test: `plaude correct` help, missing file, no args, missing glossary (4 e2e tests)
- [x] `cargo clippy` and `cargo test --all` pass

**Files likely affected:**
- `crates/plaud-llm/src/correct.rs` (new)
- `crates/plaud-llm/src/lib.rs` (re-export)
- `crates/plaude-cli/src/commands/correct.rs` (new)
- `crates/plaude-cli/src/commands/mod.rs` (add module)
- `crates/plaude-cli/src/main.rs` (add to `Commands` enum + dispatch)
- `crates/plaude-cli/tests/e2e_correct.rs` (new)

---

### L7 — Batch summarization for multiple recordings ✅ CLOSED

**Journey:** [`specs/journeys/JOURNEY-L7-batch-summarization.md`](../journeys/JOURNEY-L7-batch-summarization.md)

**Description:** Extend `plaude summarize` to accept a directory and
process all recordings that have transcripts but lack summaries for
the specified template. This enables workflows like
`plaude sync ./recs && plaude summarize ./recs/` to summarize
everything new. Progress shows per-recording status.

**DoR (Definition of Ready):**
- L4 complete (single-recording summarization works)
- L5 complete (can list existing summaries to determine what's missing)

**DoD (Definition of Done):**
- [x] `plaude summarize <dir>/` (directory detection) triggers batch mode
- [x] Scans for transcript files, skips recordings that already have a summary for the given template
- [x] Per-recording progress: `[3/12] Summarizing 1712345678... done (1.2s)`
- [x] `--force` flag re-summarizes even if summary exists
- [x] `--dry-run` flag lists recordings that would be summarized without calling LLM
- [x] Summary of batch: "Summarized 8 recordings, skipped 4 (already summarized)"
- [x] On LLM error for one recording: log error, continue to next (don't abort batch)
- [x] E2e tests: dry-run lists files, empty dir, skips summarized, force includes all (4 batch e2e tests, 13 total)
- [x] Unit tests: find_all_transcripts, summary_exists (4 new, 14 total in summarize module)
- [x] `cargo clippy` passes

**Files likely affected:**
- `crates/plaude-cli/src/commands/summarize.rs` (extend with batch mode)
- `crates/plaud-llm/src/summarize.rs` (add `summarize_batch()`)
- `crates/plaude-cli/tests/e2e_summarize.rs` (extend)

---

### L8 — Auto-summarize on sync

**Description:** Add `--auto-summarize` flag to `plaude sync` that
automatically summarizes new recordings after pulling them. The chain
is: pull recording → transcribe (if transcript missing and
`--auto-transcribe` also set) → summarize with configured default
template. This is the "set it and forget it" workflow for power users.

**DoR (Definition of Ready):**
- L4 complete (summarization works)
- L7 complete (batch summarization works)
- Sync command works (existing)
- Transcribe command works (existing)

**DoD (Definition of Done):**
- [ ] `--auto-summarize` flag on `plaude sync`
- [ ] After successful pull of new recordings: run summarization pipeline on each
- [ ] Uses default template from `llm.toml` config
- [ ] `--auto-transcribe` flag: if transcript missing, run `plaude transcribe` first (requires `transcribe` feature)
- [ ] Summary status in sync output: "Synced 5 recordings, transcribed 3, summarized 3"
- [ ] LLM failure doesn't abort sync — recording is still pulled, summarization failure logged as warning
- [ ] Summary freshness: if transcript is newer than summary, re-summarize
- [ ] E2e test: sync with `--auto-summarize` and fixture LLM → summary files created
- [ ] E2e test: LLM failure → recordings still synced, warning printed
- [ ] `cargo clippy` and `cargo test --all` pass

**Files likely affected:**
- `crates/plaude-cli/src/commands/sync/mod.rs` (add flags + post-sync pipeline)
- `crates/plaude-cli/src/commands/sync/state.rs` (optional: track summary metadata)
- `crates/plaude-cli/tests/e2e_sync.rs` (extend)

---

## Dependency graph

```
L1 (plaud-llm crate + config + provider)
├──► L2 (template system)
│    └──► L4 (summarization pipeline) ◄── L3
│         ├──► L5 (summary management)
│         │    └──► L7 (batch summarization)
│         │         └──► L8 (auto-summarize on sync)
│         └──► L6 (transcript correction) ◄── L3
└──► L3 (chunking)
```

L2 and L3 can be done in parallel after L1.
L4 depends on L1 + L2 + L3.
L5 and L6 can be done in parallel after L4 (L6 only needs L1 + L3, but L4 validates the full pipeline).
L7 depends on L4 + L5.
L8 depends on L7.

## Out of scope (future — see Spec §8 Phase 4)

- Summary full-text search across all recordings
- Summary comparison/diff between templates or models
- Export formats: HTML, PDF
- Mind map generation (Mermaid diagram template)
- Recording Q&A (`plaude ask <id> "question"`)
- Config profiles (`--profile work`)
- GPU-accelerated inference configuration
