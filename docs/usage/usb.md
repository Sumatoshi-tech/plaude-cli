# USB Mass Storage transport

When the Plaud Note's "Access via USB" toggle is enabled in the phone
app, the device shows up as a VFAT mass-storage volume labelled
`PLAUD_NOTE`. This is the cheapest way to pull recordings: no BLE
pairing, no token, just `mount` and `read`.

## Deprecation warning

Plaud has announced that USB file access will be disabled in a future
firmware update (evidence: the in-app string at
`specs/re/apk-notes/3.14.0-620/auth-token.md`). Every USB-backend
command prints a one-line deprecation notice to stderr.

## Usage

The `--backend usb` flag enables the USB transport. It always
requires `--mount <PATH>` pointing at the mounted volume:

```console
$ plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE device info
warning: Plaud has announced USB will be disabled...
Device:     PLAUD
Model:      Plaud Note
Firmware:   0095 (00:47:14 Feb 28 2024)
Serial:     123456789012345678
Storage:    usb-transport-unsupported
```

Note that `battery`, `storage`, and every recording-control method
return `Unsupported` because they have no USB analogue.

## Listing and pulling recordings

```console
$ plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE files list
ID           KIND  STARTED              WAV        ASR
1775393534   note  1775393534           69280      69280
```

```console
$ plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE files pull-one 1775393534 -o ~/plaud
```

## Sync with serial scrubbing

```console
$ plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE sync ~/plaud-mirror --sanitise
```

The `--sanitise` flag zeros the `SN:<serial>` region (bytes
`0x2C..0x42`) in every WAV's `pad ` RIFF chunk before the file is
written to disk. File sizes are preserved; audio data is untouched.
The `.ASR` sidecars are copied through unchanged (they carry no
serial watermark).

## Mount discovery

M10 does **not** auto-detect the VFAT volume. The `--mount` flag is
required. Common per-OS mount points:

| OS | Typical mount path |
|---|---|
| Linux (udisks2) | `/run/media/$USER/PLAUD_NOTE` |
| macOS | `/Volumes/PLAUD_NOTE` |
| Windows | `D:\` (or whatever the USB drive letter is) |

Auto-discovery is tracked for a later milestone; for now, a shell
alias or env var suffices:

```bash
export PLAUDE_MOUNT=/run/media/$USER/PLAUD_NOTE
alias psc='plaude-cli --backend usb --mount "$PLAUDE_MOUNT"'
```

## Auth note

The USB transport does **not** require an auth token. `list_recordings`
and `read_recording` go through filesystem reads, not BLE vendor
opcodes. `device info` reads `MODEL.txt` directly.

## See also

- [`plaude-cli sync`](sync.md) — idempotent mirror with a state file
- [`plaude-cli files`](files.md) — single-file list + pull
- [`docs/protocol/file-formats.md`](../protocol/file-formats.md) — WAV + `pad ` + ASR specs
