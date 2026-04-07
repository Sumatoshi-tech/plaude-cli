//! `plaude transcribe` — offline transcription via whisper.cpp.
//!
//! Wraps an externally-installed `whisper.cpp` CLI binary as a
//! subprocess, passing one or more WAV files and a user-supplied
//! GGML model path. Fully offline — no network, no cloud, no API key.
//!
//! Journey: specs/plaude-v1/journeys/M15-whisper-transcribe.md

use std::{path::PathBuf, process::Command as ProcessCommand};

use clap::{Args, ValueEnum};

use crate::DispatchError;

/// Default binary name looked up on `$PATH` when `--whisper-bin` is
/// not supplied.
const DEFAULT_WHISPER_BIN: &str = "whisper-cli";

/// Environment variable for the whisper binary path.
const ENV_WHISPER_BIN: &str = "PLAUDE_WHISPER_BIN";

/// Environment variable for the model path.
const ENV_WHISPER_MODEL: &str = "PLAUDE_WHISPER_MODEL";

/// Whisper output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum TranscribeFormat {
    /// Plain text.
    Txt,
    /// SubRip subtitle format.
    Srt,
    /// WebVTT subtitle format.
    Vtt,
}

impl TranscribeFormat {
    /// The whisper.cpp CLI flag for this format.
    fn whisper_flag(self) -> &'static str {
        match self {
            Self::Txt => "--output-txt",
            Self::Srt => "--output-srt",
            Self::Vtt => "--output-vtt",
        }
    }
}

/// Arguments for `plaude transcribe`.
#[derive(Debug, Args)]
pub(crate) struct TranscribeArgs {
    /// Path to the whisper.cpp binary. Also settable via `PLAUDE_WHISPER_BIN`. Defaults to `whisper-cli` on `$PATH`.
    #[arg(long, value_name = "PATH", env = ENV_WHISPER_BIN)]
    whisper_bin: Option<PathBuf>,

    /// Path to the GGML model file (e.g. `ggml-base.bin`). Also settable via `PLAUDE_WHISPER_MODEL`. Required.
    #[arg(long, value_name = "PATH", env = ENV_WHISPER_MODEL)]
    model: PathBuf,

    /// Language hint for whisper (e.g. `en`, `de`, `auto`).
    #[arg(long)]
    language: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = TranscribeFormat::Txt)]
    output_format: TranscribeFormat,

    /// One or more WAV files to transcribe.
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) fn run(args: TranscribeArgs) -> Result<(), DispatchError> {
    let whisper_bin = args.whisper_bin.unwrap_or_else(|| PathBuf::from(DEFAULT_WHISPER_BIN));

    // Validate model exists
    if !args.model.exists() {
        return Err(DispatchError::Usage(format!("model file not found: {}", args.model.display())));
    }

    // Validate each WAV file exists
    for file in &args.files {
        if !file.exists() {
            return Err(DispatchError::Runtime(format!("WAV file not found: {}", file.display())));
        }
    }

    for file in &args.files {
        transcribe_one(&whisper_bin, &args.model, args.language.as_deref(), args.output_format, file)?;
    }

    Ok(())
}

/// Run whisper.cpp on a single WAV file and print its stdout.
fn transcribe_one(
    whisper_bin: &PathBuf,
    model: &PathBuf,
    language: Option<&str>,
    format: TranscribeFormat,
    wav_file: &PathBuf,
) -> Result<(), DispatchError> {
    let mut cmd = ProcessCommand::new(whisper_bin);
    cmd.arg("--model").arg(model);
    cmd.arg(format.whisper_flag());
    cmd.arg("--no-prints");
    if let Some(lang) = language {
        cmd.arg("--language").arg(lang);
    }
    cmd.arg("--file").arg(wav_file);

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DispatchError::Unavailable(format!("whisper binary not found: {}", whisper_bin.display()))
        } else {
            DispatchError::Runtime(format!("failed to run whisper: {e}"))
        }
    })?;

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        return Err(DispatchError::Runtime(format!(
            "whisper exited with {}: {}",
            output.status,
            stderr_text.trim()
        )));
    }

    let stdout_text = String::from_utf8_lossy(&output.stdout);
    print!("{stdout_text}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TranscribeFormat;

    #[test]
    fn format_flags_match_whisper_cli_conventions() {
        assert_eq!(TranscribeFormat::Txt.whisper_flag(), "--output-txt");
        assert_eq!(TranscribeFormat::Srt.whisper_flag(), "--output-srt");
        assert_eq!(TranscribeFormat::Vtt.whisper_flag(), "--output-vtt");
    }
}
