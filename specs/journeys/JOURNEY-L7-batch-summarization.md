# Journey L7: Batch Summarization for Multiple Recordings

**Roadmap item:** L7 — Batch summarization for multiple recordings
**Spec:** `specs/llm-features/SPEC.md` §8

## Persona

**Dmitriy** — syncs many recordings, wants to summarize all new ones
in one command rather than running `plaude summarize` per file.

## Trigger

User runs `plaude summarize ./recordings/` (directory path) and the
system processes all transcripts that lack summaries.

## Phases

### Phase 1: Scan Directory
- **Action:** `plaude summarize ./recordings/`
- **System:** Finds all transcript files, checks for existing summaries
- **Success:** Lists N transcripts to summarize, M already done

### Phase 2: Batch Process
- **Action:** System iterates through transcripts
- **System:** Per-recording progress, continues on error
- **Success:** All new recordings summarized

### Phase 3: Dry Run
- **Action:** `plaude summarize ./recordings/ --dry-run`
- **System:** Lists what would be summarized without LLM calls
- **Success:** User sees plan before committing

## CLI Additions

```bash
plaude summarize ./recordings/          # Batch: summarize all new
plaude summarize ./recordings/ --force  # Re-summarize even if exists
plaude summarize ./recordings/ --dry-run # List without calling LLM
```

## Tests

### Unit Tests (plaud-llm/src/summarize.rs)
- `find_transcripts_in_dir` — finds all transcript files
- `filter_unsummarized` — skips files with existing summaries

### E2E Tests
- `summarize_batch_dry_run_lists_files`
- `summarize_batch_empty_dir_no_error`
- `summarize_batch_skips_already_summarized`

## Implementation

### Files Created
- (none — extended existing files)

### Files Modified
- `crates/plaud-llm/src/summarize.rs` — added `find_all_transcripts()`, `summary_exists()`, 4 new unit tests (14 total)
- `crates/plaude-cli/src/commands/summarize.rs` — added `--force`, `--dry-run` flags, batch mode (`summarize_batch()`), refactored single-file into `summarize_single()`
- `crates/plaude-cli/tests/e2e_summarize.rs` — added 4 batch tests (13 total)
