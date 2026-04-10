//! Transcript auto-correction pipeline.
//!
//! Sends transcript chunks through an LLM to fix speech-to-text
//! errors: punctuation, proper nouns, filler words, run-on sentences.
//! The original file is never modified — output goes to a new
//! `.corrected.txt` file.
//!
//! Journey: `specs/journeys/JOURNEY-L6-transcript-correction.md`

use std::path::{Path, PathBuf};

use genai::chat::{ChatMessage, ChatRequest};

use crate::chunk::Chunker;
use crate::provider::LlmProvider;

/// Built-in system prompt for transcript correction.
const CORRECTION_PROMPT: &str = "\
You are a transcript correction assistant. You are given a raw \
speech-to-text transcript that may contain errors. Fix the following \
issues while preserving the original meaning:

1. Fix punctuation: add missing periods, commas, and question marks.
2. Fix capitalization: proper nouns, sentence starts, acronyms.
3. Remove filler words: um, uh, like, you know, so, basically.
4. Fix run-on sentences: split overly long sentences.
5. Fix misheard words: correct obvious speech recognition errors.
6. Preserve speaker labels (e.g., [Speaker 1]) exactly as they appear.

Output ONLY the corrected transcript. Do not add commentary, \
explanations, or formatting beyond the corrections.";

/// Glossary injection prefix added to the system prompt.
const GLOSSARY_PREFIX: &str = "\n\nThe following domain-specific terms must be \
spelled exactly as shown when they appear in the transcript:\n";

/// Errors from the correction pipeline.
#[derive(Debug, thiserror::Error)]
pub enum CorrectError {
    /// Failed to read the transcript or glossary file.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// The LLM request failed.
    #[error("LLM request failed: {0}")]
    Llm(String),
}

/// Options controlling correction behavior.
#[derive(Debug, Clone)]
pub struct CorrectOptions {
    /// Whether to stream tokens to stderr.
    pub stream: bool,
}

impl Default for CorrectOptions {
    fn default() -> Self {
        Self { stream: true }
    }
}

/// Build the full correction system prompt, optionally including
/// glossary terms.
pub fn build_correction_prompt(glossary: Option<&[String]>) -> String {
    let mut prompt = CORRECTION_PROMPT.to_owned();
    if let Some(terms) = glossary {
        if !terms.is_empty() {
            prompt.push_str(GLOSSARY_PREFIX);
            for term in terms {
                prompt.push_str("- ");
                prompt.push_str(term);
                prompt.push('\n');
            }
        }
    }
    prompt
}

/// Load glossary terms from a file (one term per line).
/// Empty lines and lines starting with `#` are skipped.
pub fn load_glossary(path: &Path) -> Result<Vec<String>, CorrectError> {
    let content = std::fs::read_to_string(path)?;
    let terms: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    Ok(terms)
}

/// Derive the corrected output filename from a transcript path.
///
/// `./recordings/1712345678.txt` → `./recordings/1712345678.corrected.txt`
pub fn corrected_filename(transcript_path: &Path) -> PathBuf {
    let stem = transcript_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let parent = transcript_path.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.corrected.txt"))
}

/// Run the correction pipeline on a transcript file.
///
/// Chunks the transcript, sends each chunk through the LLM with the
/// correction prompt, and concatenates results. No merge pass needed
/// since corrections are local to each chunk.
pub async fn run_correction(
    provider: &LlmProvider,
    transcript_path: &Path,
    glossary: Option<&[String]>,
    opts: &CorrectOptions,
) -> Result<String, CorrectError> {
    let transcript = std::fs::read_to_string(transcript_path)?;
    let system_prompt = build_correction_prompt(glossary);

    let chunker = Chunker::default();
    let chunks = chunker.chunk(&transcript);
    let model = provider.model();

    let mut corrected_parts = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        let chat_req = ChatRequest::new(vec![ChatMessage::system(&system_prompt), ChatMessage::user(&chunk.text)]);

        let response = if opts.stream {
            stream_chat(provider, model, chat_req).await?
        } else {
            exec_chat(provider, model, chat_req).await?
        };

        corrected_parts.push(response);
    }

    Ok(corrected_parts.join("\n"))
}

/// Write corrected text to disk atomically.
pub fn write_corrected(output_path: &Path, text: &str) -> Result<(), std::io::Error> {
    let tmp_path = output_path.with_extension("txt.tmp");
    std::fs::write(&tmp_path, text)?;
    std::fs::rename(&tmp_path, output_path)?;
    Ok(())
}

/// Execute a chat request without streaming.
async fn exec_chat(provider: &LlmProvider, model: &str, req: ChatRequest) -> Result<String, CorrectError> {
    let response = provider
        .client()
        .exec_chat(model, req, None)
        .await
        .map_err(|e| CorrectError::Llm(e.to_string()))?;

    Ok(response.first_text().unwrap_or_default().to_owned())
}

/// Execute a chat request with streaming, printing tokens to stderr.
async fn stream_chat(provider: &LlmProvider, model: &str, req: ChatRequest) -> Result<String, CorrectError> {
    let response = provider
        .client()
        .exec_chat_stream(model, req, None)
        .await
        .map_err(|e| CorrectError::Llm(e.to_string()))?;

    let mut full_text = String::new();
    let chat_stream = response.stream;
    use futures::StreamExt;
    use genai::chat::ChatStreamEvent;
    tokio::pin!(chat_stream);
    while let Some(Ok(event)) = chat_stream.next().await {
        if let ChatStreamEvent::Chunk(chunk) = event {
            eprint!("{}", chunk.content);
            full_text.push_str(&chunk.content);
        }
    }
    eprintln!();

    Ok(full_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correction_prompt_contains_instructions() {
        let prompt = build_correction_prompt(None);
        assert!(prompt.contains("transcript correction assistant"));
        assert!(prompt.contains("Fix punctuation"));
        assert!(prompt.contains("filler words"));
        assert!(prompt.contains("speaker labels"));
    }

    #[test]
    fn correction_prompt_includes_glossary() {
        let terms = vec!["Kubernetes".to_owned(), "PostgreSQL".to_owned()];
        let prompt = build_correction_prompt(Some(&terms));
        assert!(prompt.contains("domain-specific terms"));
        assert!(prompt.contains("- Kubernetes"));
        assert!(prompt.contains("- PostgreSQL"));
    }

    #[test]
    fn correction_prompt_empty_glossary_no_section() {
        let terms: Vec<String> = vec![];
        let prompt = build_correction_prompt(Some(&terms));
        assert!(!prompt.contains("domain-specific terms"));
    }

    #[test]
    fn corrected_filename_convention() {
        let path = Path::new("/recordings/1712345678.txt");
        let result = corrected_filename(path);
        assert_eq!(result, PathBuf::from("/recordings/1712345678.corrected.txt"));
    }

    #[test]
    fn corrected_filename_from_json() {
        let path = Path::new("./data/meeting.json");
        let result = corrected_filename(path);
        assert_eq!(result, PathBuf::from("./data/meeting.corrected.txt"));
    }

    #[test]
    fn load_glossary_from_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let glossary_path = tmp.path().join("terms.txt");
        std::fs::write(&glossary_path, "Kubernetes\n# comment\nPostgreSQL\n\nRust\n").expect("write");
        let terms = load_glossary(&glossary_path).expect("load");
        assert_eq!(terms, vec!["Kubernetes", "PostgreSQL", "Rust"]);
    }

    #[test]
    fn load_glossary_missing_file_errors() {
        let result = load_glossary(Path::new("/nonexistent/glossary.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn write_corrected_atomic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = tmp.path().join("test.corrected.txt");
        write_corrected(&output, "Corrected text here").expect("write");
        assert!(output.exists());
        let content = std::fs::read_to_string(&output).expect("read");
        assert_eq!(content, "Corrected text here");
        assert!(!tmp.path().join("test.corrected.txt.tmp").exists());
    }
}
