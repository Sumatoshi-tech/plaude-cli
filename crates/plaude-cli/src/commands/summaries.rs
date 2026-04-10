//! `plaude summaries` command group — list, show, delete summaries.
//!
//! Journey: `specs/journeys/JOURNEY-L5-summary-management.md`

use std::path::{Path, PathBuf};

use clap::Subcommand;

use crate::DispatchError;

/// Pattern suffix for summary files: `*.summary.*.md`.
const SUMMARY_GLOB_SUFFIX: &str = ".summary.";

/// Hint shown when no summaries are found.
const HINT_RUN_SUMMARIZE: &str = "Run: plaude summarize <path>";

/// Summary management commands.
#[derive(Debug, Subcommand)]
pub(crate) enum SummariesCommand {
    /// List all summaries for a recording.
    List(ListArgs),
    /// Show a summary's content.
    Show(ShowArgs),
    /// Delete a specific summary.
    Delete(DeleteArgs),
}

/// Arguments for `plaude summaries list`.
#[derive(Debug, clap::Args)]
pub(crate) struct ListArgs {
    /// Path to a recording directory or transcript file.
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Output as JSON array.
    #[arg(long)]
    json: bool,
}

/// Arguments for `plaude summaries show`.
#[derive(Debug, clap::Args)]
pub(crate) struct ShowArgs {
    /// Path to a recording directory or transcript file.
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Template name of the summary to show.
    #[arg(long, value_name = "NAME")]
    template: Option<String>,
}

/// Arguments for `plaude summaries delete`.
#[derive(Debug, clap::Args)]
pub(crate) struct DeleteArgs {
    /// Path to a recording directory or transcript file.
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Template name of the summary to delete (required).
    #[arg(long, value_name = "NAME")]
    template: String,
}

/// Run the `plaude summaries` subcommand.
pub(crate) fn run(cmd: SummariesCommand) -> Result<(), DispatchError> {
    match cmd {
        SummariesCommand::List(args) => list_summaries(&args),
        SummariesCommand::Show(args) => show_summary(&args),
        SummariesCommand::Delete(args) => delete_summary(&args),
    }
}

/// Metadata parsed from a summary file's YAML front matter.
#[derive(Debug, serde::Serialize)]
struct SummaryMeta {
    template: String,
    model: String,
    created_at: String,
    file_size: u64,
    path: PathBuf,
}

/// Find all summary files in a directory or adjacent to a file.
fn find_summary_files(path: &Path) -> Vec<PathBuf> {
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(Path::new("."))
    };

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut summaries: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|e| e == "md")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.contains(SUMMARY_GLOB_SUFFIX))
        })
        .collect();

    summaries.sort();
    summaries
}

/// Extract the template name from a summary filename.
///
/// `1712345678.summary.default.md` → `"default"`
fn template_from_filename(path: &Path) -> Option<String> {
    let name = path.file_stem()?.to_str()?;
    // name is "1712345678.summary.default" (without .md)
    let after_summary = name.split(".summary.").nth(1)?;
    Some(after_summary.to_owned())
}

/// Parse YAML front matter from a summary file.
fn parse_front_matter(content: &str) -> (String, String) {
    // Format: ---\nkey: value\n...\n---\n\nbody
    if !content.starts_with("---\n") {
        return (String::new(), String::new());
    }
    if let Some(end) = content[4..].find("\n---\n") {
        let yaml_part = &content[4..4 + end];
        let body_start = 4 + end + 5; // past "\n---\n"
        let body = content[body_start..].trim_start().to_owned();
        (yaml_part.to_owned(), body)
    } else {
        (String::new(), content.to_owned())
    }
}

/// Extract a value from simple YAML front matter.
fn yaml_value(yaml: &str, key: &str) -> String {
    let prefix = format!("{key}: ");
    yaml.lines()
        .find(|l| l.starts_with(&prefix))
        .map(|l| l[prefix.len()..].trim().to_owned())
        .unwrap_or_default()
}

/// Build metadata for a summary file.
fn build_meta(path: &Path) -> Option<SummaryMeta> {
    let content = std::fs::read_to_string(path).ok()?;
    let file_size = std::fs::metadata(path).ok()?.len();
    let template = template_from_filename(path)?;
    let (yaml, _body) = parse_front_matter(&content);

    Some(SummaryMeta {
        template,
        model: yaml_value(&yaml, "model"),
        created_at: yaml_value(&yaml, "created_at"),
        file_size,
        path: path.to_owned(),
    })
}

/// `plaude summaries list <path>`
fn list_summaries(args: &ListArgs) -> Result<(), DispatchError> {
    let files = find_summary_files(&args.path);

    if files.is_empty() {
        eprintln!("No summaries found. {HINT_RUN_SUMMARIZE}");
        return Ok(());
    }

    let metas: Vec<SummaryMeta> = files.iter().filter_map(|f| build_meta(f)).collect();

    if args.json {
        let json = serde_json::to_string_pretty(&metas).map_err(|e| DispatchError::Runtime(format!("JSON serialization failed: {e}")))?;
        println!("{json}");
        return Ok(());
    }

    let h_template = "TEMPLATE";
    let h_model = "MODEL";
    let h_date = "CREATED";
    let h_size = "SIZE";
    println!("{h_template:<20} {h_model:<20} {h_date:<26} {h_size:>8}");
    let s_template = "--------";
    let s_model = "-----";
    let s_date = "-------";
    let s_size = "----";
    println!("{s_template:<20} {s_model:<20} {s_date:<26} {s_size:>8}");

    for meta in &metas {
        let size_display = format_size(meta.file_size);
        println!(
            "{:<20} {:<20} {:<26} {:>8}",
            meta.template, meta.model, meta.created_at, size_display
        );
    }

    println!();
    println!("{} summary(ies) found", metas.len());
    Ok(())
}

/// `plaude summaries show <path> [--template <name>]`
fn show_summary(args: &ShowArgs) -> Result<(), DispatchError> {
    let files = find_summary_files(&args.path);

    let target = if let Some(ref tpl) = args.template {
        files.iter().find(|f| template_from_filename(f).as_deref() == Some(tpl.as_str()))
    } else {
        // Show the most recent (last sorted, which is typically alphabetically last).
        files.last()
    };

    let Some(path) = target else {
        let msg = if let Some(ref tpl) = args.template {
            let available: Vec<String> = files.iter().filter_map(|f| template_from_filename(f)).collect();
            if available.is_empty() {
                format!("No summaries found. {HINT_RUN_SUMMARIZE}")
            } else {
                format!("No summary with template '{tpl}'. Available: {}", available.join(", "))
            }
        } else {
            format!("No summaries found. {HINT_RUN_SUMMARIZE}")
        };
        return Err(DispatchError::Runtime(msg));
    };

    let content = std::fs::read_to_string(path).map_err(|e| DispatchError::Runtime(format!("failed to read summary: {e}")))?;

    // Print body without front matter.
    let (_yaml, body) = parse_front_matter(&content);
    println!("{body}");
    Ok(())
}

/// `plaude summaries delete <path> --template <name>`
fn delete_summary(args: &DeleteArgs) -> Result<(), DispatchError> {
    let files = find_summary_files(&args.path);

    let target = files
        .iter()
        .find(|f| template_from_filename(f).as_deref() == Some(args.template.as_str()));

    let Some(path) = target else {
        return Err(DispatchError::Runtime(format!(
            "no summary with template '{}' found at {}",
            args.template,
            args.path.display()
        )));
    };

    std::fs::remove_file(path).map_err(|e| DispatchError::Runtime(format!("failed to delete summary: {e}")))?;

    eprintln!("Deleted summary: {}", path.display());
    Ok(())
}

/// Format a byte size for human display.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
