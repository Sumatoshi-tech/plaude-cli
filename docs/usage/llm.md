# LLM Integration

plaude-cli integrates with LLM providers for recording summarization
and transcript post-processing. Supported providers: Ollama (local,
default), OpenAI, Anthropic, and any OpenAI-compatible endpoint.

## Quick Start

If Ollama is running locally, no configuration is needed:

```bash
plaude llm check        # Verify connectivity
```

## Configuration

Create `~/.config/plaude/llm.toml`:

```toml
# Model identifier — provider auto-detected from name prefix
model = "llama3.2:3b"         # Ollama (default)
# model = "gpt-4o-mini"       # OpenAI
# model = "claude-sonnet-4-5-20250514"  # Anthropic

# Optional: override for custom endpoints (LM Studio, vLLM, etc.)
# [provider]
# kind = "openai"
# base_url = "http://localhost:1234/v1"
# api_key_env = "MY_LLM_KEY"  # env var name, not the key itself
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `PLAUDE_LLM_MODEL` | Override the configured model name |
| `OPENAI_API_KEY` | API key for OpenAI models |
| `ANTHROPIC_API_KEY` | API key for Anthropic models |

## Commands

### `plaude llm check`

Print the resolved provider configuration and test connectivity.

```bash
$ plaude llm check
Model:    llama3.2:3b
Provider: auto-detect (Ollama fallback)
Status:   reachable
```

### `plaude template` — Manage Templates

List, create, edit, and delete summarization prompt templates.

```bash
# List all templates (built-in + user)
plaude template list

# Show a template's content
plaude template show default

# Create a new template (blank starter)
plaude template add my-standup

# Create from a built-in as starting point
plaude template add my-standup --from meeting-notes

# Edit in $EDITOR
plaude template edit my-standup

# Delete a user template
plaude template delete my-standup
```

5 built-in templates are included: `default`, `meeting-notes`,
`action-items`, `key-decisions`, `brief`. User templates live in
`~/.config/plaude/templates/` as `.md` files. A user template with
the same name as a built-in overrides it.

### `plaude summarize <path>`

Summarize a recording transcript using an LLM:

```bash
# Summarize a transcript file
plaude summarize ./recordings/1712345678.txt

# Summarize from a directory (auto-discovers transcript)
plaude summarize ./recordings/

# Use a specific template
plaude summarize ./recordings/1712345678.txt --template action-items

# Override the model
plaude summarize ./recordings/1712345678.txt --model gpt-4o-mini

# Suppress streaming output (for scripting)
plaude summarize ./recordings/1712345678.txt --no-stream

# JSON metadata output
plaude summarize ./recordings/1712345678.txt --json
```

The summary is written alongside the transcript as
`<id>.summary.<template>.md` with YAML front matter containing model,
template, timestamp, and token count.

Long transcripts are automatically chunked to fit within the model's
context window, with partial summaries merged into a final result.

**Batch mode:** Pass a directory to summarize all transcripts:

```bash
# Summarize all new recordings in a directory
plaude summarize ./recordings/

# Preview what would be summarized
plaude summarize ./recordings/ --dry-run

# Force re-summarize even if summaries exist
plaude summarize ./recordings/ --force
```

Batch mode automatically skips recordings that already have a summary
for the specified template. Progress is shown per-recording with
timing. Errors on individual recordings don't abort the batch.

### `plaude summaries list <path>`

List all summaries for a recording:

```bash
$ plaude summaries list ./recordings/
TEMPLATE             MODEL                CREATED                        SIZE
--------             -----                -------                        ----
action-items         llama3.2:3b          2026-04-08T10:30:00+00:00     1.2 KB
default              llama3.2:3b          2026-04-08T10:00:00+00:00     2.4 KB

2 summary(ies) found
```

Use `--json` for machine-readable output.

### `plaude summaries show <path> [--template <name>]`

Display a summary:

```bash
# Show the most recent summary
plaude summaries show ./recordings/

# Show a specific template's summary
plaude summaries show ./recordings/ --template action-items
```

### `plaude summaries delete <path> --template <name>`

Remove a specific summary (requires `--template` to prevent accidental mass delete):

```bash
plaude summaries delete ./recordings/ --template default
```

### `plaude correct <path>`

Fix speech-to-text errors in a transcript using an LLM. The original
file is never modified — output goes to `<id>.corrected.txt`:

```bash
# Basic correction
plaude correct ./recordings/1712345678.txt

# With a domain glossary (one term per line)
plaude correct ./recordings/1712345678.txt --glossary ./terms.txt

# Override model
plaude correct ./recordings/1712345678.txt --model gpt-4o-mini
```

The correction prompt fixes: punctuation, capitalization, filler words
(um, uh, like), run-on sentences, and obvious speech recognition errors.
Speaker labels are preserved.

## Troubleshooting

**"Status: unreachable"** — Check that Ollama is running:
```bash
ollama serve     # Start Ollama
ollama pull llama3.2:3b   # Download the default model
```

**Using a cloud provider** — Set the appropriate API key:
```bash
export OPENAI_API_KEY="sk-..."
echo 'model = "gpt-4o-mini"' > ~/.config/plaude/llm.toml
plaude llm check
```
