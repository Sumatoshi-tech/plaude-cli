# SPEC: LLM Integration — Summarization, Transcript Post-Processing & AI Features

## 1. Summary

Add multi-provider LLM integration to plaude-cli enabling: recording summarization with customizable templates, transcript auto-correction, smart auto-summarization on sync, multiple summaries per recording, and full CLI management of summaries. The system targets privacy-conscious users who already chose plaude-cli to avoid cloud lock-in — it must work fully offline (Ollama) while also supporting cloud providers (OpenAI, Anthropic, and any OpenAI-compatible endpoint) for users who want that option.

## 2. Background & Research

### Market Context

| Product | Summarization | Templates | Multi-summary | Offline | Provider flexibility |
|---------|--------------|-----------|---------------|---------|---------------------|
| **Plaud Note app** | Yes — GPT-5.2, Claude Sonnet 4.5, Gemini 3 Pro | 10,000+ professional templates by role/industry | No (1 summary per recording) | No — cloud only, requires subscription | No — vendor-locked |
| **summarize.sh** | Yes — CLI + Chrome extension | Basic length/language control | No | Yes (local models) | Yes — any LLM via `--cli` flag |
| **Meetily** | Yes — meeting summarization | Structured output (action items, decisions) | No | Yes (Ollama + Whisper.cpp) | Yes — Ollama, OpenAI |
| **martinopiaggi/summarize** | Yes — video/audio summarization | YAML config for prompt customization | No | Yes (Ollama) | Yes — Groq, Gemini, DeepSeek, OpenRouter, Ollama |

**Key takeaways:**
- The official Plaud app charges for AI features and locks users into cloud processing — this is the core pain point plaude-cli can solve.
- No existing tool supports multiple summaries per recording with different templates — this is a differentiation opportunity.
- Template systems in existing tools are either non-existent or simplistic; the Plaud app's 10,000+ templates are marketing fluff — most users need 3-5 good templates they can customize.
- Offline-first with optional cloud is the winning position for our user base.

### Technical Context — Rust LLM Client Libraries

| Crate | Providers | Streaming | Maturity | Notes |
|-------|-----------|-----------|----------|-------|
| **genai** (0.5.x) | OpenAI, Anthropic, Gemini, Ollama, Groq, DeepSeek, Cohere, +7 more | Yes | High — active development, v0.5 Jan 2026 | Best fit: ergonomic, multi-provider, auto model→provider mapping |
| **llm** (graniet) | OpenAI, Anthropic, Ollama, +8 more | Yes | Medium — ambitious scope, newer | Heavy: agent framework, voice, too much for our needs |
| **async-llm** | OpenAI-compatible only | Yes | Medium | Limited to OpenAI protocol; no native Anthropic |
| **llm-connector** | 11+ providers | Yes | Medium | Multi-modal focus, heavier API surface |
| **ollama-rs** | Ollama only | Yes | High | Too narrow — single provider |

**Winner: `genai` crate.**

Reasoning:
1. Native support for all our target providers (Ollama, OpenAI, Anthropic) plus any OpenAI-compatible endpoint via custom resolver
2. Automatic model→provider detection (model name prefix `gpt-*` → OpenAI, `claude-*` → Anthropic, fallback → Ollama)
3. Clean API: `ChatRequest` + `ChatMessage` builder → `client.exec_chat()` or `client.exec_chat_stream()`
4. Auth via env vars (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) or custom `AuthResolver` — composable with our existing config system
5. Lightweight: `reqwest` + `tokio` (already in our dep tree) — no heavy framework overhead
6. Active maintenance with unified streaming engine as of v0.5

### Deep Dives

**Summarization prompt engineering best practices (from Gladia, Bliro, idratherbewriting):**
- Structured templates with explicit sections (summary, action items, decisions, key takeaways) outperform generic "summarize this" prompts
- Context injection matters: providing meeting type, participants, and domain terminology significantly improves output
- Multiple targeted prompts (theme extraction → per-theme detail) produce richer output than single-pass summarization
- Format specification (bullet points, tables, Markdown) controls output quality

**Plaud app's template approach:**
- Templates are categorized by profession (engineer, lawyer, doctor, PM, etc.) and meeting type (1:1, standup, interview, lecture)
- Each template is essentially a system prompt with structured output instructions
- Mind map generation is a post-processing step on the summary, not the transcript
- Users can create custom templates — essentially custom system prompts

**Transcript auto-correction patterns:**
- LLMs excel at fixing: misheard technical terms, proper nouns, acronyms, punctuation, run-on sentences, filler word removal
- Best approach: provide a domain glossary + the raw transcript → LLM returns corrected version
- Chunking required for long transcripts (context window limits) — overlap windows prevent boundary artifacts

## 3. Proposal

### Approach

Introduce a new `plaud-llm` crate (workspace member) that wraps `genai` and provides the LLM abstraction layer. Add three new CLI command groups: `plaude summarize`, `plaude summaries`, and `plaude correct`. Extend `plaude sync` with `--auto-summarize` flag. Store summaries alongside recordings as `<id>.summary.<template>.<n>.md` files, enabling multiple summaries per recording with different templates.

### Key Decisions

| # | Decision | Choice | Reasoning | Alternatives considered |
|---|----------|--------|-----------|------------------------|
| 1 | LLM client library | `genai` crate | Multi-provider, ergonomic, lightweight, auto model→provider mapping, active maintenance | `llm` (too heavy, agent framework), `async-llm` (OpenAI-only), `ollama-rs` (single provider), raw `reqwest` (reinventing the wheel) |
| 2 | Provider configuration | Config file (`~/.config/plaude/llm.toml`) + env var overrides | Persistent config for daily use, env vars for CI/scripting, follows existing plaude patterns | CLI flags only (too verbose for repeated use), env vars only (no persistence) |
| 3 | Template storage | User-editable files in `~/.config/plaude/templates/` + built-in defaults compiled into binary | Users can customize, share, and version-control templates; built-ins provide zero-config experience | Database (overkill), YAML config (less ergonomic than separate files), embedded only (not customizable) |
| 4 | Summary storage format | Markdown files alongside recordings: `<id>.summary.<template-name>.md` | Human-readable, grep-friendly, works with existing sync directory, no database needed | SQLite (overhead, not human-readable), JSON (less readable), single file per recording (limits multi-summary) |
| 5 | Transcript chunking strategy | Sliding window with overlap, chunk size adaptive to model context window | Handles arbitrarily long transcripts without losing context at boundaries | Fixed chunks (boundary artifacts), full transcript only (context window limits), map-reduce (complexity) |

### ML (Minimum Loveable)

**IN:**
- `plaude summarize <recording-dir>/<id>` — summarize a single recording's transcript using default or specified template
- `plaude summarize --template <name>` — use a specific template
- `plaude summaries list <id>` — list all summaries for a recording
- `plaude summaries show <id> [--template <name>]` — display a summary
- `plaude summaries delete <id> --template <name>` — remove a summary
- LLM provider configuration via `~/.config/plaude/llm.toml`
- 5 built-in templates: `default`, `meeting-notes`, `action-items`, `key-decisions`, `brief`
- Ollama + OpenAI + Anthropic provider support
- Streaming output to terminal during summarization

**OUT (for ML, in scope for later phases):**
- `plaude correct` (transcript auto-correction) — Phase 2
- `plaude sync --auto-summarize` — Phase 3
- Mind map generation — Phase 4
- Custom vocabulary/glossary injection — Phase 2
- Batch summarization of multiple recordings — Phase 2
- Summary search/query across all recordings — Phase 4
- Summary export formats (PDF, HTML) — Phase 4

### Anti-Goals

- **No RAG / embedding / vector DB.** Plaude-cli is a recording tool, not a knowledge base. Summaries are plain text files.
- **No built-in model hosting.** We point to Ollama or cloud APIs — we don't bundle or manage models ourselves.
- **No real-time / streaming transcription+summarization.** Summarization is a post-processing step on completed transcripts.
- **No web UI or TUI for summary browsing.** CLI + filesystem is the interface; users can use their preferred editor/viewer.
- **No "AI chat with your recordings" feature.** This is a summarization tool, not a conversational agent over your data.

## 4. Technical Design

### Architecture

```
plaude-cli
├── plaud-llm (new crate)          # LLM abstraction layer
│   ├── provider.rs                # genai Client wrapper, config loading
│   ├── config.rs                  # LlmConfig: provider, model, api_key, base_url, defaults
│   ├── template.rs                # Template loading: built-in + user directory
│   ├── summarize.rs               # Summarization pipeline: load transcript → chunk → prompt → collect
│   ├── correct.rs                 # Transcript correction pipeline (Phase 2)
│   └── chunk.rs                   # Transcript chunking with overlap
├── commands/
│   ├── summarize.rs (new)         # `plaude summarize` command
│   ├── summaries.rs (new)         # `plaude summaries list|show|delete` command
│   └── sync/mod.rs (modified)     # --auto-summarize flag (Phase 3)
```

**Data flow — summarization:**
```
1. User: `plaude summarize ./recordings/1712345678`
2. CLI: Locate transcript file: `1712345678.txt` (or .srt/.vtt/.json)
3. plaud-llm: Load LLM config from ~/.config/plaude/llm.toml
4. plaud-llm: Load template (built-in default or user-specified)
5. plaud-llm: Chunk transcript if needed (adaptive to model context window)
6. plaud-llm: For each chunk → ChatRequest(system=template, user=chunk)
7. plaud-llm: If multi-chunk → final merge pass with all chunk summaries
8. plaud-llm: Stream response to terminal
9. CLI: Write summary to `1712345678.summary.default.md`
```

**Configuration file (`~/.config/plaude/llm.toml`):**
```toml
# Default provider and model
model = "llama3.2:3b"                    # Ollama model (auto-detected)
# model = "gpt-4o-mini"                  # OpenAI (auto-detected by prefix)
# model = "claude-sonnet-4-5-20250514"  # Anthropic (auto-detected)

# Optional: override for OpenAI-compatible endpoints
# [provider]
# kind = "openai"
# base_url = "http://localhost:1234/v1"  # LM Studio, vLLM, etc.
# api_key_env = "MY_LLM_KEY"            # env var name for API key

# Summarization defaults
[summarize]
template = "default"
# max_tokens = 2048
# temperature = 0.3
```

**Template file format (`~/.config/plaude/templates/default.md`):**
```markdown
You are a precise summarization assistant. Given the transcript of a voice recording, produce a clear, structured summary.

## Output format

- **Summary**: 2-4 sentence overview of the recording
- **Key Points**: Bulleted list of the most important topics discussed
- **Action Items**: Any tasks, deadlines, or follow-ups mentioned (if applicable)
- **Decisions**: Any decisions made during the recording (if applicable)

## Rules

- Be concise. Prefer short sentences over long ones.
- Preserve technical terms, proper nouns, and numbers exactly as spoken.
- If speakers are identified, attribute key points to speakers.
- Omit filler words, false starts, and repetitions.
- If a section has no content (e.g., no action items), omit it entirely.
```

**Summary file naming convention:**
```
<recording-id>.summary.<template-name>.md

Examples:
  1712345678.summary.default.md
  1712345678.summary.meeting-notes.md
  1712345678.summary.action-items.md
```

### Non-Functional Requirements

- **Performance**: Summarization speed is bounded by the LLM provider. Local Ollama with 3B model: ~10-30s for a 30-min transcript. Cloud APIs: ~5-15s. The CLI must stream tokens to the terminal as they arrive — no waiting for full response.
- **Reliability**: Network errors to LLM providers must produce clear error messages with retry guidance. Partial responses (interrupted stream) must not corrupt existing summary files (write to temp, then rename).
- **Security**: API keys never logged or shown in `--help` output. Keys read from env vars at runtime, never persisted in config files (config stores env var *names*, not values). Transcript data sent to cloud providers only when user explicitly chooses a cloud model.
- **Observability**: `RUST_LOG=plaud_llm=debug` shows: provider selected, model used, token count, chunk count, latency. `--json` flag outputs structured metadata (model, template, tokens, duration).

### Testing Strategy

- **Unit**: Template loading and resolution (built-in → user override), config parsing, transcript chunking logic, summary file naming/discovery
- **Integration**: Mock LLM provider (record/replay HTTP fixtures) testing full summarization pipeline end-to-end through `plaud-llm`
- **E2E**: `plaude summarize` / `plaude summaries` commands against `plaud-sim` with pre-seeded transcript files. CI uses recorded HTTP fixtures (no live LLM calls).

### Migration & Compatibility

- No breaking changes. All new features are additive commands.
- Existing sync directories gain summary files alongside existing `.wav`/`.txt` files — backward compatible.
- The `plaud-llm` crate is feature-gated (`llm` feature, default-on). Users who don't want LLM deps can compile with `--no-default-features`.
- Config file (`llm.toml`) is optional — sensible defaults (Ollama, llama3.2:3b) work with zero config if Ollama is running.

### Dependencies

| Dependency | Purpose | Assessment |
|------------|---------|------------|
| **genai** ^0.5 | Multi-provider LLM client | Active, well-maintained, 245+ GitHub stars, native Ollama/OpenAI/Anthropic support, uses reqwest (already in our tree) |
| **toml** ^0.8 | Config file parsing | Standard Rust TOML parser, mature, already used widely in the ecosystem |

No other new dependencies required. `reqwest`, `tokio`, `serde`, `serde_json` already in workspace.

## 5. User Journey

### Persona

**Dmitriy** — privacy-conscious power user who bought a Plaud Note for meeting recordings but refuses to use the vendor cloud app. Uses plaude-cli to sync recordings and transcribe offline with Whisper. Has Ollama installed locally for development work. Wants to get quick summaries of recordings without leaving the terminal or sending data to the cloud.

### CJM Phases

**Phase 1: Discovery & Setup**
- User runs `plaude summarize` for the first time
- CLI detects no `llm.toml` config → checks if Ollama is reachable at localhost:11434
- If Ollama running: proceeds with default model, prints "Using Ollama (llama3.2:3b). Configure in ~/.config/plaude/llm.toml"
- If Ollama not running: prints clear error "No LLM provider configured. Install Ollama (https://ollama.com) or set model in ~/.config/plaude/llm.toml"
- Success: user gets their first summary in <30 seconds with zero config

**Phase 2: First Summarization**
- User: `plaude summarize ./recordings/1712345678`
- CLI locates `1712345678.txt` transcript, loads default template
- Tokens stream to terminal in real-time (user sees summary forming)
- Summary saved to `1712345678.summary.default.md`
- CLI prints: "Summary saved. View: plaude summaries show 1712345678"

**Phase 3: Template Exploration**
- User: `plaude summarize --list-templates` → sees built-in templates
- User: `plaude summarize ./recordings/1712345678 --template action-items`
- Gets a focused action-items-only summary alongside the existing default summary
- User: `plaude summaries list 1712345678` → sees both summaries

**Phase 4: Template Customization**
- User copies a built-in: `plaude summarize --export-template default > ~/.config/plaude/templates/my-standup.md`
- Edits the template in their favorite editor
- Uses it: `plaude summarize ./recordings/1712345678 --template my-standup`

**Phase 5: Daily Workflow (power user)**
- After `plaude sync ./recordings`, runs `plaude summarize ./recordings/1712345678` on the latest recording
- Eventually: `plaude sync ./recordings --auto-summarize` does it automatically (Phase 3 of implementation)

### Friction Map

| Friction | Phase | Opportunity |
|----------|-------|-------------|
| User doesn't have Ollama installed | Discovery | Clear error message with install link + guidance for cloud provider setup |
| User doesn't know which model to choose | Discovery | Sensible default (llama3.2:3b) + `plaude summarize --help` lists recommended models per provider |
| Long transcript takes >30s to summarize | First use | Streaming output keeps user engaged; progress indication for chunk processing |
| User doesn't know templates exist | First use | Post-summary hint: "Tip: try --template action-items for a focused summary" |
| Summary quality varies with model | Daily use | Document recommended models per use case in `--help` and README |
| User wants to re-summarize with different settings | Daily use | Running summarize again with same template overwrites; `--template` flag enables parallel summaries |

## 6. Risks & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| `genai` crate becomes unmaintained | High — need to switch LLM library | Low — active development, recent v0.5 release | Wrap genai behind our own `LlmProvider` trait in `plaud-llm`; swapping implementation doesn't affect CLI commands |
| Ollama not installed / not running | Medium — first-run failure | Medium — not all users have Ollama | Clear error messages, fallback guidance to cloud providers, document setup in README |
| Long transcripts exceed model context window | Medium — truncated or degraded summaries | High — common for 1hr+ recordings | Adaptive chunking with overlap + merge pass; document recommended models with large context windows |
| API key leakage in logs/config | High — security breach | Low — config stores env var names, not values | Never log API keys; use `Zeroizing` for key material in memory; config references env var names only |
| Summary quality inconsistency across providers | Medium — user frustration | Medium — different models have different strengths | Built-in templates tuned for general case; document model recommendations; temperature defaults to 0.3 for consistency |
| Transcript not available (user forgot to run `plaude transcribe`) | Low — user error | High — new users | `plaude summarize` auto-detects missing transcript and suggests `plaude transcribe` command |

## 7. Open Questions

1. **Should `plaude summarize` auto-transcribe if no transcript exists?** Adds convenience but couples two features. Recommendation: no for ML, yes for Phase 3 (auto-summarize in sync already implies this chain).
2. **Should summary metadata (model, template, timestamp, token count) be embedded in the summary file or a sidecar?** Recommendation: YAML front matter in the Markdown file — keeps it self-contained and human-readable.
3. **Should we support non-English templates?** Recommendation: templates are user-editable text files — users can write them in any language. Built-in templates are English only for ML.
4. **Token budget management — should we expose max_tokens config or auto-detect from model?** Recommendation: auto-detect from genai's model metadata where available, with user override in config.
5. **Should `plaude correct` produce a new file or modify in-place?** Recommendation: new file (`<id>.corrected.txt`) — non-destructive, original always preserved.

## 8. Implementation Roadmap

### Phase 1: Core LLM Integration (ML)
1. **`plaud-llm` crate** — genai wrapper, config loading, template system
2. **`plaude summarize` command** — single-recording summarization with template selection and streaming output
3. **`plaude summaries` command** — list, show, delete summaries for a recording
4. **5 built-in templates** — default, meeting-notes, action-items, key-decisions, brief
5. **Config system** — `~/.config/plaude/llm.toml` with sensible defaults

### Phase 2: Transcript Enhancement
6. **`plaude correct` command** — LLM-powered transcript auto-correction (fix misheard terms, punctuation, proper nouns)
7. **Custom vocabulary/glossary** — user-defined term list injected into correction and summarization prompts
8. **Batch operations** — `plaude summarize ./recordings/` processes all recordings with missing summaries
9. **Transcript chunking improvements** — speaker-aware chunk boundaries (don't split mid-speaker-turn)

### Phase 3: Automation
10. **`plaude sync --auto-summarize`** — automatically summarize new recordings after sync
11. **Auto-transcribe chain** — if transcript missing, run transcribe → then summarize
12. **Summary freshness** — detect when transcript is newer than summary, offer re-summarization
13. **Config profiles** — named config sets for different use cases (e.g., `plaude summarize --profile work`)

### Phase 4: Advanced Features
14. **Summary search** — `plaude summaries search "quarterly goals"` across all recordings
15. **Summary comparison** — diff two summaries of the same recording (different templates/models)
16. **Export formats** — `plaude summaries export --format html|pdf <id>`
17. **Mind map generation** — structured Markdown or Mermaid diagram output template
18. **Recording Q&A** — `plaude ask <id> "what was decided about the deadline?"` — focused extraction from transcript
