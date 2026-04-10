# Plaude

Offline command-line tool for the [Plaud Note](https://www.plaud.ai/) voice recorder.

Connects directly over Bluetooth — **no cloud, no phone app** needed after a one-time setup.

## What it does

- **Download recordings** from the device to your computer
- **Start/stop/pause/resume recording** remotely from the command line
- **Read battery, storage, and device settings** over Bluetooth
- **Transcribe recordings** offline via whisper.cpp (GPU-accelerated with CUDA/Vulkan)
- **Summarize recordings** using LLM (Ollama, OpenAI, Anthropic) with customizable templates
- **Correct transcripts** using LLM to fix speech-to-text errors, punctuation, and filler words
- **Sanitised export** that strips the forensic serial watermark from WAV files

## Install

Requirements: Rust 1.85+, Linux with BlueZ.

```bash
git clone https://github.com/plaude-cli/plaude-cli
cd plaude-cli
make install    # installs `plaude` to ~/.cargo/bin
```

## First-time setup

Capture the auth token from your phone's Plaud app (one-time):

```bash
# Turn OFF your real Plaud device first!
plaude auth bootstrap
# Open the Plaud app on your phone — it will connect to your computer
# Token captured. Fingerprint: a82dcb11ff56d11d
# Turn your Plaud device back on
```

Or set a token manually if you already have it:

```bash
plaude auth set-token <32-hex-chars>
```

## Usage

```bash
# Check battery (no auth needed)
plaude battery

# View device info
plaude device info

# List recordings
plaude files list

# Download a recording
plaude files pull-one <ID> -o ~/plaud-recordings

# Record remotely
plaude record start
plaude record stop

# Read/write settings
plaude settings list
plaude settings set mic-gain 20

# Sync everything
plaude sync ~/plaud-recordings

# Transcribe (GPU-accelerated, auto-downloads model)
plaude transcribe --quality high ~/plaud/recording.wav

# Summarize with LLM (Ollama, OpenAI, Anthropic)
plaude summarize ~/plaud/recording.txt
plaude summarize --template action-items ~/plaud/recording.txt

# Correct transcript errors
plaude correct ~/plaud/recording.txt

# Manage summaries
plaude summaries list ~/plaud/
plaude summaries show ~/plaud/ --template meeting-notes
```

## Documentation

Full user guide: [`docs/usage/`](docs/usage/index.md)

## Build from source

```bash
make build        # release build
make test         # run all tests
make lint         # clippy + fmt + audit
make install      # install to ~/.cargo/bin
```

## Privacy notice

1. **BLE traffic is unencrypted** — don't sync near untrusted people
2. **WAV files contain your device serial** — use `plaude sync --sanitise`
3. **The auth token is a permanent credential** — stored in your OS keyring

Run `plaude --about` for the full disclosure.

## License

MIT
