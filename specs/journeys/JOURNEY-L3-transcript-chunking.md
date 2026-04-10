# Journey L3: Transcript Chunking with Adaptive Overlap

**Roadmap item:** L3 — Transcript chunking with adaptive overlap
**Spec:** `specs/llm-features/SPEC.md` §4

## Persona

**Dmitriy** — records 1hr+ meetings, transcripts exceed LLM context
windows. Needs chunking that preserves coherent speaker turns and
sentence boundaries for quality summarization.

## Trigger

The summarization pipeline (L4) will call `Chunker::chunk()` before
sending transcript text to the LLM. This module is used internally —
no direct CLI surface.

## Phases

### Phase 1: Short Transcript (fits in context)
- **Input:** Transcript < max_tokens
- **Output:** Single chunk containing entire text
- **Invariant:** No splitting, no data loss

### Phase 2: Long Transcript (needs splitting)
- **Input:** Transcript > max_tokens
- **Output:** Multiple ordered chunks with overlap
- **Invariant:** Chunks cover entire text; overlap preserves context

### Phase 3: Boundary Awareness
- **Input:** Transcript with paragraph/sentence/speaker boundaries
- **Output:** Chunks split at natural boundaries, not mid-sentence
- **Invariant:** Prefer paragraph > sentence > speaker turn > word

## Design

- Token estimation: 1 token ≈ 4 chars (no tokenizer dep)
- Default: 4096 tokens (~16384 chars), 10% overlap
- Overlap is line-based: last N lines of previous chunk prepend next
- Empty input → empty Vec

## Tests

### Unit Tests (plaud-llm/src/chunk.rs)
- `empty_input_returns_empty` — No chunks from empty string
- `short_text_returns_single_chunk` — Text under budget → 1 chunk
- `single_chunk_covers_all_lines` — start_line=0, end_line=last
- `long_text_produces_multiple_chunks` — Text over budget → >1 chunks
- `chunks_are_ordered_by_index` — Indices 0, 1, 2...
- `overlap_present_between_chunks` — Last lines of chunk N appear in chunk N+1
- `all_lines_covered` — Union of chunk ranges covers input
- `splits_at_paragraph_boundary` — Prefers blank-line breaks
- `splits_at_sentence_boundary` — Falls back to sentence ends
- `exact_boundary_no_extra_chunk` — Text exactly at budget → 1 chunk

## Implementation

### Files Created
- `crates/plaud-llm/src/chunk.rs` — `Chunker`, `Chunk` types, splitting heuristics (paragraph > sentence > speaker > word), overlap, 14 unit tests

### Files Modified
- `crates/plaud-llm/src/lib.rs` — added `pub mod chunk`
