# Journey L1: LLM Config & Provider Connection

**Roadmap item:** L1 — `plaud-llm` crate: config loading and provider connection
**Spec:** `specs/llm-features/SPEC.md` §4

## Persona

**Dmitriy** — privacy-first power user running plaude-cli on Linux.
Has Ollama installed locally. Wants to verify LLM connectivity before
attempting summarization.

## Trigger

User wants to set up LLM integration for the first time, or verify
that their provider configuration is correct.

## Phases

### Phase 1: Zero-Config Discovery
- **Action:** User runs `plaude llm check` with no config file
- **System:** Loads defaults (Ollama, `llama3.2:3b`), attempts connection
- **Success:** Prints model, provider kind, reachability status
- **Pain:** Ollama not installed → clear error with install link

### Phase 2: Config File Setup
- **Action:** User creates `~/.config/plaude/llm.toml`
- **System:** Parses TOML, validates fields, merges with defaults
- **Success:** `plaude llm check` reflects configured model/provider
- **Pain:** Invalid TOML → clear parse error with line number

### Phase 3: Cloud Provider Setup
- **Action:** User sets `model = "gpt-4o-mini"` and `OPENAI_API_KEY` env var
- **System:** Auto-detects OpenAI from model prefix, loads key from env
- **Success:** `plaude llm check` shows OpenAI provider, reachable
- **Pain:** Missing API key → error naming the expected env var

### Phase 4: Custom Endpoint
- **Action:** User configures `[provider]` with `base_url` for LM Studio
- **System:** Uses custom endpoint instead of default provider URL
- **Success:** Connects to local LM Studio instance
- **Pain:** Wrong URL → connection refused with clear error

## Friction Map

| Friction | Phase | Opportunity |
|----------|-------|-------------|
| No Ollama installed | 1 | Error message with install URL |
| Invalid TOML syntax | 2 | Error with file path + line number |
| Missing API key | 3 | Name the env var to set |
| Wrong base_url | 4 | Show attempted URL in error |
| Unknown model prefix | 3 | Fallback to Ollama, log info |

## Tests

### Unit Tests (plaud-llm)
- `config_default_returns_ollama_model` — Default() produces llama3.2:3b
- `config_load_full_toml` — All fields parsed from valid TOML
- `config_load_minimal_toml` — Only model field, rest defaults
- `config_load_missing_file_returns_defaults` — No file → defaults
- `config_env_var_overrides_model` — PLAUDE_LLM_MODEL wins over file
- `config_invalid_toml_returns_error` — Garbled file → typed error

### E2E Tests (plaude-cli)
- `llm_check_no_config_prints_default_model` — Shows llama3.2:3b
- `llm_check_help_exits_zero` — `plaude llm --help` works

## Implementation

### Files Created
- `crates/plaud-llm/Cargo.toml` — new workspace crate
- `crates/plaud-llm/src/lib.rs` — crate root, re-exports `config` and `provider`
- `crates/plaud-llm/src/config.rs` — `LlmConfig`, `ProviderConfig`, `ConfigError`, TOML loading
- `crates/plaud-llm/src/provider.rs` — `LlmProvider` wrapping `genai::Client`, `ProviderStatus`, `check()`
- `crates/plaude-cli/src/commands/llm.rs` — `plaude llm check` command handler
- `crates/plaude-cli/tests/e2e_llm.rs` — 5 E2E tests
- `docs/usage/llm.md` — user documentation

### Files Modified
- `Cargo.toml` — added `plaud-llm` to workspace members, `genai` + `toml` to workspace deps
- `crates/plaude-cli/Cargo.toml` — added `plaud-llm` optional dep, `llm` feature (default-on)
- `crates/plaude-cli/src/commands/mod.rs` — added `llm` module (feature-gated)
- `crates/plaude-cli/src/main.rs` — added `Llm` to `Commands` enum + dispatch (feature-gated)
