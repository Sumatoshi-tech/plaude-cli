# Journey L2: Template System for Summarization Prompts

**Roadmap item:** L2 — Template system: built-in and user-editable summarization prompts
**Spec:** `specs/llm-features/SPEC.md` §4

## Persona

**Dmitriy** — privacy-first power user. Has plaude-cli with LLM
configured (L1). Wants structured summaries of recordings using
different prompt templates depending on context (meeting, quick
review, action tracking).

## Trigger

User wants to explore available templates before summarizing, or
customize a template to match their workflow.

## Phases

### Phase 1: Discover Templates
- **Action:** User runs `plaude summarize --list-templates`
- **System:** Loads built-in templates, scans user config dir for overrides
- **Success:** Table printed: name, source (built-in/user), first-line preview
- **Pain:** No templates dir exists → still works, shows built-ins only

### Phase 2: Export & Customize
- **Action:** `plaude summarize --export-template default > ~/.config/plaude/templates/my-standup.md`
- **System:** Writes built-in template content to stdout
- **Success:** User can edit the file and use it as custom template
- **Pain:** Unknown template name → clear error listing available names

### Phase 3: Use Custom Template
- **Action:** `plaude summarize --list-templates` (after creating custom file)
- **System:** Shows both built-in and user templates, user overrides of same name indicated
- **Success:** Custom template appears with "user" source

## Friction Map

| Friction | Phase | Opportunity |
|----------|-------|-------------|
| User doesn't know templates exist | 1 | `plaude summarize --help` mentions `--list-templates` |
| Template dir doesn't exist | 1 | Works fine, only built-ins shown |
| Unknown template name | 2 | Error lists available names |
| User template overrides built-in | 3 | `list` shows source column clearly |

## Tests

### Unit Tests (plaud-llm/src/template.rs)
- `builtin_count_is_five` — Exactly 5 built-in templates
- `builtin_names` — Names are: default, meeting-notes, action-items, key-decisions, brief
- `builtins_are_nonempty` — Every built-in body is non-empty
- `get_unknown_returns_none` — Unknown name → None
- `get_builtin_returns_template` — Known name → Some(template)
- `user_overrides_builtin` — User file with same name wins
- `user_template_added_to_list` — User-only template appears in list
- `list_includes_all_sources` — List includes both built-in and user

### E2E Tests (plaude-cli)
- `summarize_list_templates_shows_five` — `--list-templates` prints 5 rows
- `summarize_export_template_default` — `--export-template default` outputs content
- `summarize_export_unknown_template_fails` — Unknown name → error

## Implementation

### Files Created
- `crates/plaud-llm/src/template.rs` — `Template`, `TemplateInfo`, `TemplateRegistry`, 5 built-in template bodies, user override loading
- `crates/plaude-cli/src/commands/summarize.rs` — `plaude summarize --list-templates` and `--export-template` handlers
- `crates/plaude-cli/tests/e2e_summarize.rs` — 7 E2E tests

### Files Modified
- `crates/plaud-llm/src/lib.rs` — added `pub mod template`
- `crates/plaude-cli/src/commands/mod.rs` — added `summarize` module (feature-gated)
- `crates/plaude-cli/src/main.rs` — added `Summarize` to `Commands` enum + dispatch
