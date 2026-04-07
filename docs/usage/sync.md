# Sync — Mirror Recordings

Mirror all recordings from the device into a local directory.

```bash
plaude sync ~/plaud-recordings
```

## How it works

1. Lists all unsynced recordings on the device
2. Downloads each one (ASR + decoded WAV)
3. Saves a state file (`.plaude-sync.json`) to track what's been synced
4. On the next run, only downloads new recordings

## Options

```bash
plaude sync ~/plaud --dry-run          # show what would be downloaded
plaude sync ~/plaud --sanitise         # strip device serial from WAV files
plaude sync ~/plaud --concurrency 1    # (reserved for future use)
```

## Dry run

Preview without downloading:

```bash
plaude sync ~/plaud --dry-run
# would pull: 1775589501
```

## Idempotent

Running sync twice is safe — already-downloaded files are skipped:

```bash
plaude sync ~/plaud
# nothing to do
```

## USB sync (faster)

For bulk downloads, USB is much faster:

```bash
plaude --backend usb --mount /run/media/$USER/PLAUD_NOTE sync ~/plaud --sanitise
```
