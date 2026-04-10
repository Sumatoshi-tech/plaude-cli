//! Summarization pipeline: transcript → chunk → LLM → summary file.
//!
//! Journey: `specs/journeys/JOURNEY-L4-summarization-pipeline.md`

use std::path::{Path, PathBuf};

use genai::chat::{ChatMessage, ChatRequest};

use crate::chunk::Chunker;
use crate::provider::LlmProvider;
use crate::template::Template;

/// Merge prompt used when multiple chunk summaries need combining.
const MERGE_SYSTEM_PROMPT: &str = "\
You are a summarization assistant. You are given multiple partial \
summaries of different sections of the same recording. Merge them \
into a single coherent summary, removing duplication and preserving \
all key information. Output only the final merged summary.";

/// Transcript file extensions in order of preference.
const TRANSCRIPT_EXTENSIONS: &[&str] = &["json", "txt", "srt", "vtt"];

/// Hint shown when no transcript is found.
const HINT_TRANSCRIBE: &str = "run `plaude transcribe` first to create a transcript";

/// Errors from the summarization pipeline.
#[derive(Debug, thiserror::Error)]
pub enum SummarizeError {
    /// No transcript file found at the given path.
    #[error("no transcript found at {path} — {HINT_TRANSCRIBE}")]
    NoTranscript {
        /// The path that was searched.
        path: PathBuf,
    },
    /// Failed to read the transcript file.
    #[error("failed to read transcript: {0}")]
    ReadTranscript(#[from] std::io::Error),
    /// The LLM request failed.
    #[error("LLM request failed: {0}")]
    Llm(String),
}

/// Result of a completed summarization.
#[derive(Debug, Clone)]
pub struct SummaryResult {
    /// The generated summary text (without front matter).
    pub text: String,
    /// Model used for generation.
    pub model: String,
    /// Template name used.
    pub template: String,
    /// Approximate token count (from LLM response, if available).
    pub token_count: Option<u64>,
}

/// Options controlling summarization behavior.
#[derive(Debug, Clone)]
pub struct SummarizeOptions {
    /// Whether to stream tokens to stderr.
    pub stream: bool,
    /// Whether to output JSON metadata to stdout.
    pub json_output: bool,
}

impl Default for SummarizeOptions {
    fn default() -> Self {
        Self {
            stream: true,
            json_output: false,
        }
    }
}

/// Discover a transcript file from a path.
///
/// If `path` is a file, uses it directly. If it's a directory,
/// searches for transcript files by extension preference order
/// (json > txt > srt > vtt). Returns the first match.
pub fn discover_transcript(path: &Path) -> Result<PathBuf, SummarizeError> {
    if path.is_file() {
        return Ok(path.to_owned());
    }

    if path.is_dir() {
        for ext in TRANSCRIPT_EXTENSIONS {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().is_some_and(|e| e == *ext) {
                        return Ok(p);
                    }
                }
            }
        }
    }

    Err(SummarizeError::NoTranscript { path: path.to_owned() })
}

/// Find all transcript files in a directory.
///
/// Returns sorted list of paths matching transcript extensions.
pub fn find_all_transcripts(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut transcripts: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| TRANSCRIPT_EXTENSIONS.contains(&ext))
        })
        .collect();

    transcripts.sort();
    transcripts
}

/// Check whether a summary file already exists for a transcript and
/// template combination.
pub fn summary_exists(transcript_path: &Path, template_name: &str) -> bool {
    summary_filename(transcript_path, template_name).exists()
}

/// Derive the summary output filename from a transcript path and
/// template name.
///
/// Given `./recordings/1712345678.txt` and template `default`,
/// returns `./recordings/1712345678.summary.default.md`.
pub fn summary_filename(transcript_path: &Path, template_name: &str) -> PathBuf {
    let stem = transcript_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let parent = transcript_path.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.summary.{template_name}.md"))
}

/// Format YAML front matter for the summary file.
pub fn format_front_matter(model: &str, template: &str, token_count: Option<u64>) -> String {
    let now = jiff::Zoned::now();
    let timestamp = now.strftime("%Y-%m-%dT%H:%M:%S%:z");
    let tokens = token_count.map_or_else(|| "unknown".to_owned(), |t| t.to_string());
    format!("---\nmodel: {model}\ntemplate: {template}\ncreated_at: {timestamp}\ntoken_count: {tokens}\n---\n\n")
}

/// Run the full summarization pipeline.
///
/// 1. Read the transcript
/// 2. Chunk if needed
/// 3. Send each chunk to the LLM with the template as system message
/// 4. If multi-chunk, merge partial summaries with a final LLM pass
/// 5. Write summary file atomically
pub async fn run_pipeline(
    provider: &LlmProvider,
    transcript_path: &Path,
    template: &Template,
    opts: &SummarizeOptions,
) -> Result<SummaryResult, SummarizeError> {
    let transcript = std::fs::read_to_string(transcript_path)?;

    let chunker = Chunker::default();
    let chunks = chunker.chunk(&transcript);

    let model = provider.model();

    // Summarize each chunk.
    let mut partial_summaries = Vec::with_capacity(chunks.len());
    let total_chunks = chunks.len();
    for (i, chunk) in chunks.iter().enumerate() {
        if total_chunks > 1 && !opts.json_output {
            eprintln!("[{}/{}] Processing chunk ({} chars)...", i + 1, total_chunks, chunk.text.len());
        }

        let chat_req = ChatRequest::new(vec![ChatMessage::system(&template.body), ChatMessage::user(&chunk.text)]);

        let response = if opts.stream {
            stream_chat(provider, model, chat_req).await?
        } else {
            exec_chat(provider, model, chat_req).await?
        };

        partial_summaries.push(response);
    }

    // Merge if multi-chunk.
    let final_text = if partial_summaries.len() > 1 {
        let combined = partial_summaries.join("\n\n---\n\n");
        let merge_req = ChatRequest::new(vec![ChatMessage::system(MERGE_SYSTEM_PROMPT), ChatMessage::user(&combined)]);

        if opts.stream {
            if !opts.json_output {
                eprintln!("\n--- Merging partial summaries ---\n");
            }
            stream_chat(provider, model, merge_req).await?
        } else {
            exec_chat(provider, model, merge_req).await?
        }
    } else {
        partial_summaries.into_iter().next().unwrap_or_default()
    };

    Ok(SummaryResult {
        text: final_text,
        model: model.to_owned(),
        template: template.name.clone(),
        token_count: None, // genai doesn't expose this consistently
    })
}

/// Write a summary to disk atomically (write to .tmp, then rename).
pub fn write_summary(output_path: &Path, front_matter: &str, summary_text: &str) -> Result<(), std::io::Error> {
    let tmp_path = output_path.with_extension("md.tmp");
    let content = format!("{front_matter}{summary_text}\n");
    std::fs::write(&tmp_path, &content)?;
    std::fs::rename(&tmp_path, output_path)?;
    Ok(())
}

/// Execute a chat request without streaming.
async fn exec_chat(provider: &LlmProvider, model: &str, req: ChatRequest) -> Result<String, SummarizeError> {
    let response = provider
        .client()
        .exec_chat(model, req, None)
        .await
        .map_err(|e| SummarizeError::Llm(e.to_string()))?;

    Ok(response.first_text().unwrap_or_default().to_owned())
}

/// Execute a chat request with streaming, printing tokens to stderr.
async fn stream_chat(provider: &LlmProvider, model: &str, req: ChatRequest) -> Result<String, SummarizeError> {
    let response = provider
        .client()
        .exec_chat_stream(model, req, None)
        .await
        .map_err(|e| SummarizeError::Llm(e.to_string()))?;

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
    eprintln!(); // Final newline after streaming.

    Ok(full_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_transcript_finds_txt() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("1712345678.txt"), "hello").expect("write");
        let result = discover_transcript(tmp.path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().ends_with(".txt"));
    }

    #[test]
    fn discover_transcript_finds_json() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("1712345678.json"), "{}").expect("write");
        let result = discover_transcript(tmp.path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().ends_with(".json"));
    }

    #[test]
    fn discover_transcript_prefers_json_over_txt() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("recording.txt"), "text").expect("write");
        std::fs::write(tmp.path().join("recording.json"), "{}").expect("write");
        let result = discover_transcript(tmp.path()).unwrap();
        assert!(
            result.to_str().unwrap().ends_with(".json"),
            "expected .json preference, got: {result:?}"
        );
    }

    #[test]
    fn discover_transcript_missing_returns_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = discover_transcript(tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("no transcript found"), "got: {msg}");
        assert!(msg.contains("plaude transcribe"), "got: {msg}");
    }

    #[test]
    fn discover_transcript_direct_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let file = tmp.path().join("my-transcript.txt");
        std::fs::write(&file, "content").expect("write");
        let result = discover_transcript(&file).unwrap();
        assert_eq!(result, file);
    }

    #[test]
    fn summary_filename_convention() {
        let transcript = Path::new("/recordings/1712345678.txt");
        let result = summary_filename(transcript, "default");
        assert_eq!(result, PathBuf::from("/recordings/1712345678.summary.default.md"));
    }

    #[test]
    fn summary_filename_with_custom_template() {
        let transcript = Path::new("./data/meeting.json");
        let result = summary_filename(transcript, "action-items");
        assert_eq!(result, PathBuf::from("./data/meeting.summary.action-items.md"));
    }

    #[test]
    fn front_matter_contains_required_fields() {
        let fm = format_front_matter("llama3.2:3b", "default", Some(150));
        assert!(fm.starts_with("---\n"));
        assert!(fm.contains("model: llama3.2:3b"));
        assert!(fm.contains("template: default"));
        assert!(fm.contains("created_at:"));
        assert!(fm.contains("token_count: 150"));
        assert!(fm.contains("---\n\n"));
    }

    #[test]
    fn front_matter_unknown_tokens() {
        let fm = format_front_matter("model", "tpl", None);
        assert!(fm.contains("token_count: unknown"));
    }

    #[test]
    fn write_summary_atomic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = tmp.path().join("test.summary.default.md");
        write_summary(&output, "---\nmodel: test\n---\n\n", "Summary text").expect("write");
        assert!(output.exists());
        let content = std::fs::read_to_string(&output).expect("read");
        assert!(content.contains("model: test"));
        assert!(content.contains("Summary text"));
        // Temp file should be cleaned up.
        assert!(!tmp.path().join("test.summary.default.md.tmp").exists());
    }

    #[test]
    fn find_all_transcripts_in_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("rec1.txt"), "text").expect("write");
        std::fs::write(tmp.path().join("rec2.json"), "{}").expect("write");
        std::fs::write(tmp.path().join("rec3.wav"), "binary").expect("write"); // not a transcript
        std::fs::write(tmp.path().join("rec4.srt"), "srt").expect("write");

        let results = find_all_transcripts(tmp.path());
        assert_eq!(results.len(), 3);
        // Should be sorted.
        let names: Vec<&str> = results.iter().filter_map(|p| p.file_name()?.to_str()).collect();
        assert!(names.contains(&"rec1.txt"));
        assert!(names.contains(&"rec2.json"));
        assert!(names.contains(&"rec4.srt"));
        assert!(!names.contains(&"rec3.wav"));
    }

    #[test]
    fn find_all_transcripts_empty_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(find_all_transcripts(tmp.path()).is_empty());
    }

    #[test]
    fn summary_exists_false_when_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let transcript = tmp.path().join("rec.txt");
        std::fs::write(&transcript, "text").expect("write");
        assert!(!summary_exists(&transcript, "default"));
    }

    #[test]
    fn summary_exists_true_when_present() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let transcript = tmp.path().join("rec.txt");
        std::fs::write(&transcript, "text").expect("write");
        std::fs::write(tmp.path().join("rec.summary.default.md"), "summary").expect("write");
        assert!(summary_exists(&transcript, "default"));
    }
}
