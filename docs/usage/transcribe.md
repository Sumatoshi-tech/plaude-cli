# Transcribe — Offline Speech-to-Text

Convert recordings to text using a built-in Whisper engine. Fully offline — no external binaries, no cloud APIs.

## Quick start

```bash
plaude transcribe recording.wav
# First run auto-downloads the model (~369 MB)
```

## Quality presets

```bash
plaude transcribe recording.wav                  # default: medium
plaude transcribe --quality fast recording.wav    # fastest, ~75 MB model
plaude transcribe --quality high recording.wav    # best accuracy, ~1.5 GB model
```

| Preset | Speed | Accuracy | Model size | Best for |
|---|---|---|---|---|
| `fast` | ~2s / 30s audio | Good | 75 MB | Quick notes, testing |
| `medium` | ~5s / 30s audio | Very good | 369 MB | Daily voice memos |
| `high` | ~15s / 30s audio | Best | 1.5 GB | Important meetings |

## Speaker identification

Identify who said what:

```bash
plaude transcribe --diarize recording.wav
# [Speaker 1] [00:00:00.00 → 00:00:12.40] Hello, this is a test.
# [Speaker 2] [00:00:12.40 → 00:00:25.20] Thanks for the demo.
```

Diarization models (~53 MB) are downloaded automatically on first use.

## Output formats

```bash
plaude transcribe recording.wav                          # plain text (default)
plaude transcribe --output-format srt recording.wav      # SubRip subtitles
plaude transcribe --output-format vtt recording.wav      # WebVTT subtitles
plaude transcribe --output-format json recording.wav     # structured JSON
```

### JSON output (for AI pipelines)

```bash
plaude transcribe --output-format json --diarize recording.wav
```

```json
{
  "file": "recording.wav",
  "duration_seconds": 25.2,
  "language": "auto",
  "quality": "medium",
  "model": "ggml-distil-medium.en.bin",
  "segments": [
    {"start": 0.0, "end": 12.4, "text": "Hello...", "speaker": "Speaker 1"},
    {"start": 12.4, "end": 25.2, "text": "Thanks...", "speaker": "Speaker 2"}
  ],
  "speakers": ["Speaker 1", "Speaker 2"],
  "full_text": "Hello... Thanks..."
}
```

**Piping to AI tools:**

```bash
plaude transcribe --output-format json recording.wav | jq -r '.full_text' | llm "Summarize this"
```

## Language support

Default presets use English-only models. For other languages:

```bash
plaude transcribe --language de recording.wav
```

## All options

```
plaude transcribe [OPTIONS] <FILES>...

Arguments:
  <FILES>...              WAV files to transcribe

Options:
  --quality <PRESET>      fast, medium (default), high
  --output-format <FMT>   txt (default), srt, vtt, json
  --language <CODE>       Language hint (e.g. en, de, ja)
  --diarize               Identify speakers
  --model <PATH>          Custom GGML model file
  --models-dir <PATH>     Model cache directory
  --list-models           Show available models
  --no-download           Don't auto-download missing models
```

## Model management

Models are cached at `~/.local/share/plaude/models/`. Override with `PLAUDE_MODELS_DIR`.

```bash
plaude transcribe --list-models    # show all available models with download URLs
```

## Full workflow

```bash
plaude record start
# ... speak ...
plaude record stop
plaude files list
plaude files pull-one <ID> -o ~/plaud
plaude transcribe --quality high ~/plaud/<ID>.wav > ~/plaud/<ID>.txt

# Summarize the transcript
plaude summarize ~/plaud/<ID>.txt
plaude summarize --template action-items ~/plaud/<ID>.txt

# Fix transcription errors (optional)
plaude correct ~/plaud/<ID>.txt
```

See [LLM Integration](llm.md) for full summarization and correction documentation.

## Progress and tips

- **Phase indicators**: Loading model → Transcribing → Identifying speakers (if `--diarize`)
- **Auto-tip**: If transcription takes >30s, suggests `--quality fast`
- **JSON mode**: Progress suppressed on stderr — stdout stays clean for piping
- **Multiple files**: Per-file progress with filename prefix
