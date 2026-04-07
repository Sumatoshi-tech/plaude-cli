# Transcribe — Offline Speech-to-Text

Convert recordings to text using a local [whisper.cpp](https://github.com/ggerganov/whisper.cpp) installation. Fully offline.

## Prerequisites

1. Install whisper.cpp so `whisper-cli` is on your PATH
2. Download a GGML model (e.g. `ggml-base.bin`)

## Usage

```bash
plaude transcribe --model ~/models/ggml-base.bin recording.wav
```

## Options

| Flag | Default | Description |
|---|---|---|
| `--model <PATH>` | (required) | Path to GGML model file |
| `--whisper-bin <PATH>` | `whisper-cli` | Path to whisper binary |
| `--language <LANG>` | auto-detect | Language hint (e.g. `en`, `de`) |
| `--output-format <FMT>` | `txt` | `txt`, `srt`, or `vtt` |

## Environment variables

```bash
export PLAUDE_WHISPER_BIN=/opt/whisper.cpp/build/bin/whisper-cli
export PLAUDE_WHISPER_MODEL=~/models/ggml-base.bin
plaude transcribe recording.wav
```

## Full workflow

```bash
plaude record start
# ... speak for a while ...
plaude record stop
plaude files list
plaude files pull-one <ID> -o ~/plaud
plaude transcribe --model ~/models/ggml-base.bin ~/plaud/<ID>.wav
```
