# `plaude-cli transcribe` — offline transcription via whisper.cpp

## Overview

The `transcribe` subcommand wraps an externally-installed
[whisper.cpp](https://github.com/ggerganov/whisper.cpp) CLI binary to
transcribe Plaud recording WAV files to text, entirely offline.

plaude-cli does **not** bundle whisper.cpp or download models. You
must install the whisper.cpp CLI and download a GGML model file
yourself.

## Prerequisites

1. Build or install `whisper.cpp` so the `whisper-cli` binary is on
   your `$PATH`, or note its path.
2. Download a GGML model file (e.g. `ggml-base.bin`) from the
   [whisper.cpp model repo](https://huggingface.co/ggerganov/whisper.cpp).

## Usage

```
plaude-cli transcribe [OPTIONS] --model <PATH> <FILES>...
```

### Required arguments

| Argument | Description |
|---|---|
| `--model <PATH>` | Path to the GGML model file. Also settable via `PLAUDE_WHISPER_MODEL`. |
| `<FILES>...` | One or more WAV file paths to transcribe. |

### Optional arguments

| Flag | Default | Description |
|---|---|---|
| `--whisper-bin <PATH>` | `whisper-cli` | Path to the whisper.cpp binary. Also settable via `PLAUDE_WHISPER_BIN`. |
| `--language <LANG>` | auto-detect | Language hint (e.g. `en`, `de`, `ja`). |
| `--output-format <FMT>` | `txt` | Output format: `txt`, `srt`, or `vtt`. |

## Examples

### Basic transcription

```bash
# Transcribe a single file
plaude-cli transcribe --model ~/models/ggml-base.bin recording.wav

# Transcribe multiple files
plaude-cli transcribe --model ~/models/ggml-base.bin *.wav
```

### With language hint

```bash
plaude-cli transcribe --model ~/models/ggml-base.bin --language en recording.wav
```

### Using environment variables

```bash
export PLAUDE_WHISPER_BIN=/opt/whisper.cpp/build/bin/whisper-cli
export PLAUDE_WHISPER_MODEL=~/models/ggml-base.bin
plaude-cli transcribe recording.wav
```

### Combined with sync

```bash
# Sync recordings, then transcribe all WAVs
plaude-cli --backend sim sync ~/plaud
plaude-cli transcribe --model ~/models/ggml-base.bin ~/plaud/*.wav
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success — transcript printed to stdout |
| 1 | Runtime error (WAV file not found, whisper process failure) |
| 2 | Usage error (model file not found, no files specified) |
| 69 | Whisper binary not found |
