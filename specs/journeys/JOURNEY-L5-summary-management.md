# Journey L5: Summary Management — List, Show, Delete

**Roadmap item:** L5 — Summary management: list, show, delete
**Spec:** `specs/llm-features/SPEC.md` §3

## Persona

**Dmitriy** — has multiple summaries per recording from different
templates. Needs to browse, view, and clean up summaries via CLI.

## Trigger

User wants to see what summaries exist for a recording, read one,
or remove an outdated summary.

## Phases

### Phase 1: List Summaries
- **Action:** `plaude summaries list ./recordings/`
- **System:** Scans for `*.summary.*.md` files, parses front matter
- **Success:** Table with template, model, date, size
- **Pain:** No summaries → helpful "Run: plaude summarize" message

### Phase 2: Show Summary
- **Action:** `plaude summaries show ./recordings/ --template default`
- **System:** Finds matching summary file, prints content to stdout
- **Success:** Full summary text displayed
- **Pain:** Wrong template name → error with available list

### Phase 3: Delete Summary
- **Action:** `plaude summaries delete ./recordings/ --template default`
- **System:** Removes matching summary file
- **Success:** File deleted, confirmation message
- **Pain:** `--template` required (no accidental mass delete)

## Tests

### E2E Tests
- `summaries_list_with_fixture_shows_entry`
- `summaries_list_empty_shows_hint`
- `summaries_show_prints_content`
- `summaries_show_missing_template_errors`
- `summaries_delete_removes_file`
- `summaries_delete_requires_template`
- `summaries_list_json_output`
- `summaries_help_exits_zero`

## Implementation

### Files Created
- `crates/plaude-cli/src/commands/summaries.rs` — `SummariesCommand` (list/show/delete), front matter parsing, summary file discovery, metadata extraction
- `crates/plaude-cli/tests/e2e_summaries.rs` — 11 E2E tests

### Files Modified
- `crates/plaude-cli/src/commands/mod.rs` — added `summaries` module (feature-gated)
- `crates/plaude-cli/src/main.rs` — added `Summaries` to `Commands` enum + dispatch
