# SPEC: Offline Transcription & Speaker Diarization for Plaude

## 1. Summary

Plaude needs best-in-class offline speech-to-text with optional speaker diarization for Plaud Note recordings (mono 16kHz 16-bit PCM WAV). The current implementation shells out to an external `whisper-cli` binary. This spec evaluates all viable approaches and recommends a Rust-native integration using `whisper-rs` for transcription and `pyannote-rs` for speaker diarization — both as compiled Rust dependencies, no external binaries needed.

## 2. Background & Research

### Market Context

| Product | Approach | Diarization | Offline | Notes |
|---|---|---|---|---|
| **whisper.cpp** | C++ standalone binary | tinydiarize (experimental, small.en only) | Yes | De facto standard. 79k GitHub stars. |
| **WhisperX** | Python (faster-whisper + pyannote) | Yes (pyannote 3.1) | Yes | Best combined pipeline but Python-only |
| **Vosk** | C library with FFI bindings | Basic speaker ID | Yes | Lighter models, lower accuracy than Whisper |
| **Candle (HuggingFace)** | Pure Rust ML framework | No | Yes | Whisper example exists but immature for production |

### Technical Context — Rust STT Landscape

| Crate | Type | Maturity | Downloads | Last release |
|---|---|---|---|---|
| **whisper-rs** (0.16.0) | FFI bindings to whisper.cpp | High — 245k downloads, 46 reverse deps | 245,022 | 2026-03-12 |
| **vosk-rs** | FFI bindings to Vosk | Low — limited docs | ~5k | 2023 |
| **candle-whisper** | Pure Rust via candle framework | Example-quality | N/A (part of candle) | Active |
| **rwhisper** | Higher-level wrapper | Low | ~2k | 2024 |

### Speaker Diarization — Rust Options

| Crate | Type | Approach | Performance |
|---|---|---|---|
| **pyannote-rs** (0.3.4) | ONNX Runtime | segmentation-3.0 + wespeaker embeddings | 1 hour in <1 min on CPU |
| **native-pyannote-rs** | Pure Rust (Burn framework) | Same models, no ONNX dep | 1 hour in <1 min on CPU |
| **tinydiarize** (via whisper.cpp) | Built into whisper | Special tokens for speaker turns | small.en only, experimental |

### Deep Dives

**whisper-rs is the clear winner for STT.** 245k downloads, actively maintained (migrated to Codeberg 2025), supports all Whisper models including distil variants. The API is clean: load model → create state → `state.full(params, audio)` → iterate segments. Audio format requirement matches Plaude exactly: mono 16-bit 16kHz WAV.

**Note:** The GitHub repo is archived (migrated to Codeberg), but the crate continues to be published and maintained.

**pyannote-rs is the clear winner for diarization.** Two variants exist:
- `pyannote-rs` — uses ONNX Runtime (more mature, GPU acceleration via DirectML/CoreML)
- `native-pyannote-rs` — pure Rust via Burn framework (no C++ deps, same accuracy)

Both process 1 hour of audio in under 1 minute on CPU. The architecture: segmentation model (10s sliding window) → speaker embeddings (wespeaker) → cosine similarity clustering.

**Model recommendations for voice recorder audio:**
- **English only:** `distil-medium.en` — 6x faster than large, within 1% WER, best for meetings/memos
- **Multilingual:** `small` — good accuracy/speed balance on CPU
- **With diarization:** `small.en-tdrz` — tinydiarize finetuned, but only English and experimental
- **Best quality:** `large-v3-turbo` — nearly large-v3 accuracy at 3x speed, but needs decent hardware

## 3. Proposal

### Approach

Replace the external `whisper-cli` shelling with `whisper-rs` as a compiled Rust dependency. Add optional speaker diarization via `pyannote-rs`. Both are feature-gated so they don't bloat the binary for users who don't need transcription.

### Key Decisions

| Decision | Choice | Reasoning | Alternatives |
|---|---|---|---|
| STT engine | `whisper-rs` (FFI to whisper.cpp) | 245k downloads, mature API, all models supported, exact audio format match | candle-whisper (immature), vosk-rs (lower accuracy), keep shelling out (fragile) |
| Diarization | `pyannote-rs` (ONNX) | Proven accuracy (pyannote 3.1 is SOTA), 1hr < 1min, actively maintained | native-pyannote-rs (newer, less tested), tinydiarize (English only, experimental), none |
| Default model | `distil-medium.en` | 6x faster than large, <1% WER difference, optimal for voice memos | small (multilingual), large-v3-turbo (highest quality), tiny (fastest) |
| Feature gating | `transcribe` cargo feature, off by default | whisper.cpp adds ~15MB to binary; users who don't transcribe shouldn't pay | always-on (bloat), separate binary (UX complexity) |
| Model management | User downloads models manually, `--model` flag | Avoids bundling 50MB+ models, user controls disk usage | Auto-download (network dependency), bundle (huge binary) |

### ML (Minimum Loveable)

**IN:**
- `plaude transcribe <file.wav>` with `whisper-rs` (no external binary)
- `--model <path>` to specify GGML model file
- `--language <code>` for language hint
- `--output-format txt|srt|vtt`
- `--diarize` flag that runs pyannote-rs and annotates output with `[Speaker 1]`, `[Speaker 2]` labels
- `plaude transcribe --list-models` showing recommended models with download URLs

**OUT:**
- Automatic model downloading (user responsibility)
- Real-time/streaming transcription (batch only)
- GPU acceleration (CPU-first, GPU as future enhancement)
- Training or fine-tuning
- Translation (transcription only)

### Anti-Goals

- **No Python dependency.** The whole point of Rust-native is avoiding Python.
- **No model bundling.** Models are 50MB–3GB. Users download what they need.
- **No cloud fallback.** This is an offline tool. Period.

## 4. Technical Design

### Architecture

```
plaude transcribe <files...>
    │
    ├── Load GGML model via whisper-rs
    ├── Read WAV file (already decoded PCM from our Opus decoder)
    ├── Run whisper inference → segments with timestamps
    │
    ├── [if --diarize]
    │   ├── Load pyannote ONNX models
    │   ├── Run segmentation → speaker turns with timestamps
    │   ├── Run embedding → speaker identities
    │   └── Merge: align whisper segments with speaker turns
    │
    └── Format output (txt/srt/vtt with optional speaker labels)
```

### Dependencies

| Crate | Version | Feature-gated? | Size impact |
|---|---|---|---|
| `whisper-rs` | 0.16 | Yes (`transcribe`) | ~15MB (links whisper.cpp statically) |
| `pyannote-rs` | 0.3 | Yes (`diarize`) | ~20MB (ONNX Runtime) |

### Non-Functional Requirements

- **Performance:** Transcribe a 30-second recording in <10 seconds on a modern laptop CPU with `distil-medium.en`
- **Memory:** <500MB peak for `distil-medium.en`; <2GB for `large-v3`
- **Reliability:** Graceful error on corrupt audio, missing model file, unsupported format
- **Observability:** Progress bar during transcription; `RUST_LOG=debug` for timing details

### Testing Strategy

- **Unit:** Model loading, segment formatting, diarization merge logic
- **Integration:** Transcribe a known WAV fixture, assert output contains expected words
- **E2E:** `plaude transcribe --model <path> fixture.wav` produces non-empty text output

## 5. UX Design

### 5.1 Quality Presets — Abstract Users From Models

Users should never need to know model names. Instead, a `--quality` flag maps to the right model automatically.

```bash
plaude transcribe recording.wav                  # default: --quality medium
plaude transcribe --quality fast recording.wav    # fastest, ~75 MB model
plaude transcribe --quality high recording.wav    # best accuracy, ~1.5 GB model
```

| Preset | Model | Size | Speed (30s audio) | WER | Use case |
|---|---|---|---|---|---|
| `fast` | `ggml-tiny.en.bin` | 75 MB | ~2s | ~8% | Quick notes, testing |
| `medium` (default) | `ggml-distil-medium.en.bin` | 369 MB | ~5s | ~4% | Daily voice memos |
| `high` | `ggml-large-v3-turbo.bin` | 1.5 GB | ~15s | ~2% | Important meetings |

Advanced users can still bypass presets with `--model <path>` for custom models.

**Language detection:** The default presets use English-only models (`.en` variants). For other languages, pass `--language <code>` which automatically switches to multilingual model variants:

```bash
plaude transcribe --language de recording.wav     # uses multilingual model
```

### 5.2 Automatic Model Download

When a model isn't present locally, plaude downloads it automatically with full progress indication. No manual steps.

**First-time experience:**

```
$ plaude transcribe recording.wav

Model 'distil-medium.en' not found locally.
Downloading ggml-distil-medium.en.bin (369 MB) from huggingface.co...
████████████████████████████████ 369/369 MB  [00:42]  8.8 MB/s
Model saved to ~/.local/share/plaude/models/ggml-distil-medium.en.bin

Transcribing recording.wav (25.2s of audio)...
████████████████████████████████ 100%  [00:05]

Hello, this is a test recording from the Plaud Note device...
```

**Subsequent runs — instant, no download:**

```
$ plaude transcribe recording.wav
Transcribing recording.wav (25.2s of audio)...
████████████████████████████████ 100%  [00:05]

Hello, this is a test recording from the Plaud Note device...
```

**Model cache location:** `$XDG_DATA_HOME/plaude/models/` (defaults to `~/.local/share/plaude/models/`). Override with `PLAUDE_MODELS_DIR` env var or `--models-dir <PATH>`.

**Offline mode:** If the model is missing and there's no network, show a clear error with the manual download URL:

```
$ plaude transcribe recording.wav
Error: Model 'distil-medium.en' not found and download failed (no network).

To download manually:
  curl -L -o ~/.local/share/plaude/models/ggml-distil-medium.en.bin \
    https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-distil-medium.en.bin

Or set PLAUDE_MODELS_DIR to point to your existing models.
```

### 5.3 Durable UX Flow With Guidance

Every step of the transcription pipeline provides clear feedback and helpful error messages.

**Progress phases:**

```
$ plaude transcribe --quality high --diarize recording.wav

Loading model 'large-v3-turbo' (1.5 GB)...        ✓  [0.8s]
Transcribing recording.wav (25.2s of audio)...     ████████████████ 100%  [00:15]
Identifying speakers...                            ████████████████ 100%  [00:03]
Merging transcript with speaker labels...          ✓

[Speaker 1] 00:00:00 → 00:00:12
Hello, this is a test recording from the Plaud Note device.

[Speaker 2] 00:00:12 → 00:00:25
Thanks for the demo. The audio quality is really clear.
```

**Error guidance — every failure tells you exactly what to do:**

```
$ plaude transcribe
Error: No audio files specified.

Usage: plaude transcribe [OPTIONS] <FILES>...

Examples:
  plaude transcribe recording.wav
  plaude transcribe --quality high --diarize ~/plaud/*.wav
```

```
$ plaude transcribe nonexistent.wav
Error: File not found: nonexistent.wav

Tip: List your recordings with 'plaude files list' and download with:
  plaude files pull-one <ID> -o ~/plaud
```

```
$ plaude transcribe recording.wav --diarize
Error: Speaker diarization models not found.

Downloading pyannote segmentation model (18 MB)...
████████████████████████████████ 100%
Downloading wespeaker embedding model (35 MB)...
████████████████████████████████ 100%
Models saved to ~/.local/share/plaude/models/

Transcribing with speaker identification...
```

### 5.4 AI-Friendly Structured Output

The `--output-format json` flag produces structured JSON designed for downstream AI pipelines (summarization, search, RAG).

```bash
plaude transcribe --output-format json --diarize recording.wav
```

```json
{
  "file": "recording.wav",
  "duration_seconds": 25.2,
  "language": "en",
  "quality": "medium",
  "model": "distil-medium.en",
  "segments": [
    {
      "start": 0.0,
      "end": 12.4,
      "text": "Hello, this is a test recording from the Plaud Note device.",
      "speaker": "Speaker 1"
    },
    {
      "start": 12.4,
      "end": 25.2,
      "text": "Thanks for the demo. The audio quality is really clear.",
      "speaker": "Speaker 2"
    }
  ],
  "speakers": ["Speaker 1", "Speaker 2"],
  "full_text": "Hello, this is a test recording from the Plaud Note device. Thanks for the demo. The audio quality is really clear."
}
```

**Design choices for AI consumption:**

- **`full_text`** — concatenated transcript for simple LLM prompts
- **`segments`** with timestamps — for precise referencing and summarization
- **`speaker`** per segment — for meeting-notes AI that needs "who said what"
- **Flat structure** — no nesting beyond segments array; easy to parse with `jq`
- **Metadata** (duration, language, model) — for pipeline routing and quality assessment

**Piping to AI tools:**

```bash
# Summarize with an LLM
plaude transcribe --output-format json recording.wav | \
  jq -r '.full_text' | \
  llm "Summarize this meeting transcript in 3 bullet points"

# Extract action items
plaude transcribe --output-format json --diarize recording.wav | \
  llm "Extract action items with owners from this transcript"

# Search across recordings
for f in ~/plaud/*.wav; do
  plaude transcribe --output-format json "$f"
done | jq -s '.' > all_transcripts.json
```

### 5.5 Complete CLI Surface

```
plaude transcribe [OPTIONS] <FILES>...

Arguments:
  <FILES>...              WAV files to transcribe

Options:
  --quality <PRESET>      Transcription quality: fast, medium (default), high
  --language <CODE>       Language hint (e.g. en, de, ja). Default: auto-detect
  --diarize               Identify speakers (downloads models on first use)
  --output-format <FMT>   Output format: txt (default), srt, vtt, json
  --model <PATH>          Use a custom GGML model file (overrides --quality)
  --models-dir <PATH>     Model cache directory (default: ~/.local/share/plaude/models)
  --list-models           Show available models with sizes and download URLs
  --no-download           Fail instead of auto-downloading missing models
```

## 6. User Journey

### Persona
Non-technical professional who records meetings and wants text transcripts.
Also: developer piping transcripts into AI summarization tools.

### CJM Phases

| Phase | User action | System response | Success signal |
|---|---|---|---|
| **Record** | Press button on Plaud device | Device records | LED indicator |
| **Download** | `plaude files pull-one <ID>` | Downloads + decodes WAV | File on disk |
| **Transcribe (first time)** | `plaude transcribe recording.wav` | Auto-downloads model, transcribes | Text on stdout |
| **Transcribe (repeat)** | `plaude transcribe recording.wav` | Instant — model cached | Text in <10s |
| **With speakers** | `plaude transcribe --diarize ...` | Downloads diarization models, runs pipeline | Speaker-labeled text |
| **AI pipeline** | `plaude transcribe --output-format json ...` | Structured JSON | Pipe to `jq` / LLM |

### Friction Map

| Friction | Phase | Solution |
|---|---|---|
| "Which model do I pick?" | First use | `--quality` presets — user never sees model names |
| "I have to download something first?" | First use | Auto-download with progress bar |
| "It's slow on my laptop" | Transcribe | Default `medium` preset; suggest `fast` in slow-hardware message |
| "I want speaker names, not numbers" | Diarize | Future: `--speaker-names "Alice,Bob"` mapping |
| "How do I use this with ChatGPT?" | AI pipeline | `--output-format json` with `full_text` field |

## 7. Risks & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| whisper-rs build complexity (C++ compilation) | Build breaks on some systems | Medium | Feature-gate; document build prereqs (cmake, clang) |
| ONNX Runtime portability | Doesn't build on all Linux distros | Low | pyannote-rs handles this; fallback to native-pyannote-rs |
| Model size discourages users | Low adoption | Medium | Auto-download removes friction; `fast` preset is only 75 MB |
| whisper-rs archived on GitHub | Maintenance concerns | Low | Active on Codeberg; crates.io releases continue |
| Auto-download fails (corporate firewall) | Broken first-run | Low | `--no-download` flag + manual URL in error message |
| Large model fills disk | User frustration | Low | Show size before download; `plaude models list` shows cache usage |

## 8. Open Questions

1. Should `plaude models` become a top-level subcommand for managing the model cache (list, download, delete)?
2. Should `--diarize` output format be `[Speaker 1]: text` or configurable?
3. Should we support `native-pyannote-rs` (pure Rust, no ONNX) as an opt-in alternative?
4. Should there be a `--summarize` flag that pipes to a local LLM (e.g. Ollama)?

## 9. Implementation Roadmap

| Phase | Scope | Effort |
|---|---|---|
| **Phase 1** | Replace whisper-cli with `whisper-rs` in-process, `--quality` presets | 2 days |
| **Phase 2** | Auto-download model management with progress | 1 day |
| **Phase 3** | `--output-format json` with AI-friendly schema | 0.5 day |
| **Phase 4** | `--diarize` via `pyannote-rs` with speaker labels | 2–3 days |
| **Phase 5** | Durable error messages and guidance for every failure path | 0.5 day |
