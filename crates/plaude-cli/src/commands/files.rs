//! `plaude-cli files list` + `plaude-cli files pull-one`.
//!
//! Both commands go through the authenticated [`TransportProvider`]
//! chain established in M6. `list` prints a table (text) or an array
//! (json). `pull-one` fetches the stereo PCM `.WAV` and the mono Opus
//! `.ASR` sidecar of a single recording, writes them to disk under
//! `<id>.wav` / `<id>.asr`, and reports progress via `indicatif`.
//!
//! Resume semantics in M7 are **idempotent skip**: if the target
//! file already exists with exactly the expected byte count, the
//! command is a no-op for that file. Mid-offset resume and streaming
//! writes are tracked for a later hardening milestone and require
//! range-reads on the `Transport` trait.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use plaud_auth::DEFAULT_DEVICE_ID;
use plaud_domain::{AuthToken, Recording, RecordingId};
use plaud_transport::Transport;
use serde::Serialize;

use crate::{
    DispatchError,
    commands::{auth::build_store, backend::TransportProvider, output::OutputFormat},
};

/// Column header row for the text output of `files list`.
const TEXT_TABLE_HEADER: &str = "ID           KIND  STARTED              WAV        ASR";
/// Human-readable message emitted by `pull-one` when the target files
/// already exist at the expected sizes.
const ALREADY_UP_TO_DATE_MSG: &str = "already up to date";
/// Extension for the stereo PCM file.
const WAV_EXTENSION: &str = "wav";
/// Extension for the mono Opus sidecar.
const ASR_EXTENSION: &str = "asr";
/// Progress-bar template string passed to `indicatif`.
const PROGRESS_TEMPLATE: &str = "{spinner} {msg} [{bar:32.cyan/blue}] {bytes}/{total_bytes}";
/// Progress-bar progress-char string.
const PROGRESS_CHARS: &str = "=>-";
/// Error context used when a partial pre-existing file is rewritten
/// without `--resume` or rewritten from scratch during a retry.
const PARTIAL_FILE_REWRITE_CONTEXT: &str = "rewriting partial file";

/// `plaude-cli files` subcommand tree.
#[derive(Debug, Subcommand)]
pub(crate) enum FilesCommand {
    /// List every recording on the connected device.
    List(ListArgs),
    /// Download a single recording (both `.WAV` and `.ASR`) to disk.
    PullOne(PullOneArgs),
}

/// Arguments for `plaude-cli files list`.
#[derive(Debug, Args)]
pub(crate) struct ListArgs {
    /// Output format: `text` (default table) or `json`.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

/// Arguments for `plaude-cli files pull-one`.
#[derive(Debug, Args)]
pub(crate) struct PullOneArgs {
    /// The recording id to pull. Matches the value printed in the
    /// `ID` column of `files list`.
    id: String,
    /// Output directory. Defaults to the current working directory.
    /// Created if missing.
    #[arg(short = 'o', long = "output-dir", value_name = "DIR")]
    output_dir: Option<PathBuf>,
    /// When set, skip files already present at the expected size.
    /// Without this flag, pre-existing files are overwritten.
    #[arg(long)]
    resume: bool,
}

/// Flat JSON projection of a [`Recording`].
#[derive(Debug, Serialize)]
struct RecordingJson {
    id: String,
    kind: String,
    started_at_unix_seconds: i64,
    wav_size: u64,
    asr_size: u64,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) async fn run(cmd: FilesCommand, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    match cmd {
        FilesCommand::List(args) => list(args, provider, config_dir).await,
        FilesCommand::PullOne(args) => pull_one(args, provider, config_dir).await,
    }
}

async fn list(args: ListArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let transport = authenticated_transport(provider, config_dir).await?;
    let recordings = transport
        .list_recordings()
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    print_list(&recordings, args.output)
}

async fn pull_one(args: PullOneArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let recording_id = RecordingId::new(args.id.clone()).map_err(|e| DispatchError::Usage(format!("invalid recording id: {e}")))?;
    let transport = authenticated_transport(provider, config_dir).await?;
    // Try to find the recording in the device listing. For BLE delta
    // protocol, the listing may be empty even if the recording exists.
    // If not found, attempt the download anyway with size=0 (unknown).
    let listing = transport.list_recordings().await.unwrap_or_default();
    let meta = listing.iter().find(|r| r.id() == &recording_id).cloned();
    let dir = args.output_dir.unwrap_or_else(|| PathBuf::from("."));
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to create output dir {}: {e}", dir.display())))?;
    let wav_path = dir.join(format!("{}.{WAV_EXTENSION}", recording_id.as_str()));
    let asr_path = dir.join(format!("{}.{ASR_EXTENSION}", recording_id.as_str()));
    let wav_size = meta.as_ref().map_or(0, |m| m.wav_size());
    let asr_size = meta.as_ref().map_or(0, |m| m.asr_size());
    let wav_written = pull_file(transport.as_ref(), &recording_id, FileKind::Wav, wav_size, &wav_path, args.resume).await?;
    // ASR sidecar download is best-effort — BLE transport may not
    // support it separately from the WAV.
    let asr_written = pull_file(transport.as_ref(), &recording_id, FileKind::Asr, asr_size, &asr_path, args.resume)
        .await
        .unwrap_or_default();
    if !wav_written && !asr_written {
        println!("{} {}", recording_id.as_str(), ALREADY_UP_TO_DATE_MSG);
    }
    Ok(())
}

async fn authenticated_transport(provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<Box<dyn Transport>, DispatchError> {
    let token = load_token(config_dir).await?;
    provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))
}

async fn load_token(config_dir: Option<&Path>) -> Result<AuthToken, DispatchError> {
    let store = build_store(config_dir)?;
    let stored = store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read token store: {e}")))?;
    stored.ok_or(DispatchError::AuthRequired)
}

/// Returns `Ok(true)` if the file was written (or rewritten) and
/// `Ok(false)` if the existing on-disk file was kept thanks to
/// `--resume`. Any I/O or transport failure surfaces through
/// `DispatchError`.
async fn pull_file(
    transport: &dyn Transport,
    id: &RecordingId,
    kind: FileKind,
    expected_size: u64,
    path: &Path,
    resume: bool,
) -> Result<bool, DispatchError> {
    if resume && file_is_already_complete(path, expected_size).await? {
        return Ok(false);
    }
    let bytes = match kind {
        FileKind::Wav => transport.read_recording(id).await,
        FileKind::Asr => transport.read_recording_asr(id).await,
    };
    let bytes = bytes.map_err(|e| DispatchError::from_transport_error(&e))?;
    write_with_progress(path, &bytes, id, kind).await
}

async fn file_is_already_complete(path: &Path, expected_size: u64) -> Result<bool, DispatchError> {
    match tokio::fs::metadata(path).await {
        Ok(meta) if meta.len() == expected_size => Ok(true),
        Ok(_) | Err(_) => Ok(false),
    }
}

async fn write_with_progress(path: &Path, bytes: &[u8], id: &RecordingId, kind: FileKind) -> Result<bool, DispatchError> {
    if tokio::fs::try_exists(path).await.unwrap_or(false) {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| DispatchError::Runtime(format!("{PARTIAL_FILE_REWRITE_CONTEXT} {}: {e}", path.display())))?;
    }
    tokio::fs::write(path, bytes)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to write {}: {e}", path.display())))?;
    render_progress(id.as_str(), kind, bytes.len() as u64);
    Ok(true)
}

fn render_progress(id: &str, kind: FileKind, size: u64) {
    let bar = ProgressBar::new(size);
    if let Ok(style) = ProgressStyle::with_template(PROGRESS_TEMPLATE) {
        bar.set_style(style.progress_chars(PROGRESS_CHARS));
    }
    bar.set_message(format!("{id} {}", kind.label()));
    bar.set_position(size);
    bar.finish();
}

fn print_list(recordings: &[Recording], output: OutputFormat) -> Result<(), DispatchError> {
    match output {
        OutputFormat::Text => {
            println!("{TEXT_TABLE_HEADER}");
            for r in recordings {
                println!(
                    "{:<12} {:<5} {:<20} {:<10} {:<10}",
                    r.id().as_str(),
                    r.kind().name(),
                    r.started_at_unix_seconds(),
                    r.wav_size(),
                    r.asr_size()
                );
            }
            Ok(())
        }
        OutputFormat::Json => {
            let payload: Vec<RecordingJson> = recordings.iter().map(RecordingJson::from).collect();
            let rendered = serde_json::to_string(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))?;
            println!("{rendered}");
            Ok(())
        }
    }
}

impl From<&Recording> for RecordingJson {
    fn from(r: &Recording) -> Self {
        Self {
            id: r.id().as_str().to_owned(),
            kind: r.kind().name().to_owned(),
            started_at_unix_seconds: r.started_at_unix_seconds(),
            wav_size: r.wav_size(),
            asr_size: r.asr_size(),
        }
    }
}

/// Identifies which of the two paired files we're pulling right now.
#[derive(Debug, Clone, Copy)]
enum FileKind {
    Wav,
    Asr,
}

impl FileKind {
    fn label(self) -> &'static str {
        match self {
            Self::Wav => WAV_EXTENSION,
            Self::Asr => ASR_EXTENSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use plaud_domain::{Recording, RecordingId, RecordingKind};
    use serde_json::Value;

    use super::{ALREADY_UP_TO_DATE_MSG, ASR_EXTENSION, RecordingJson, TEXT_TABLE_HEADER, WAV_EXTENSION};

    const BASENAME: &str = "1775393534";
    const WAV_SIZE: u64 = 128;
    const ASR_SIZE: u64 = 64;

    fn sample() -> Recording {
        Recording::new(RecordingId::new(BASENAME).expect("valid"), RecordingKind::Note, WAV_SIZE, ASR_SIZE)
    }

    #[test]
    fn recording_json_schema_has_stable_keys() {
        let payload = RecordingJson::from(&sample());
        let rendered = serde_json::to_string(&payload).expect("encode");
        let parsed: Value = serde_json::from_str(&rendered).expect("parse");
        assert_eq!(parsed["id"], BASENAME);
        assert_eq!(parsed["wav_size"], WAV_SIZE);
        assert_eq!(parsed["asr_size"], ASR_SIZE);
        assert!(parsed.get("kind").is_some());
        assert!(parsed.get("started_at_unix_seconds").is_some());
    }

    #[test]
    fn text_table_header_and_file_extensions_are_stable() {
        // Mutation target: any rename of these constants breaks
        // downstream scripts that parse the text output.
        assert!(TEXT_TABLE_HEADER.starts_with("ID"));
        assert!(TEXT_TABLE_HEADER.contains("KIND"));
        assert!(TEXT_TABLE_HEADER.contains("WAV"));
        assert!(TEXT_TABLE_HEADER.contains("ASR"));
        assert_eq!(WAV_EXTENSION, "wav");
        assert_eq!(ASR_EXTENSION, "asr");
        assert!(ALREADY_UP_TO_DATE_MSG.contains("up to date"));
    }
}
