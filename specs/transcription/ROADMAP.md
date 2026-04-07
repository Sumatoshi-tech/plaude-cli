# Transcription & Diarization — Roadmap

Master implementation checklist for the transcription upgrade from
external whisper-cli shelling to in-process `whisper-rs` with quality
presets, auto-download, structured output, and speaker diarization.

**Spec:** [`SPEC.md`](SPEC.md)

## Current state

The `plaude transcribe` command exists (M15) but shells out to an
external `whisper-cli` binary. Tests use mock shell scripts. The Opus
decoder (`commands/decode.rs`) already produces playable mono 16kHz
WAV from BLE downloads — the exact format whisper-rs expects.

## Dependency on existing code

- `crates/plaude-cli/src/commands/transcribe.rs` — will be rewritten
- `crates/plaude-cli/src/commands/decode.rs` — Opus→WAV decoder, reused as-is
- `crates/plaude-cli/tests/e2e_transcribe.rs` — will be replaced

---

### T1 — In-process whisper-rs transcription with quality presets ✅ CLOSED

**Description:** Replace the external whisper-cli shelling with
`whisper-rs` as a compiled dependency behind a `transcribe` cargo
feature. Add `--quality fast|medium|high` presets that map to specific
GGML model filenames. Keep `--model <path>` for advanced users.
Audio is read as PCM i16 samples from the WAV file.

**DoR (Definition of Ready):**
- Spec reviewed and approved
- `whisper-rs` 0.16 builds on the developer's system (cmake, clang available)

**DoD (Definition of Done):**
- [ ] `whisper-rs` added as optional dep behind `transcribe` feature in `plaude-cli/Cargo.toml`
- [ ] `TranscribeArgs` updated: `--quality fast|medium|high` (default `medium`), `--language`, `--model` override
- [ ] Quality presets map to GGML filenames: `fast`→`ggml-tiny.en.bin`, `medium`→`ggml-distil-medium.en.bin`, `high`→`ggml-large-v3-turbo.bin`
- [ ] Non-English `--language` switches to multilingual model variants
- [ ] `run()` loads model via `WhisperContext::new_with_params`, creates state, calls `state.full()`, iterates segments
- [ ] Reads WAV file as `Vec<f32>` PCM samples (convert i16→f32, mono 16kHz)
- [ ] Outputs plain text with timestamps to stdout
- [ ] Existing `--whisper-bin` / `PLAUDE_WHISPER_BIN` flags removed (breaking change, documented)
- [ ] E2e test: transcribe a short WAV fixture with a real tiny model (feature-gated `#[cfg(feature = "transcribe")]`)
- [ ] Unit test: quality preset → model filename mapping
- [ ] `make lint` and `make test` pass (tests without `transcribe` feature must still pass)

**Files likely affected:**
- `Cargo.toml` (workspace dep)
- `crates/plaude-cli/Cargo.toml` (feature + dep)
- `crates/plaude-cli/src/commands/transcribe.rs` (rewrite)
- `crates/plaude-cli/src/commands/mod.rs` (feature gate)
- `crates/plaude-cli/src/main.rs` (feature gate dispatch)
- `crates/plaude-cli/tests/e2e_transcribe.rs` (rewrite)

---

### T2 — Automatic model download with progress ✅ CLOSED

**Description:** When a model file isn't present in the local cache
directory, automatically download it from HuggingFace with a progress
bar. Cache at `$XDG_DATA_HOME/plaude/models/` (default
`~/.local/share/plaude/models/`). Support `PLAUDE_MODELS_DIR` env var
and `--models-dir` flag. Add `--no-download` to fail instead of
auto-downloading. Add `--list-models` to show available models.

**DoR (Definition of Ready):**
- T1 complete (whisper-rs works with a manually placed model)

**DoD (Definition of Done):**
- [ ] Model cache directory: `dirs::data_dir()/plaude/models/`, overridable via `PLAUDE_MODELS_DIR` or `--models-dir`
- [ ] On missing model: print name + size, download from `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/<filename>` with `indicatif` progress bar
- [ ] On download failure with `--no-download`: clear error with manual `curl` command
- [ ] On download failure without `--no-download`: clear error with manual `curl` command
- [ ] `--list-models` prints table: preset name, model filename, size, download URL
- [ ] Existing cached model is reused without re-downloading
- [ ] Unit test: model path resolution (preset → cache dir + filename)
- [ ] E2e test: `--list-models` prints expected table
- [ ] `make lint` passes

**Files likely affected:**
- `crates/plaude-cli/src/commands/transcribe.rs` (model resolver + downloader)
- `crates/plaude-cli/Cargo.toml` (add `reqwest` or lightweight HTTP client)
- `Cargo.toml` (workspace dep for HTTP client)

---

### T3 — Output formats: txt, srt, vtt, json (AI-friendly) ✅ CLOSED

**Description:** Add `--output-format txt|srt|vtt|json`. The `json`
format produces structured output with `file`, `duration_seconds`,
`language`, `quality`, `model`, `segments[]` (start, end, text),
`full_text`, and `speakers[]` (empty when not diarizing). Designed
for piping to `jq` and LLM tools.

**DoR (Definition of Ready):**
- T1 complete (whisper-rs produces segments with timestamps)

**DoD (Definition of Done):**
- [ ] `--output-format txt` (default): plain text, one segment per line with timestamp prefix
- [ ] `--output-format srt`: SubRip subtitle format with sequential numbering
- [ ] `--output-format vtt`: WebVTT format with `WEBVTT` header
- [ ] `--output-format json`: structured JSON per spec §5.4 schema
- [ ] JSON includes `file`, `duration_seconds`, `language`, `quality`, `model`, `segments`, `speakers`, `full_text`
- [ ] `speakers` array is empty when `--diarize` is not used
- [ ] Unit tests for each format (given segments → assert formatted output)
- [ ] E2e test: `--output-format json` produces valid JSON parseable by `serde_json`
- [ ] `make lint` passes

**Files likely affected:**
- `crates/plaude-cli/src/commands/transcribe.rs` (formatters)

---

### T4 — Speaker diarization via pyannote-rs ✅ CLOSED

**Description:** Add `--diarize` flag that runs `pyannote-rs` after
whisper transcription to identify speakers. Merge speaker turns with
whisper segments by timestamp alignment. Output annotated with
`[Speaker 1]`, `[Speaker 2]` labels. Auto-download diarization
models (segmentation + embedding) on first use.

**DoR (Definition of Ready):**
- T1 and T2 complete (in-process transcription + auto-download)
- T3 complete (output formats support speaker field)
- `pyannote-rs` 0.3 builds on the developer's system

**DoD (Definition of Done):**
- [ ] `pyannote-rs` added as optional dep behind `diarize` cargo feature
- [ ] `--diarize` flag triggers speaker identification pipeline after transcription
- [ ] Diarization models auto-downloaded to same cache dir as whisper models
- [ ] Segmentation model: `segmentation-3.0.onnx` from pyannote HuggingFace
- [ ] Embedding model: `wespeaker_en_voxceleb_CAM++.onnx` from pyannote-rs
- [ ] Merge logic: each whisper segment gets the speaker label of the diarization turn that overlaps most
- [ ] Text output: `[Speaker N] HH:MM:SS → HH:MM:SS\ntext\n`
- [ ] JSON output: each segment includes `"speaker": "Speaker N"` field
- [ ] SRT/VTT output: speaker label prepended to each subtitle line
- [ ] Progress bar for diarization phase
- [ ] Unit test: segment-speaker merge logic with known timestamps
- [ ] E2e test: `--diarize` with a stereo fixture produces speaker labels
- [ ] `make lint` passes

**Files likely affected:**
- `Cargo.toml` (workspace dep)
- `crates/plaude-cli/Cargo.toml` (feature + dep)
- `crates/plaude-cli/src/commands/transcribe.rs` (diarization pipeline + merge)

---

### T5 — Durable UX: progress, guidance, and error messages ✅ CLOSED

**Description:** Polish the end-to-end user experience. Every phase
of the pipeline (model loading, transcription, diarization, merging)
shows a progress indicator. Every error tells the user exactly what
to do next. Add helpful tips (e.g. "Tip: use --quality fast for
quicker results").

**DoR (Definition of Ready):**
- T1–T4 complete (full pipeline works)

**DoD (Definition of Done):**
- [ ] Phase indicators: "Loading model..." ✓, "Transcribing..." progress bar, "Identifying speakers..." progress bar, "Merging..." ✓
- [ ] Missing file error includes tip about `plaude files list` + `pull-one`
- [ ] No-args error includes usage examples
- [ ] Slow transcription (>30s) prints tip: "Tip: use --quality fast for quicker results"
- [ ] Multiple files: per-file progress with filename prefix
- [ ] `--output-format json` suppresses progress to stderr (stdout stays clean JSON)
- [ ] E2e tests for error messages: no args, missing file, invalid quality
- [ ] `make lint` passes
- [ ] `docs/usage/transcribe.md` updated with all new flags and examples

**Files likely affected:**
- `crates/plaude-cli/src/commands/transcribe.rs`
- `docs/usage/transcribe.md`

---

## Dependency graph

```
T1 (whisper-rs in-process)
├──► T2 (auto-download)
├──► T3 (output formats)
│    └──► T4 (diarization) — needs T2 for model download + T3 for output
└──► T5 (UX polish) — needs T1–T4
```

T2 and T3 can be done in parallel after T1.
T4 depends on both T2 and T3.
T5 is the final polish pass.

## Out of scope (future)

- GPU acceleration (`--device gpu` flag)
- Real-time/streaming transcription
- `plaude models` top-level subcommand for cache management
- `--summarize` flag with local LLM integration
- `native-pyannote-rs` as ONNX-free alternative
- Translation mode (`--translate`)
