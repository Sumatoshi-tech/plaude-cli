# On-device file formats

## `MODEL.txt`

- **Location**: root of the VFAT volume exposed by USB MSC (`/MODEL.txt`).
- **Size observed**: 70 bytes.
- **Encoding**: ASCII, LF-terminated lines, trailing LF.
- **Access**: readable over USB MSC when "Access via USB" is enabled in the
  Plaud phone app. Not yet confirmed whether BLE or Wi-Fi exposes it.

### Schema

```
<product-name> V<firmware-build>@<build-time> <build-date>\n
Serial No.:<serial>\n
```

- **Line 1** — product + build string
  - `<product-name>` — free-form ASCII, observed: `PLAUD NOTE`.
  - `V<firmware-build>` — zero-padded build number, observed: `V0095`.
  - `<build-time>` — `HH:MM:SS`, observed: `00:47:14`.
  - `<build-date>` — `Mon DD YYYY` (C `asctime`-like), observed: `Feb 28 2024`.
- **Line 2** — `Serial No.:` followed by the device serial. Observed value
  is a 18-digit decimal string that matches the USB `iSerial` descriptor
  exactly.

### Implementation note

`MODEL.txt` is the cheapest source of `DeviceInfo` on USB. The `plaud-transport-usb`
crate should parse it with a fixed-field reader rather than a loose regex,
and should treat the build-time/build-date as a single opaque string until
we have enough samples to justify a stricter parse.

### Evidence

- [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`](../../specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md)
  — full contents (with serial redacted).

## Recording path scheme

```
/{NOTES,CALLS}/<YYYYMMDD>/<unix_seconds>.<EXT>
```

- **Top-level directory**: determined by the device's physical mode slider.
  `NOTES/` for button-triggered recordings, `CALLS/` for call-recording mode.
- **Date directory**: `YYYYMMDD` of the recording start. Created lazily by
  the device on the day of the first recording in that mode.
- **Basename**: 10-digit decimal **Unix epoch timestamp in seconds**, marking
  the start of the recording. Timezone semantics (UTC vs device-local)
  require one more sample on a different day to confirm.
- **Extensions**: `.WAV` and `.ASR`, always written as an atomic pair.

### Example (from V0095 evidence)

```
/CALLS/20260405/1775393534.WAV
/CALLS/20260405/1775393534.ASR
```

### Implementation note

`plaud-transport-usb`'s `ListRecordings` should enumerate both subtrees,
group files by basename, and surface the pair `(wav, asr)` as a single
logical `Recording` in the domain — never as two independent files.

## WAV container (V0095)

- **Codec**: Microsoft PCM, 16-bit, **stereo (2 channels)**, **16000 Hz**.
- **Block align**: 4 bytes, **bit rate**: 512 kbit/s (64000 B/s).
- **Extra RIFF chunk**: `pad ` — device-specific, described below.
- **Audio data start offset**: exactly `0x200` (512 bytes) — the header is
  hand-padded to a 512-byte boundary so sample data is flash-sector aligned.

### Chunk layout

| Offset | Size | Chunk | Notes |
|---|---|---|---|
| `0x00` | 12 | `RIFF` + size + `WAVE` | standard |
| `0x0C` | 24 | `fmt ` (16-byte payload) | standard PCM, 16-bit stereo 16 kHz |
| `0x24` | 468 | **`pad ` (460-byte payload)** | **device-specific, see below** |
| `0x1F8` | 8 | `data` header | size = sample bytes |
| `0x200` | … | PCM samples | 17.32 s observed in the baseline capture |

### `pad ` chunk payload layout (460 bytes)

| Offset (within payload) | Size | Field | Value in V0095 baseline |
|---|---|---|---|
| 0 | 3 | Magic | `SN:` |
| 3 | 18 | Device serial (ASCII decimal) | `<SERIAL>` (matches USB `iSerial`) |
| 21 | 1 | Null terminator | `0x00` |
| 22 | 438 | Zero padding | all `0x00` |

**Security note**: every WAV recording is forensically watermarked with the
device's serial number. Users very likely do not know this. The CLI's
`files pull` command must expose a `--sanitise` mode that rewrites the
`pad ` chunk to all zeros (preserving size, so audio offsets don't shift)
before the file leaves the host.

**Open question**: the 438 trailing bytes are all zero on an empty baseline.
It is possible (likely?) that longer recordings populate more of this area
with additional metadata — firmware version, timestamps, mic gain,
integrity hash. To be re-checked after a longer recording.

## `.ASR` sidecar — on-device Opus stream (hypothesis, high confidence)

- **Pairing**: always co-located with a `.WAV` of the same basename, written
  atomically with the same mtime.
- **Size model**: **constant 4000 B/s = 32 kbit/s CBR**, matching the WAV
  duration exactly. 17.32 s × 4000 B/s = 69,280 bytes — the exact observed size.
- **Frame layout**: **80-byte frames, one frame per 20 ms** (50 Hz). Every
  80-byte boundary in the file starts with byte `0xB8`.
- **Hypothesis**: each frame is a single Opus packet. `0xB8` interpreted as an
  Opus TOC byte (RFC 6716 §3.1) decodes as config = 23 (CELT-only, wideband
  16 kHz, 20 ms frame), stereo bit = 0 (mono), packet-count = 0 (one frame
  per packet). This matches the observed 80 B / 20 ms / 32 kbit/s layout and
  the fact that every frame starts with the same TOC byte (constant config ⇒
  constant TOC under CBR).
- **Why it exists**: the device performs on-device Opus encoding in parallel
  with PCM recording, so the phone app can upload a compact mono stream to
  Plaud's cloud ASR without shipping the full stereo WAV. The `.ASR`
  extension = "Automatic Speech Recognition payload".
- **Verification plan**: wrap a copy of the payload in a synthetic Ogg-Opus
  container (`ogg_page` around each 80-byte packet with `granule_pos`
  advancing 320 samples per packet) and feed to `opusinfo`. This is a
  structural check and does not require decoding audio content.
- **Implementation note**: the CLI should treat `.ASR` as an opaque sidecar
  in v1 (copy through unchanged). Demultiplexing to Ogg-Opus is a later
  feature gated on the verification above.

### Evidence

- [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`](../../specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md)
  — original USB baseline.
- [`specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md`](../../specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md)
  — first-recording capture: WAV header hex, `pad ` chunk dump, `.ASR` frame
  analysis, `ffprobe` output.
