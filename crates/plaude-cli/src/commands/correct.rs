//! `plaude correct` command — transcript auto-correction via LLM.
//!
//! Journey: `specs/journeys/JOURNEY-L6-transcript-correction.md`

use std::path::{Path, PathBuf};

use clap::Args;

use crate::DispatchError;

/// Default plaude config directory when none is specified.
fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("plaude"))
        .unwrap_or_else(|| PathBuf::from(".config/plaude"))
}

/// Correct a transcript using an LLM to fix speech-to-text errors.
#[derive(Debug, Args)]
pub(crate) struct CorrectArgs {
    /// Path to the transcript file to correct.
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Path to a glossary file (one term per line) for domain-specific corrections.
    #[arg(long, value_name = "FILE")]
    glossary: Option<PathBuf>,

    /// Override the LLM model from config.
    #[arg(long, value_name = "MODEL")]
    model: Option<String>,

    /// Suppress streaming output (for scripting).
    #[arg(long)]
    no_stream: bool,
}

/// Run the `plaude correct` command.
pub(crate) async fn run(args: CorrectArgs, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let dir = config_dir.map(PathBuf::from).unwrap_or_else(default_config_dir);

    let path = args
        .path
        .as_deref()
        .ok_or_else(|| DispatchError::Usage("no transcript path supplied; run `plaude correct --help`".to_owned()))?;

    if !path.is_file() {
        return Err(DispatchError::Runtime(format!(
            "file not found: {} — provide a path to a transcript file",
            path.display()
        )));
    }

    // Load LLM config.
    let mut config =
        plaud_llm::config::LlmConfig::load(&dir).map_err(|e| DispatchError::Runtime(format!("failed to load LLM config: {e}")))?;
    if let Some(ref model_override) = args.model {
        config = config.with_model(model_override.clone());
    }

    // Load glossary if provided.
    let glossary = if let Some(ref glossary_path) = args.glossary {
        Some(
            plaud_llm::correct::load_glossary(glossary_path)
                .map_err(|e| DispatchError::Runtime(format!("failed to load glossary: {e}")))?,
        )
    } else {
        None
    };

    eprintln!("Correcting {} using model '{}'...", path.display(), config.model);

    let provider = plaud_llm::provider::LlmProvider::new(config);
    let opts = plaud_llm::correct::CorrectOptions { stream: !args.no_stream };

    let corrected = plaud_llm::correct::run_correction(&provider, path, glossary.as_deref(), &opts)
        .await
        .map_err(|e| DispatchError::Runtime(e.to_string()))?;

    // Write output.
    let output_path = plaud_llm::correct::corrected_filename(path);
    plaud_llm::correct::write_corrected(&output_path, &corrected)
        .map_err(|e| DispatchError::Runtime(format!("failed to write corrected file: {e}")))?;

    eprintln!();
    eprintln!("Corrected transcript saved to {}", output_path.display());
    Ok(())
}
