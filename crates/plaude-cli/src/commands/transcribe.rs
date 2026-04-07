//! `plaude transcribe` — offline transcription via whisper-rs.
//!
//! Transcribes WAV files to text using an in-process whisper.cpp
//! engine via the `whisper-rs` crate. No external binary needed.
//!
//! Quality presets (`--quality fast|medium|high`) map to specific
//! GGML model files so users never need to know model names.

use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use serde::Serialize;

use crate::DispatchError;

// ---------------------------------------------------------------------------
// Quality presets
// ---------------------------------------------------------------------------

/// Transcription quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Quality {
    /// Fastest — tiny model, ~75 MB, ~8% WER.
    Fast,
    /// Balanced (default) — distil-medium, ~369 MB, ~4% WER.
    Medium,
    /// Best accuracy — large-v3-turbo, ~1.5 GB, ~2% WER.
    High,
}

/// GGML model filename for a quality preset.
///
/// English-only variants are used by default. When a non-English
/// `--language` is specified, [`model_filename_multilingual`] should
/// be called instead.
pub(crate) fn model_filename(quality: Quality) -> &'static str {
    match quality {
        Quality::Fast => "ggml-tiny.en.bin",
        Quality::Medium => "ggml-distil-medium.en.bin",
        Quality::High => "ggml-large-v3-turbo.bin",
    }
}

/// GGML model filename for non-English languages.
pub(crate) fn model_filename_multilingual(quality: Quality) -> &'static str {
    match quality {
        Quality::Fast => "ggml-tiny.bin",
        Quality::Medium => "ggml-small.bin",        // distil-medium has no multilingual variant
        Quality::High => "ggml-large-v3-turbo.bin", // turbo is always multilingual
    }
}

/// Check if a language code requires multilingual models.
fn needs_multilingual(language: Option<&str>) -> bool {
    match language {
        None | Some("en") | Some("auto") => false,
        Some(_) => true,
    }
}

// ---------------------------------------------------------------------------
// Model registry
// ---------------------------------------------------------------------------

/// Base URL for GGML model downloads.
const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// A model entry in the registry.
struct ModelInfo {
    /// Quality preset name.
    preset: &'static str,
    /// GGML filename.
    filename: &'static str,
    /// Approximate file size in bytes.
    size_bytes: u64,
    /// Whether this is the English-only variant.
    english_only: bool,
}

impl ModelInfo {
    /// Download URL.
    fn url(&self) -> String {
        format!("{MODEL_BASE_URL}/{}", self.filename)
    }

    /// Human-readable size.
    fn size_display(&self) -> String {
        if self.size_bytes >= 1_073_741_824 {
            format!("{:.1} GB", self.size_bytes as f64 / 1_073_741_824.0)
        } else {
            format!("{} MB", self.size_bytes / 1_048_576)
        }
    }
}

/// Pyannote segmentation model filename.
#[cfg(feature = "diarize")]
const DIARIZE_SEGMENTATION_MODEL: &str = "segmentation-3.0.onnx";

/// Pyannote embedding model filename.
#[cfg(feature = "diarize")]
const DIARIZE_EMBEDDING_MODEL: &str = "wespeaker_en_voxceleb_CAM++.onnx";

/// Base URL for pyannote diarization model downloads (from pyannote-rs GitHub releases).
#[cfg(feature = "diarize")]
const DIARIZE_SEGMENTATION_URL: &str =
    "https://github.com/thewh1teagle/pyannote-rs/releases/download/v0.1.0/segmentation-3.0.onnx";

/// Base URL for wespeaker embedding model downloads.
#[cfg(feature = "diarize")]
const DIARIZE_EMBEDDING_URL: &str = "https://github.com/thewh1teagle/pyannote-rs/releases/download/v0.1.0/wespeaker_en_voxceleb_CAM++.onnx";

/// Speaker similarity threshold for diarization clustering.
#[cfg(feature = "diarize")]
const SPEAKER_SIMILARITY_THRESHOLD: f32 = 0.5;

/// Maximum number of distinct speakers to identify.
#[cfg(feature = "diarize")]
const MAX_SPEAKERS: usize = 10;

/// All known models.
const MODEL_REGISTRY: &[ModelInfo] = &[
    ModelInfo {
        preset: "fast",
        filename: "ggml-tiny.en.bin",
        size_bytes: 77_704_715,
        english_only: true,
    },
    ModelInfo {
        preset: "fast",
        filename: "ggml-tiny.bin",
        size_bytes: 77_691_713,
        english_only: false,
    },
    ModelInfo {
        preset: "medium",
        filename: "ggml-distil-medium.en.bin",
        size_bytes: 387_261_097,
        english_only: true,
    },
    ModelInfo {
        preset: "medium",
        filename: "ggml-small.bin",
        size_bytes: 487_601_967,
        english_only: false,
    },
    ModelInfo {
        preset: "high",
        filename: "ggml-large-v3-turbo.bin",
        size_bytes: 1_625_697_825,
        english_only: false,
    },
];

/// Print the model table for `--list-models`.
fn print_model_table() {
    println!("{:<10} {:<30} {:<10} URL", "PRESET", "MODEL", "SIZE");
    for m in MODEL_REGISTRY {
        let lang = if m.english_only { "en" } else { "multi" };
        println!(
            "{:<10} {:<30} {:<10} {}",
            format!("{} ({})", m.preset, lang),
            m.filename,
            m.size_display(),
            m.url()
        );
    }
}

// ---------------------------------------------------------------------------
// Output formats & Segment
// ---------------------------------------------------------------------------

/// Transcript output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    /// Plain text with timestamps (default).
    Txt,
    /// SubRip subtitle format.
    Srt,
    /// WebVTT subtitle format.
    Vtt,
    /// Structured JSON for AI pipelines.
    Json,
}

/// A single transcription segment with timestamps and optional speaker.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Segment {
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds.
    pub end: f64,
    /// Transcribed text.
    pub text: String,
    /// Speaker label (filled by diarization, empty otherwise).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
}

/// Full transcript JSON schema for `--output-format json`.
#[derive(Debug, Serialize)]
struct TranscriptJson<'a> {
    file: &'a str,
    duration_seconds: f64,
    language: &'a str,
    quality: &'a str,
    model: &'a str,
    segments: &'a [Segment],
    speakers: Vec<String>,
    full_text: String,
}

/// Format segments as plain text with timestamps.
fn format_txt(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| {
            let ts0 = format_timestamp_secs(s.start);
            let ts1 = format_timestamp_secs(s.end);
            let prefix = s.speaker.as_deref().map_or(String::new(), |sp| format!("[{sp}] "));
            format!("[{ts0} → {ts1}] {prefix}{}", s.text.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format segments as SubRip (.srt).
fn format_srt(segments: &[Segment]) -> String {
    segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let ts0 = format_srt_time(s.start);
            let ts1 = format_srt_time(s.end);
            let prefix = s.speaker.as_deref().map_or(String::new(), |sp| format!("[{sp}] "));
            format!("{}\n{ts0} --> {ts1}\n{prefix}{}\n", i + 1, s.text.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format segments as WebVTT (.vtt).
fn format_vtt(segments: &[Segment]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for s in segments {
        let ts0 = format_vtt_time(s.start);
        let ts1 = format_vtt_time(s.end);
        let prefix = s.speaker.as_deref().map_or(String::new(), |sp| format!("[{sp}] "));
        out.push_str(&format!("{ts0} --> {ts1}\n{prefix}{}\n\n", s.text.trim()));
    }
    out
}

/// Format segments as structured JSON.
fn format_json(
    segments: &[Segment],
    file: &str,
    duration: f64,
    language: &str,
    quality: &str,
    model: &str,
) -> Result<String, DispatchError> {
    let speakers: Vec<String> = {
        let mut seen = Vec::new();
        for s in segments {
            if let Some(sp) = &s.speaker {
                if !seen.contains(sp) {
                    seen.push(sp.clone());
                }
            }
        }
        seen
    };
    let full_text = segments.iter().map(|s| s.text.trim()).collect::<Vec<_>>().join(" ");
    let payload = TranscriptJson {
        file,
        duration_seconds: duration,
        language,
        quality,
        model,
        segments,
        speakers,
        full_text,
    };
    serde_json::to_string_pretty(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))
}

/// Format seconds as `HH:MM:SS.ss` (for txt output).
fn format_timestamp_secs(secs: f64) -> String {
    let total = secs as i64;
    let cs = ((secs - total as f64) * 100.0) as i64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}.{cs:02}")
}

/// Format seconds as SRT time `HH:MM:SS,mmm`.
fn format_srt_time(secs: f64) -> String {
    let total = secs as i64;
    let ms = ((secs - total as f64) * 1000.0) as i64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

/// Format seconds as VTT time `HH:MM:SS.mmm`.
fn format_vtt_time(secs: f64) -> String {
    let total = secs as i64;
    let ms = ((secs - total as f64) * 1000.0) as i64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

// ---------------------------------------------------------------------------
// CLI arguments
// ---------------------------------------------------------------------------

/// Arguments for `plaude transcribe`.
#[derive(Debug, Args)]
pub(crate) struct TranscribeArgs {
    /// Transcription quality preset: fast, medium (default), high.
    #[arg(long, value_enum, default_value_t = Quality::Medium)]
    quality: Quality,

    /// Output format: txt (default), srt, vtt, json.
    #[arg(long, value_enum, default_value_t = OutputFormat::Txt)]
    output_format: OutputFormat,

    /// Language hint (e.g. `en`, `de`, `ja`). Default: auto-detect.
    #[arg(long)]
    language: Option<String>,

    /// Use a custom GGML model file (overrides --quality).
    #[arg(long, value_name = "PATH")]
    model: Option<PathBuf>,

    /// Model cache directory.
    #[arg(long, value_name = "PATH", env = "PLAUDE_MODELS_DIR")]
    models_dir: Option<PathBuf>,

    /// Show available models with sizes and download URLs, then exit.
    #[arg(long)]
    list_models: bool,

    /// Identify speakers in the recording (downloads diarization models on first use).
    #[arg(long)]
    diarize: bool,

    /// Fail instead of auto-downloading a missing model.
    #[arg(long)]
    no_download: bool,

    /// One or more WAV files to transcribe.
    #[arg(required_unless_present = "list_models")]
    files: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Entry point dispatched from `main::dispatch`.
#[cfg(feature = "transcribe")]
pub(crate) fn run(args: TranscribeArgs) -> Result<(), DispatchError> {
    if args.list_models {
        print_model_table();
        return Ok(());
    }

    for file in &args.files {
        if !file.exists() {
            return Err(DispatchError::Runtime(format!(
                "WAV file not found: {}\n\nTip: List your recordings with 'plaude files list' and download with:\n  plaude files pull-one <ID> -o ~/plaud",
                file.display()
            )));
        }
    }

    let show_progress = args.output_format != OutputFormat::Json;

    let model_path = resolve_model_path(&args)?;
    if !model_path.exists() {
        ensure_model(&model_path, args.no_download)?;
    }

    if show_progress {
        eprint!("Loading model...");
    }
    let ctx = whisper_rs::WhisperContext::new_with_params(
        model_path.to_str().unwrap_or_default(),
        whisper_rs::WhisperContextParameters::default(),
    )
    .map_err(|e| DispatchError::Runtime(format!("failed to load whisper model: {e}")))?;
    if show_progress {
        eprintln!(" done");
    }

    let model_name = model_path.file_name().unwrap_or_default().to_string_lossy().into_owned();

    // Prepare diarization models if needed
    #[cfg(feature = "diarize")]
    let diarize_models = if args.diarize { Some(prepare_diarize_models(&args)?) } else { None };

    let file_count = args.files.len();

    for file in &args.files {
        if show_progress {
            if file_count > 1 {
                eprintln!("Transcribing {}...", file.display());
            } else {
                eprintln!("Transcribing...");
            }
        }
        let start_time = std::time::Instant::now();
        let segments = transcribe_file(&ctx, file, args.language.as_deref())?;
        let elapsed = start_time.elapsed();
        if show_progress && elapsed.as_secs() > 30 {
            eprintln!("Tip: use --quality fast for quicker results");
        }

        #[cfg(feature = "diarize")]
        let segments = if let Some((ref seg_model, ref emb_model)) = diarize_models {
            let mut segs = segments;
            diarize_segments(&mut segs, file, seg_model, emb_model)?;
            segs
        } else {
            segments
        };

        let duration = segments.last().map_or(0.0, |s| s.end);
        let lang = args.language.as_deref().unwrap_or("auto");
        let quality_str = match args.quality {
            Quality::Fast => "fast",
            Quality::Medium => "medium",
            Quality::High => "high",
        };

        let output = match args.output_format {
            OutputFormat::Txt => format_txt(&segments),
            OutputFormat::Srt => format_srt(&segments),
            OutputFormat::Vtt => format_vtt(&segments),
            OutputFormat::Json => format_json(&segments, &file.to_string_lossy(), duration, lang, quality_str, &model_name)?,
        };
        println!("{output}");
    }

    Ok(())
}

/// Fallback when the `transcribe` feature is disabled.
#[cfg(not(feature = "transcribe"))]
pub(crate) fn run(_args: TranscribeArgs) -> Result<(), DispatchError> {
    Err(DispatchError::Runtime(
        "transcription support is not compiled in — rebuild with `cargo build --features transcribe`".to_owned(),
    ))
}

// ---------------------------------------------------------------------------
// Core transcription
// ---------------------------------------------------------------------------

#[cfg(feature = "transcribe")]
fn transcribe_file(ctx: &whisper_rs::WhisperContext, path: &Path, language: Option<&str>) -> Result<Vec<Segment>, DispatchError> {
    let samples = read_wav_samples(path)?;

    let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    if let Some(lang) = language {
        params.set_language(Some(lang));
    }
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let mut state = ctx
        .create_state()
        .map_err(|e| DispatchError::Runtime(format!("create state: {e}")))?;

    state
        .full(params, &samples)
        .map_err(|e| DispatchError::Runtime(format!("transcription failed: {e}")))?;

    let n_segments = state.full_n_segments();
    let mut segments = Vec::new();

    for i in 0..n_segments {
        let Some(seg) = state.get_segment(i) else {
            continue;
        };
        let t0 = seg.start_timestamp();
        let t1 = seg.end_timestamp();
        let text = seg
            .to_str_lossy()
            .map_err(|e| DispatchError::Runtime(format!("segment text: {e}")))?;
        segments.push(Segment {
            start: t0 as f64 / 100.0,
            end: t1 as f64 / 100.0,
            text: text.trim().to_owned(),
            speaker: None,
        });
    }

    Ok(segments)
}

// ---------------------------------------------------------------------------
// WAV reader
// ---------------------------------------------------------------------------

#[cfg(feature = "transcribe")]
fn read_wav_samples(path: &Path) -> Result<Vec<f32>, DispatchError> {
    let reader = hound::WavReader::open(path).map_err(|e| DispatchError::Runtime(format!("failed to read WAV {}: {e}", path.display())))?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err(DispatchError::Runtime(format!(
            "expected mono WAV, got {} channels in {}",
            spec.channels,
            path.display()
        )));
    }
    // Convert i16 samples to f32 in [-1.0, 1.0] range
    let samples: Vec<f32> = reader
        .into_samples::<i16>()
        .filter_map(Result::ok)
        .map(|s| f32::from(s) / f32::from(i16::MAX))
        .collect();
    Ok(samples)
}

// ---------------------------------------------------------------------------
// Model resolution
// ---------------------------------------------------------------------------

fn resolve_model_path(args: &TranscribeArgs) -> Result<PathBuf, DispatchError> {
    if let Some(explicit) = &args.model {
        return Ok(explicit.clone());
    }
    let filename = if needs_multilingual(args.language.as_deref()) {
        model_filename_multilingual(args.quality)
    } else {
        model_filename(args.quality)
    };
    let dir = args
        .models_dir
        .clone()
        .or_else(default_models_dir)
        .ok_or_else(|| DispatchError::Runtime("cannot determine models directory — set PLAUDE_MODELS_DIR".to_owned()))?;
    Ok(dir.join(filename))
}

fn default_models_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("plaude").join("models"))
}

/// Ensure the whisper model file exists — download it if allowed.
#[cfg(feature = "transcribe")]
fn ensure_model(path: &Path, no_download: bool) -> Result<(), DispatchError> {
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    let url = format!("{MODEL_BASE_URL}/{filename}");

    let size_str = MODEL_REGISTRY
        .iter()
        .find(|m| m.filename == filename.as_ref())
        .map(|m| m.size_display())
        .unwrap_or_else(|| "unknown size".to_owned());

    eprintln!("Model '{filename}' not found locally.");
    eprintln!("Downloading {filename} ({size_str}) from huggingface.co...");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    download_file(&url, path, no_download)?;
    eprintln!("Model saved to {}", path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Speaker diarization
// ---------------------------------------------------------------------------

/// Prepare diarization model paths, downloading if needed.
#[cfg(feature = "diarize")]
fn prepare_diarize_models(args: &TranscribeArgs) -> Result<(PathBuf, PathBuf), DispatchError> {
    let dir = args
        .models_dir
        .clone()
        .or_else(default_models_dir)
        .ok_or_else(|| DispatchError::Runtime("cannot determine models directory".to_owned()))?;

    let seg_path = dir.join(DIARIZE_SEGMENTATION_MODEL);
    let emb_path = dir.join(DIARIZE_EMBEDDING_MODEL);

    if !seg_path.exists() {
        eprintln!("Downloading segmentation model...");
        if let Some(parent) = seg_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        download_file(DIARIZE_SEGMENTATION_URL, &seg_path, args.no_download)?;
    }

    if !emb_path.exists() {
        eprintln!("Downloading embedding model...");
        if let Some(parent) = emb_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        download_file(DIARIZE_EMBEDDING_URL, &emb_path, args.no_download)?;
    }

    Ok((seg_path, emb_path))
}

/// Download a file with progress bar (reused for both whisper and diarization models).
#[cfg(feature = "transcribe")]
fn download_file(url: &str, dest: &Path, no_download: bool) -> Result<(), DispatchError> {
    if no_download {
        return Err(DispatchError::Runtime(format!(
            "model not found: {}\n\nTo download manually:\n  curl -L -o {} {url}",
            dest.display(),
            dest.display()
        )));
    }

    use std::io::Read;

    let response = ureq::get(url)
        .call()
        .map_err(|e| DispatchError::Runtime(format!("download failed: {e}")))?;

    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let bar = indicatif::ProgressBar::new(total_size);
    bar.set_style(
        indicatif::ProgressStyle::with_template("{bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}] {bytes_per_sec}")
            .unwrap_or_else(|_| indicatif::ProgressStyle::default_bar()),
    );

    let tmp_path = dest.with_extension("tmp");
    let mut file =
        std::fs::File::create(&tmp_path).map_err(|e| DispatchError::Runtime(format!("failed to create {}: {e}", tmp_path.display())))?;

    let mut body = response.into_body().into_reader();
    let mut buf = [0u8; 8192];
    loop {
        let n = body
            .read(&mut buf)
            .map_err(|e| DispatchError::Runtime(format!("download read error: {e}")))?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n]).map_err(|e| DispatchError::Runtime(format!("write error: {e}")))?;
        bar.inc(n as u64);
    }
    bar.finish();
    std::fs::rename(&tmp_path, dest).map_err(|e| DispatchError::Runtime(format!("rename failed: {e}")))?;
    Ok(())
}

/// Run speaker diarization and annotate whisper segments with speaker labels.
#[cfg(feature = "diarize")]
fn diarize_segments(segments: &mut [Segment], wav_path: &Path, seg_model: &Path, emb_model: &Path) -> Result<(), DispatchError> {
    eprintln!("Identifying speakers...");

    let (samples, sample_rate) = pyannote_rs::read_wav(wav_path.to_str().unwrap_or_default())
        .map_err(|e| DispatchError::Runtime(format!("diarize WAV read: {e}")))?;

    let mut extractor =
        pyannote_rs::EmbeddingExtractor::new(emb_model).map_err(|e| DispatchError::Runtime(format!("embedding model load: {e}")))?;
    let mut manager = pyannote_rs::EmbeddingManager::new(MAX_SPEAKERS);

    let diarize_segments = pyannote_rs::get_segments(&samples, sample_rate, seg_model)
        .map_err(|e| DispatchError::Runtime(format!("segmentation failed: {e}")))?;

    // Collect speaker turns with timestamps
    let mut speaker_turns: Vec<(f64, f64, usize)> = Vec::new();
    for seg_result in diarize_segments {
        let seg = match seg_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: diarization segment error: {e}");
                continue;
            }
        };
        if let Ok(embedding) = extractor.compute(&seg.samples) {
            let speaker = if manager.get_all_speakers().len() >= MAX_SPEAKERS {
                manager.get_best_speaker_match(embedding.collect()).unwrap_or(0)
            } else {
                manager
                    .search_speaker(embedding.collect(), SPEAKER_SIMILARITY_THRESHOLD)
                    .unwrap_or(0)
            };
            speaker_turns.push((seg.start, seg.end, speaker));
        }
    }

    // Merge: assign each whisper segment the speaker of the most-overlapping turn
    for seg in segments.iter_mut() {
        seg.speaker = find_best_speaker(seg.start, seg.end, &speaker_turns);
    }

    Ok(())
}

/// Find the speaker label that overlaps most with a time range.
#[cfg(any(feature = "diarize", test))]
fn find_best_speaker(start: f64, end: f64, turns: &[(f64, f64, usize)]) -> Option<String> {
    let mut best_overlap = 0.0_f64;
    let mut best_speaker = None;
    for &(t_start, t_end, speaker_id) in turns {
        let overlap_start = start.max(t_start);
        let overlap_end = end.min(t_end);
        let overlap = (overlap_end - overlap_start).max(0.0);
        if overlap > best_overlap {
            best_overlap = overlap;
            best_speaker = Some(speaker_id);
        }
    }
    best_speaker.map(|id| format!("Speaker {}", id + 1))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_preset_fast_maps_to_tiny_en() {
        assert_eq!(model_filename(Quality::Fast), "ggml-tiny.en.bin");
    }

    #[test]
    fn quality_preset_medium_maps_to_distil_medium_en() {
        assert_eq!(model_filename(Quality::Medium), "ggml-distil-medium.en.bin");
    }

    #[test]
    fn quality_preset_high_maps_to_large_v3_turbo() {
        assert_eq!(model_filename(Quality::High), "ggml-large-v3-turbo.bin");
    }

    #[test]
    fn multilingual_fast_maps_to_tiny() {
        assert_eq!(model_filename_multilingual(Quality::Fast), "ggml-tiny.bin");
    }

    #[test]
    fn multilingual_medium_maps_to_small() {
        assert_eq!(model_filename_multilingual(Quality::Medium), "ggml-small.bin");
    }

    #[test]
    fn needs_multilingual_false_for_en() {
        assert!(!needs_multilingual(Some("en")));
        assert!(!needs_multilingual(None));
        assert!(!needs_multilingual(Some("auto")));
    }

    #[test]
    fn needs_multilingual_true_for_other_languages() {
        assert!(needs_multilingual(Some("de")));
        assert!(needs_multilingual(Some("ja")));
    }

    #[test]
    fn model_registry_has_all_presets() {
        for preset in &["fast", "medium", "high"] {
            assert!(
                MODEL_REGISTRY.iter().any(|m| m.preset == *preset),
                "missing preset {preset} in registry"
            );
        }
    }

    #[test]
    fn model_registry_urls_contain_filename() {
        for m in MODEL_REGISTRY {
            assert!(m.url().contains(m.filename), "URL doesn't contain filename for {}", m.filename);
            assert!(m.url().starts_with("https://"), "URL doesn't start with https for {}", m.filename);
        }
    }

    #[test]
    fn model_size_display_formats_correctly() {
        let small = MODEL_REGISTRY.iter().find(|m| m.filename == "ggml-tiny.en.bin").unwrap();
        assert!(small.size_display().contains("MB"), "tiny model should show MB");
        let large = MODEL_REGISTRY.iter().find(|m| m.filename == "ggml-large-v3-turbo.bin").unwrap();
        assert!(large.size_display().contains("GB"), "large model should show GB");
    }

    #[test]
    fn format_timestamp_secs_zero() {
        assert_eq!(format_timestamp_secs(0.0), "00:00:00.00");
    }

    #[test]
    fn format_timestamp_secs_complex() {
        assert_eq!(format_timestamp_secs(5045.67), "01:24:05.67");
    }

    #[test]
    fn format_srt_time_renders_comma_separator() {
        assert_eq!(format_srt_time(65.123), "00:01:05,123");
    }

    #[test]
    fn format_vtt_time_renders_dot_separator() {
        assert_eq!(format_vtt_time(65.123), "00:01:05.123");
    }

    #[test]
    fn format_txt_renders_segments() {
        let segments = vec![
            Segment {
                start: 0.0,
                end: 5.0,
                text: "Hello".to_owned(),
                speaker: None,
            },
            Segment {
                start: 5.0,
                end: 10.0,
                text: "World".to_owned(),
                speaker: None,
            },
        ];
        let out = format_txt(&segments);
        assert!(out.contains("Hello"));
        assert!(out.contains("World"));
        assert!(out.contains("→"));
    }

    #[test]
    fn format_srt_has_sequential_numbers() {
        let segments = vec![
            Segment {
                start: 0.0,
                end: 3.0,
                text: "First".to_owned(),
                speaker: None,
            },
            Segment {
                start: 3.0,
                end: 6.0,
                text: "Second".to_owned(),
                speaker: None,
            },
        ];
        let out = format_srt(&segments);
        assert!(out.contains("1\n"));
        assert!(out.contains("2\n"));
        assert!(out.contains("-->"));
    }

    #[test]
    fn format_vtt_has_header() {
        let segments = vec![Segment {
            start: 0.0,
            end: 5.0,
            text: "Test".to_owned(),
            speaker: None,
        }];
        let out = format_vtt(&segments);
        assert!(out.starts_with("WEBVTT"));
        assert!(out.contains("Test"));
    }

    #[test]
    fn format_json_produces_valid_json() {
        let segments = vec![Segment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_owned(),
            speaker: None,
        }];
        let json = format_json(&segments, "test.wav", 5.0, "en", "fast", "tiny.bin").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["file"], "test.wav");
        assert!(parsed["full_text"].as_str().unwrap().contains("Hello"));
        assert!(parsed["segments"].is_array());
        assert!(parsed["speakers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn format_json_includes_speakers_when_present() {
        let segments = vec![Segment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_owned(),
            speaker: Some("Speaker 1".to_owned()),
        }];
        let json = format_json(&segments, "test.wav", 5.0, "en", "fast", "tiny.bin").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["speakers"][0], "Speaker 1");
        assert_eq!(parsed["segments"][0]["speaker"], "Speaker 1");
    }

    #[test]
    fn format_txt_with_speaker_prefix() {
        let segments = vec![Segment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_owned(),
            speaker: Some("Speaker 1".to_owned()),
        }];
        let out = format_txt(&segments);
        assert!(out.contains("[Speaker 1]"));
    }

    #[test]
    fn find_best_speaker_selects_most_overlapping_turn() {
        let turns = vec![(0.0, 5.0, 0), (5.0, 10.0, 1), (10.0, 15.0, 0)];
        // Segment 6.0–12.0 overlaps speaker 1 for 4s, speaker 0 for 2s
        let result = find_best_speaker(6.0, 12.0, &turns);
        assert_eq!(result.as_deref(), Some("Speaker 2"));
    }

    #[test]
    fn find_best_speaker_returns_none_when_no_overlap() {
        let turns = vec![(0.0, 5.0, 0)];
        assert_eq!(find_best_speaker(10.0, 15.0, &turns), None);
    }

    #[test]
    fn find_best_speaker_empty_turns() {
        assert_eq!(find_best_speaker(0.0, 5.0, &[]), None);
    }
}
