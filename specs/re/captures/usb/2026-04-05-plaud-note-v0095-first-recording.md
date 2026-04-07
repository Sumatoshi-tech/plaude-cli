# First recording capture — PLAUD NOTE V0095

- **Date**: 2026-04-05
- **Device**: PLAUD NOTE, firmware V0095
- **Serial**: `<SERIAL>` (redacted; identical to USB baseline capture)
- **Action**: user pressed the record button once on the physical device,
  recorded ~17 seconds, stopped.
- **Device mode slider**: on CALLS (recording landed in `CALLS/`, not `NOTES/`).

## Directory state after recording

```
/MODEL.txt                           70 B    (unchanged)
/NOTES/                              empty
/CALLS/20260405/                     new date directory, created by device
/CALLS/20260405/1775393534.WAV       1,108,992 B
/CALLS/20260405/1775393534.ASR          69,280 B
```

- `/CALLS/20260405/` was created by the device, not pre-existing.
- WAV and ASR share the same basename and identical mtime → they are written
  atomically as a pair by the device firmware.

## Filename decode

- **Basename**: `1775393534` — 10-digit decimal, interpreted as a Unix epoch
  timestamp in seconds. Decodes to 2026-04-05 ~18:52 in some timezone.
  File mtime is `2026-04-05 18:52:32 +0300` (host local). UTC vs device-local
  semantics require a second sample on a different day to disambiguate.
- **Directory name**: `20260405` = `YYYYMMDD` of recording start.

## `.WAV` header — full annotated hex (first 64 bytes)

```
00000000  52 49 46 46 f8 eb 10 00  57 41 56 45 66 6d 74 20  |RIFF....WAVEfmt |
00000010  10 00 00 00 01 00 02 00  80 3e 00 00 00 fa 00 00  |.........>......|
00000020  04 00 10 00 70 61 64 20  cc 01 00 00 53 4e 3a 38  |....pad ....SN:8|
00000030  38 38 33 31 37 33 30 32  34 33 31 30 31 37 38 38  |8831730243101788|
```

Decoded:

| Offset | Bytes | Meaning |
|---|---|---|
| `0x00` | `52 49 46 46` | `RIFF` |
| `0x04` | `f8 eb 10 00` | LE u32 = 1,108,984 = filesize − 8 ✓ |
| `0x08` | `57 41 56 45` | `WAVE` |
| `0x0C` | `66 6d 74 20` | `fmt ` chunk id |
| `0x10` | `10 00 00 00` | chunk size = 16 |
| `0x14` | `01 00` | AudioFormat = 1 (PCM) |
| `0x16` | `02 00` | NumChannels = 2 |
| `0x18` | `80 3e 00 00` | SampleRate = 16000 |
| `0x1C` | `00 fa 00 00` | ByteRate = 64000 (= 16000·2·2) ✓ |
| `0x20` | `04 00` | BlockAlign = 4 |
| `0x22` | `10 00` | BitsPerSample = 16 |
| `0x24` | `70 61 64 20` | **`pad ` — custom chunk** |
| `0x28` | `cc 01 00 00` | pad chunk size = 460 |
| `0x2C` | `53 4e 3a …` | **`SN:<serial>`** |

## `.WAV` `pad ` chunk — full dump (468 bytes at offset 0x24)

```
00000024  70 61 64 20 cc 01 00 00  53 4e 3a 38 38 38 33 31  |pad ....SN:88831|
00000034  37 33 30 32 34 33 31 30  31 37 38 38 31 00 00 00  |7302431017881...|
00000044  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
*
000001f4  00 00 00 00 64 61 74 61  00 ea 10 00 fe ff 58 15  |....data......X.|
```

- Bytes `0x24..0x2C` — chunk header (`pad `, size 460).
- Bytes `0x2C..0x41` — `SN:<serial>` (21 bytes).
- Byte `0x41` — null terminator.
- Bytes `0x42..0x1F7` — all `0x00` (438 zero bytes).
- Byte `0x1F8` — `data` chunk begins.
- **Byte `0x200` — first PCM sample.** Audio is aligned to a 512-byte
  boundary by construction; the `pad ` chunk size is chosen to make this so.

Interpretation: `pad ` serves two purposes simultaneously — it forensically
watermarks the file with the device serial, and it flash-sector-aligns
the audio payload for write performance. The 438 trailing zero bytes are
headroom, likely filled in by longer recordings or future firmware.

## `.WAV` stream metadata (from `ffprobe`)

```
codec_name       = pcm_s16le
sample_rate      = 16000
channels         = 2
bits_per_sample  = 16
bit_rate         = 512000
duration         = 17.320000
size             = 1108992
nb_streams       = 1
format_name      = wav
```

Consistency check: `duration × bit_rate / 8 = 17.32 × 64000 = 1,108,480`
payload bytes + `0x200` header bytes = `1,108,992` — matches the on-disk
size exactly. No hidden trailing bytes.

## `.ASR` sidecar — first 512 bytes

```
00000000  b8 5b 4d 65 95 98 21 bf  b3 f4 bc 36 4f 83 ba a3  |.[Me..!....6O...|
00000010  c0 c2 12 5f 19 82 d2 02  4c ae f6 e0 48 93 ba ab  |..._....L...H...|
00000020  74 92 e0 57 af 1c c5 11  87 35 7b c3 ea 0d 5d 06  |t..W.....5{...].|
00000030  df a5 32 1a 48 1e 32 2c  94 90 cd 4a b5 f2 c6 16  |..2.H.2,...J....|
00000040  b3 e7 a3 1c ea 12 52 48  e5 ec e1 d0 e7 aa 98 fb  |......RH........|
00000050  b8 63 99 ca 94 78 a2 38  15 12 f3 68 35 47 1c 4a  |.c...x.8...h5G.J|
00000060  d7 b4 20 c0 d7 d4 8c 82  67 b6 c6 ae 05 6e c5 47  |.. .....g....n.G|
00000070  b3 a9 17 6d bd df 79 a3  1c 8c fd 18 39 62 82 f5  |...m..y.....9b..|
00000080  2a 9b b8 4c 77 59 69 c7  f2 da f9 d5 dd b2 af 25  |*..LwYi........%|
00000090  a2 6e 37 ce 1c 6d 44 60  8e f6 6e d9 21 72 b5 c1  |.n7..mD`..n.!r..|
000000a0  b8 41 03 5d 2a 26 1a 87  35 d1 b6 d7 fa b9 ca bb  |.A.]*&..5.......|
...
000000f0  b8 7d 54 d3 …                                    |.}T…            |
00000140  b8 7f 7b db …
00000190  b8 63 e3 de …
000001e0  b8 7e a3 90 …
```

### Structural analysis

- **File size**: 69,280 bytes exact.
- **Divisibility**: `69280 = 866 × 80` exactly.
- **Frame rate check**: `866 frames ÷ 17.32 s = 50.0 frames/s` exactly.
- **Per-frame offset**: every offset at a multiple of 80 bytes
  (`0x00, 0x50, 0xA0, 0xF0, 0x140, 0x190, 0x1E0, …`) has `0xB8` as its
  leading byte.
- **Implied format**: **80-byte CBR frames, 50 fps, 32 kbit/s, single-byte
  sync marker `0xB8` at frame start**.

### Opus CELT hypothesis

Interpreting `0xB8` as an **Opus TOC byte** (RFC 6716 §3.1):

```
0xB8 = 1011 1 000
       config=23  s=0  c=00
```

- config 23 = CELT-only, wideband (16 kHz), **20 ms** frame
- s = 0 → mono
- c = 00 → one Opus frame per packet

That is fully consistent with: 20 ms per frame × 50 fps, mono (matches the
"ASR upload" purpose), 16 kHz (matches the WAV sample rate), and a constant
TOC byte across every packet (CBR with fixed config).

**Conclusion**: the `.ASR` file is almost certainly a raw sequence of Opus
CELT packets, produced by an on-device Opus encoder running in parallel to
the WAV recorder. The device stores both a high-fidelity stereo PCM WAV for
local playback and a compact mono Opus stream for efficient upload to
Plaud's cloud ASR (hence the `.ASR` extension).

### Verification (not yet done)

- Wrap each 80-byte packet in an Ogg-Opus page with `granule_pos` advancing
  by 320 samples (= 20 ms × 16 kHz) and run `opusinfo` on the result.
- Compare the derived parameters against the Opus TOC byte interpretation.
- Do **not** decode audio content as part of verification — parameter
  extraction is sufficient.

## Implications for the roadmap

- `ListRecordings (USB)`, `ReadRecording (USB)`, `Audio container ID`,
  `Filename scheme` all move to resolved in the backlog.
- New backlog items:
  1. Decode the remaining 438 bytes of the `pad ` chunk across longer /
     differently-configured recordings.
  2. Verify the ASR Opus hypothesis with `opusinfo` on a synthetic Ogg wrap.
  3. Add a `--sanitise` mode to `plaude files pull` that zeros the `SN:`
     bytes in the `pad ` chunk without shifting audio offsets.
  4. Confirm basename timezone semantics (UTC vs device-local) with a
     second recording on a different day.
  5. Determine whether `NOTES/` uses the same layout (blocked on the user
     flipping the mode slider).