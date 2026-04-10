//! `plaude summarize` command — recording summarization via LLM.
//!
//! Journey: `specs/journeys/JOURNEY-L4-summarization-pipeline.md`
//! Journey: `specs/journeys/JOURNEY-L7-batch-summarization.md`

use std::path::{Path, PathBuf};

use clap::Args;

use crate::DispatchError;

/// Default template name used when `--template` is not specified.
const DEFAULT_TEMPLATE: &str = "default";

/// Default plaude config directory when none is specified.
fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("plaude"))
        .unwrap_or_else(|| PathBuf::from(".config/plaude"))
}

/// Summarize a recording transcript using an LLM.
#[derive(Debug, Args)]
pub(crate) struct SummarizeArgs {
    /// Path to a transcript file or directory containing transcripts.
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Summarization template to use.
    #[arg(long, default_value = DEFAULT_TEMPLATE, value_name = "NAME")]
    template: String,

    /// Override the LLM model from config.
    #[arg(long, value_name = "MODEL")]
    model: Option<String>,

    /// Suppress streaming output (for scripting).
    #[arg(long)]
    no_stream: bool,

    /// Output structured metadata as JSON to stdout.
    #[arg(long)]
    json: bool,

    /// Re-summarize even if a summary already exists.
    #[arg(long)]
    force: bool,

    /// List recordings that would be summarized without calling the LLM.
    #[arg(long)]
    dry_run: bool,

    /// List templates (shortcut for `plaude template list`).
    #[arg(long, hide = true)]
    list_templates: bool,

    /// Export a built-in template (shortcut for `plaude template show`).
    #[arg(long, value_name = "NAME", hide = true)]
    export_template: Option<String>,
}

/// Run the `plaude summarize` command.
pub(crate) async fn run(args: SummarizeArgs, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let dir = config_dir.map(PathBuf::from).unwrap_or_else(default_config_dir);

    if args.list_templates {
        return list_templates(&dir);
    }
    if let Some(ref name) = args.export_template {
        return export_template(name);
    }

    let path = args
        .path
        .as_deref()
        .ok_or_else(|| DispatchError::Usage("no recording path supplied; run `plaude summarize --help`".to_owned()))?;

    // Batch mode: directory → process all transcripts.
    if path.is_dir() {
        return summarize_batch(&dir, path, &args).await;
    }

    // Single file mode.
    summarize_single(&dir, path, &args).await
}

/// Run the summarization pipeline for a single recording.
async fn summarize_single(config_dir: &Path, path: &Path, args: &SummarizeArgs) -> Result<(), DispatchError> {
    use plaud_llm::summarize::{
        SummarizeOptions, discover_transcript, format_front_matter, run_pipeline, summary_exists, summary_filename, write_summary,
    };
    use plaud_llm::template::TemplateRegistry;

    // Load LLM config.
    let mut config =
        plaud_llm::config::LlmConfig::load(config_dir).map_err(|e| DispatchError::Runtime(format!("failed to load LLM config: {e}")))?;
    if let Some(ref model_override) = args.model {
        config = config.with_model(model_override.clone());
    }

    // Load template.
    let registry = TemplateRegistry::load(config_dir);
    let template = registry.get(&args.template).ok_or_else(|| {
        let infos = registry.list();
        let available: Vec<&str> = infos.iter().map(|i| i.name.as_str()).collect();
        DispatchError::Runtime(format!("unknown template '{}'. Available: {}", args.template, available.join(", ")))
    })?;

    // Discover transcript.
    let transcript_path = discover_transcript(path).map_err(|e| DispatchError::Runtime(e.to_string()))?;

    // Skip if already summarized (unless --force).
    if !args.force && summary_exists(&transcript_path, &args.template) {
        if !args.json {
            eprintln!(
                "Skipped {} (summary already exists, use --force to re-summarize)",
                transcript_path.display()
            );
        }
        return Ok(());
    }

    if args.dry_run {
        println!("{}", transcript_path.display());
        return Ok(());
    }

    if !args.json {
        eprintln!(
            "Summarizing {} with template '{}' using model '{}'...",
            transcript_path.display(),
            args.template,
            config.model
        );
    }

    let provider = plaud_llm::provider::LlmProvider::new(config);
    let opts = SummarizeOptions {
        stream: !args.no_stream && !args.json,
        json_output: args.json,
    };

    let start = std::time::Instant::now();
    let result = run_pipeline(&provider, &transcript_path, template, &opts)
        .await
        .map_err(|e| DispatchError::Runtime(e.to_string()))?;
    let duration = start.elapsed();

    let output_path = summary_filename(&transcript_path, &args.template);
    let front_matter = format_front_matter(&result.model, &result.template, result.token_count);
    write_summary(&output_path, &front_matter, &result.text)
        .map_err(|e| DispatchError::Runtime(format!("failed to write summary: {e}")))?;

    if args.json {
        let meta = serde_json::json!({
            "model": result.model,
            "template": result.template,
            "token_count": result.token_count,
            "duration_secs": duration.as_secs_f64(),
            "output_path": output_path.to_str(),
        });
        println!("{}", serde_json::to_string_pretty(&meta).unwrap_or_default());
    } else {
        eprintln!();
        eprintln!("Summary saved to {}", output_path.display());
        eprintln!("Tip: try --template action-items for a focused summary");
    }

    Ok(())
}

/// Batch mode: summarize all transcripts in a directory.
async fn summarize_batch(config_dir: &Path, dir: &Path, args: &SummarizeArgs) -> Result<(), DispatchError> {
    use plaud_llm::summarize::{
        SummarizeOptions, find_all_transcripts, format_front_matter, run_pipeline, summary_exists, summary_filename, write_summary,
    };
    use plaud_llm::template::TemplateRegistry;

    // Load LLM config.
    let mut config =
        plaud_llm::config::LlmConfig::load(config_dir).map_err(|e| DispatchError::Runtime(format!("failed to load LLM config: {e}")))?;
    if let Some(ref model_override) = args.model {
        config = config.with_model(model_override.clone());
    }

    // Load template.
    let registry = TemplateRegistry::load(config_dir);
    let template = registry.get(&args.template).ok_or_else(|| {
        let infos = registry.list();
        let available: Vec<&str> = infos.iter().map(|i| i.name.as_str()).collect();
        DispatchError::Runtime(format!("unknown template '{}'. Available: {}", args.template, available.join(", ")))
    })?;

    let all_transcripts = find_all_transcripts(dir);
    if all_transcripts.is_empty() {
        eprintln!("No transcripts found in {}", dir.display());
        return Ok(());
    }

    // Partition into to-do and skip.
    let (to_summarize, skipped): (Vec<_>, Vec<_>) = all_transcripts
        .iter()
        .partition(|t| args.force || !summary_exists(t, &args.template));

    if args.dry_run {
        for path in &to_summarize {
            println!("{}", path.display());
        }
        eprintln!(
            "Would summarize {} recording(s), skip {} (already summarized)",
            to_summarize.len(),
            skipped.len()
        );
        return Ok(());
    }

    let total = to_summarize.len();
    let provider = plaud_llm::provider::LlmProvider::new(config);
    let opts = SummarizeOptions {
        stream: !args.no_stream && !args.json,
        json_output: args.json,
    };

    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for (idx, transcript_path) in to_summarize.iter().enumerate() {
        let label = transcript_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        if !args.json {
            eprint!("[{}/{}] Summarizing {label}...", idx + 1, total);
        }

        let start = std::time::Instant::now();
        match run_pipeline(&provider, transcript_path, template, &opts).await {
            Ok(result) => {
                let duration = start.elapsed();
                let output_path = summary_filename(transcript_path, &args.template);
                let front_matter = format_front_matter(&result.model, &result.template, result.token_count);
                if let Err(e) = write_summary(&output_path, &front_matter, &result.text) {
                    eprintln!(" error writing: {e}");
                    failed += 1;
                } else {
                    if !args.json {
                        eprintln!(" done ({:.1}s)", duration.as_secs_f64());
                    }
                    succeeded += 1;
                }
            }
            Err(e) => {
                eprintln!(" error: {e}");
                failed += 1;
            }
        }
    }

    eprintln!();
    eprintln!(
        "Summarized {} recording(s), skipped {} (already summarized){}",
        succeeded,
        skipped.len(),
        if failed > 0 { format!(", {failed} failed") } else { String::new() }
    );

    Ok(())
}

/// `--list-templates`: print a table of available templates.
fn list_templates(config_dir: &Path) -> Result<(), DispatchError> {
    use plaud_llm::template::TemplateRegistry;

    let registry = TemplateRegistry::load(config_dir);
    let infos = registry.list();

    let header_name = "NAME";
    let header_source = "SOURCE";
    let header_preview = "PREVIEW";
    println!("{header_name:<20} {header_source:<10} {header_preview}");
    let sep_name = "----";
    let sep_source = "------";
    let sep_preview = "-------";
    println!("{sep_name:<20} {sep_source:<10} {sep_preview}");

    for info in &infos {
        println!("{:<20} {:<10} {}", info.name, info.source, info.preview);
    }

    println!();
    println!("{} template(s) available", infos.len());
    Ok(())
}

/// `--export-template <name>`: write a built-in template body to stdout.
fn export_template(name: &str) -> Result<(), DispatchError> {
    use plaud_llm::template::TemplateRegistry;

    match TemplateRegistry::builtin_body(name) {
        Some(body) => {
            println!("{body}");
            Ok(())
        }
        None => {
            let names: Vec<&str> = TemplateRegistry::builtin_names().iter().map(|(n, _)| *n).collect();
            Err(DispatchError::Runtime(format!(
                "unknown template '{name}'. Available built-in templates: {}",
                names.join(", ")
            )))
        }
    }
}
