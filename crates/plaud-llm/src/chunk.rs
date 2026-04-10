//! Transcript chunking with adaptive overlap.
//!
//! Splits long transcripts into overlapping windows that fit within a
//! model's context window. Splitting heuristics prefer natural
//! boundaries: paragraph breaks > sentence ends > speaker turns >
//! word boundaries.
//!
//! Journey: `specs/journeys/JOURNEY-L3-transcript-chunking.md`

/// Approximate characters per token. Simple heuristic — avoids a
/// tokenizer dependency while staying conservative.
const CHARS_PER_TOKEN: usize = 4;

/// Default token budget per chunk. Set high (32K) since modern LLMs
/// (Ollama, OpenAI, Anthropic) support 32K-128K context windows.
/// A 70-minute transcript is ~19K tokens — fits in one chunk.
const DEFAULT_MAX_TOKENS: usize = 32768;

/// Default overlap as a fraction of max tokens (10%).
const DEFAULT_OVERLAP_FRACTION: f32 = 0.10;

/// A single chunk of transcript text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Zero-based index of this chunk in the sequence.
    pub index: usize,
    /// The text content of this chunk.
    pub text: String,
    /// First line number (0-based) from the original text included.
    pub start_line: usize,
    /// Last line number (0-based, inclusive) from the original text.
    pub end_line: usize,
}

/// Splits transcript text into overlapping chunks that fit within a
/// token budget.
#[derive(Debug, Clone)]
pub struct Chunker {
    /// Maximum characters per chunk (derived from token budget).
    max_chars: usize,
    /// Number of overlap lines to prepend from the previous chunk.
    overlap_lines: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_TOKENS, DEFAULT_OVERLAP_FRACTION)
    }
}

impl Chunker {
    /// Create a chunker with a given token budget and overlap fraction.
    ///
    /// `overlap_pct` is clamped to `[0.0, 0.5]`.
    pub fn new(max_tokens: usize, overlap_pct: f32) -> Self {
        let max_chars = max_tokens.saturating_mul(CHARS_PER_TOKEN).max(1);
        let clamped = overlap_pct.clamp(0.0, 0.5);
        // Estimate overlap lines from the char budget — assume ~80
        // chars/line as a rough average for transcripts.
        let overlap_chars = (max_chars as f32 * clamped) as usize;
        let overlap_lines = (overlap_chars / 80).max(1);
        Self { max_chars, overlap_lines }
    }

    /// Split `text` into ordered chunks. Returns an empty `Vec` for
    /// empty input. Short texts that fit the budget return a single
    /// chunk.
    pub fn chunk(&self, text: &str) -> Vec<Chunk> {
        if text.is_empty() {
            return Vec::new();
        }

        let lines: Vec<&str> = text.lines().collect();

        // Fast path: entire text fits in budget.
        if text.len() <= self.max_chars {
            return vec![Chunk {
                index: 0,
                text: text.to_owned(),
                start_line: 0,
                end_line: lines.len().saturating_sub(1),
            }];
        }

        let mut chunks = Vec::new();
        let mut cursor = 0; // current line index

        while cursor < lines.len() {
            let (end, chunk_text) = self.find_chunk_end(&lines, cursor);
            chunks.push(Chunk {
                index: chunks.len(),
                text: chunk_text,
                start_line: cursor,
                end_line: end,
            });

            // Advance cursor past the chunk, then back up by overlap.
            // Guarantee forward progress: cursor must advance at least
            // one line beyond the previous start to prevent infinite loops.
            let next = end + 1;
            if next >= lines.len() {
                break;
            }
            cursor = next.saturating_sub(self.overlap_lines).max(cursor + 1);
        }

        chunks
    }

    /// Find the end line for a chunk starting at `start`, respecting
    /// the character budget and preferring natural break points.
    fn find_chunk_end(&self, lines: &[&str], start: usize) -> (usize, String) {
        let mut char_count = 0;
        let mut last_paragraph = None; // blank-line boundary
        let mut last_sentence = None; // line ending in .!?
        let mut last_speaker = None; // line starting with [Speaker

        for (i, line) in lines.iter().enumerate().skip(start) {
            // +1 for the newline between lines.
            let line_chars = line.len() + 1;

            if char_count + line_chars > self.max_chars && i > start {
                // We've exceeded the budget. Pick the best break point.
                let break_at = last_paragraph
                    .or(last_sentence)
                    .or(last_speaker)
                    .unwrap_or(i.saturating_sub(1).max(start));

                let text = join_lines(lines, start, break_at);
                return (break_at, text);
            }

            char_count += line_chars;

            // Track natural boundaries.
            if line.trim().is_empty() && i > start {
                last_paragraph = Some(i.saturating_sub(1));
            }
            let trimmed = line.trim();
            if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
                last_sentence = Some(i);
            }
            if (trimmed.starts_with("[Speaker") || trimmed.starts_with("[SPEAKER")) && i > start {
                last_speaker = Some(i.saturating_sub(1));
            }
        }

        // Remaining lines fit — take them all.
        let end = lines.len().saturating_sub(1);
        let text = join_lines(lines, start, end);
        (end, text)
    }
}

/// Join lines[start..=end] with newlines.
fn join_lines(lines: &[&str], start: usize, end: usize) -> String {
    lines[start..=end].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a chunker with small budget for testing.
    fn small_chunker(max_chars: usize) -> Chunker {
        // Pass max_chars directly as "tokens" with CHARS_PER_TOKEN=4,
        // so actual char budget = max_chars * 4. To get exact char
        // control, divide by CHARS_PER_TOKEN.
        Chunker {
            max_chars,
            overlap_lines: 1,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let c = Chunker::default();
        assert!(c.chunk("").is_empty());
    }

    #[test]
    fn short_text_returns_single_chunk() {
        let c = Chunker::default();
        let chunks = c.chunk("Hello world");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world");
        assert_eq!(chunks[0].index, 0);
    }

    #[test]
    fn single_chunk_covers_all_lines() {
        let c = Chunker::default();
        let text = "Line one\nLine two\nLine three";
        let chunks = c.chunk(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 0);
        assert_eq!(chunks[0].end_line, 2);
    }

    #[test]
    fn long_text_produces_multiple_chunks() {
        // 50-char budget, overlap 1 line.
        let c = small_chunker(50);
        let text = "AAAA BBBB CCCC DDDD\nEEEE FFFF GGGG HHHH\nIIII JJJJ KKKK LLLL\nMMMM NNNN OOOO PPPP";
        let chunks = c.chunk(text);
        assert!(chunks.len() > 1, "expected multiple chunks, got {}", chunks.len());
    }

    #[test]
    fn chunks_are_ordered_by_index() {
        let c = small_chunker(40);
        let text = "Line A.\nLine B.\nLine C.\nLine D.\nLine E.";
        let chunks = c.chunk(text);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn overlap_present_between_chunks() {
        let c = small_chunker(30);
        // Each line is ~8 chars + newline = ~9 chars.
        // Budget 30 chars → ~3 lines per chunk, overlap 1 line.
        let text = "Line AA.\nLine BB.\nLine CC.\nLine DD.\nLine EE.\nLine FF.";
        let chunks = c.chunk(text);
        assert!(chunks.len() >= 2, "need at least 2 chunks for overlap test");

        // The last line(s) of chunk 0 should appear at the start of chunk 1.
        let chunk0_last_line = chunks[0].text.lines().last().unwrap();
        let chunk1_first_line = chunks[1].text.lines().next().unwrap();
        assert_eq!(
            chunk0_last_line, chunk1_first_line,
            "overlap: last line of chunk 0 should be first line of chunk 1"
        );
    }

    #[test]
    fn all_original_lines_covered() {
        let c = small_chunker(40);
        let text = "A\nB\nC\nD\nE\nF\nG\nH\nI\nJ";
        let lines: Vec<&str> = text.lines().collect();
        let chunks = c.chunk(text);

        // Every line index must appear in at least one chunk's range.
        for (idx, _line) in lines.iter().enumerate() {
            let covered = chunks.iter().any(|ch| ch.start_line <= idx && idx <= ch.end_line);
            assert!(covered, "line {idx} not covered by any chunk");
        }
    }

    #[test]
    fn splits_at_paragraph_boundary() {
        // Budget that forces a split, with a blank line in the middle.
        let c = small_chunker(60);
        let text = "First paragraph line one.\nFirst paragraph line two.\n\nSecond paragraph line one.\nSecond paragraph line two.";
        let chunks = c.chunk(text);
        if chunks.len() > 1 {
            // First chunk should end before or at the blank line.
            let first_end = chunks[0].end_line;
            // The blank line is line 2 (0-indexed).
            assert!(
                first_end <= 2,
                "expected split at paragraph boundary, first chunk ends at line {first_end}"
            );
        }
    }

    #[test]
    fn splits_at_sentence_boundary() {
        let c = small_chunker(50);
        let text = "Start of text here.\nMiddle sentence ends here.\nContinued without period\nAnother line here.";
        let chunks = c.chunk(text);
        if chunks.len() > 1 {
            let last_line = chunks[0].text.lines().last().unwrap().trim();
            assert!(
                last_line.ends_with('.') || last_line.ends_with('!') || last_line.ends_with('?'),
                "expected chunk to end at sentence boundary, got: '{last_line}'"
            );
        }
    }

    #[test]
    fn exact_boundary_no_extra_chunk() {
        // Text that is exactly at the char budget → single chunk.
        let text = "A".repeat(DEFAULT_MAX_TOKENS * CHARS_PER_TOKEN);
        let c = Chunker::default();
        let chunks = c.chunk(&text);
        assert_eq!(chunks.len(), 1, "text at exact budget should be 1 chunk");
    }

    #[test]
    fn default_chunker_uses_expected_values() {
        let c = Chunker::default();
        assert_eq!(c.max_chars, DEFAULT_MAX_TOKENS * CHARS_PER_TOKEN);
        assert!(c.overlap_lines >= 1);
    }

    #[test]
    fn single_line_returns_single_chunk() {
        let c = Chunker::default();
        let chunks = c.chunk("Just one line");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 0);
        assert_eq!(chunks[0].end_line, 0);
    }

    #[test]
    fn overlap_clamp_high() {
        // overlap_pct > 0.5 is clamped.
        let c = Chunker::new(100, 0.9);
        // Should not panic, and overlap should be reasonable.
        assert!(c.overlap_lines >= 1);
    }

    #[test]
    fn overlap_clamp_low() {
        // overlap_pct < 0 is clamped to 0.
        let c = Chunker::new(100, -1.0);
        // With 0% overlap, overlap_lines is still at least 1 (the .max(1)).
        assert!(c.overlap_lines >= 1);
    }
}
