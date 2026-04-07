# Files — List & Download Recordings

## Listing recordings

```bash
plaude files list
# ID           KIND  STARTED              DURATION   SIZE
# 1775583902   note  2026-04-07 20:45:02  6s         26.4 KB asr
# 1775589180   note  2026-04-07 22:13:00  1m 04s     251.1 KB asr
# 1775589501   note  2026-04-07 22:18:21  25s        98.4 KB asr
```

JSON output:

```bash
plaude files list --output json
```

> **BLE note:** Only unsynced recordings appear. If the phone app already synced a recording, it won't show here. Use `--backend usb` for a complete listing.

## Downloading a recording {#downloading}

```bash
plaude files pull-one 1775589501 -o ~/plaud-recordings
```

This downloads and saves:
- `1775589501.asr` — raw Opus audio from the device
- `1775589501.wav` — decoded PCM audio (playable in any media player)

The WAV is automatically decoded from the Opus data. Duration and quality match the original recording (16-bit mono, 16 kHz).

### Progress

A progress indicator shows during download. BLE transfers are slow (~500 bytes/sec), so a 25-second recording takes about 3 minutes.

### Resume

Skip already-downloaded files:

```bash
plaude files pull-one 1775589501 -o ~/plaud-recordings --resume
# 1775589501 already up to date
```

## USB fallback

For faster downloads and full WAV files (stereo PCM), mount the device via USB:

```bash
plaude --backend usb --mount /run/media/$USER/PLAUD_NOTE files list
plaude --backend usb --mount /run/media/$USER/PLAUD_NOTE files pull-one <ID> -o ~/plaud
```

USB provides the original stereo WAV files (~1 MB per 15 seconds) instead of the compressed ASR sidecar.
