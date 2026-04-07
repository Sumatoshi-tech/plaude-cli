# M10 — `plaud-transport-usb` fallback

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M10 |
| **Journey name** | "When I enable USB mass-storage on my Plaud device, I can point `plaude-cli` at the mount path, list recordings, pull WAV+ASR pairs, and run `sync --sanitise` to scrub my device serial out of every WAV before it leaves my laptop — all without any BLE." |
| **Primary actor** | Privacy-conscious user on pre-deprecation firmware who prefers USB over BLE. |
| **Dependencies** | M1 (`Transport` trait), M6 (CLI `--backend` plumbing), M9 (`sync` command + `TransportProvider`). |
| **Blocks** | Nothing — M10 is a parallel transport that slots into the existing provider pattern. |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M10 row. |

## Context

Plaud Notes on firmware that still offers "Access via USB" enumerate
as a VFAT mass-storage volume labelled `PLAUD_NOTE`. Its layout is
documented in `docs/protocol/file-formats.md`:

- `/MODEL.txt` — 2-line ASCII file (product + build + serial)
- `/NOTES/<YYYYMMDD>/<unix>.{WAV,ASR}` — button-triggered recordings
- `/CALLS/<YYYYMMDD>/<unix>.{WAV,ASR}` — call-mode recordings

Every WAV file carries a **forensically watermarked `pad ` RIFF chunk**
at file offset `0x24` whose payload starts `SN:<18-digit-serial>\0…`.
If a user uploads these WAVs anywhere, the serial leaks. M10 ships a
`WavSanitiser` that rewrites the serial region to zeros **without
shifting any audio offsets**, and a `--sanitise` flag on
`plaude-cli sync` that applies it to every WAV pulled via any backend.

### Scope split: crate vs. CLI

- **`plaud-transport-usb` crate**: full `Transport` implementation
  against a mounted `PLAUD_NOTE` volume plus the `WavSanitiser`
  helper. Tested against a synthetic VFAT-shaped fixture.
- **`plaude-cli`**: a new `Backend::Usb` variant, a global
  `--mount <PATH>` flag consulted only by the `usb` backend, and a
  `--sanitise` flag on `sync`. No auto-discovery yet (see
  scope-reductions).

### Scope-reductions vs. literal DoD

1. **Auto-discovery of the USB mount path** (enumerate block devices,
   match VFAT label `PLAUD_NOTE`, cross-platform Linux/macOS/Windows)
   is deferred. M10 ships with an explicit `--mount <PATH>` argument
   the user passes, and hints in `docs/usage/usb.md` at common
   per-OS mount points. Auto-discovery is a heavy dependency on
   `udisks2`/`diskutil`/`GetVolumeInformation` and adds no value
   the user can't get from a shell variable.
2. **Deprecation warning** (Plaud's in-app string about USB being
   removed in a future firmware) is printed to stderr on **every**
   run of the USB backend, not just the first. First-run tracking
   across processes would require an extra state file; a per-run
   warning is noisy by design and harder to miss.
3. **`plaude-cli files list / pull-one --backend usb`** is wired
   too, not just `sync`. All three commands (`files list`,
   `files pull-one`, `sync`) go through the same `TransportProvider`
   so the backend pivot is free.
4. **Cross-platform fixture tests**: every test runs against a
   plain tempdir filled with fixture bytes. No real VFAT mount is
   required — the `UsbTransport` operates on any directory whose
   layout matches the documented schema, which keeps CI hermetic.

## Customer journey (CJM)

### Phase 1 — Device identity over USB

**Action**: enable USB access on the device, wait for it to mount,
run `plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE device info`.

**Expected**: exit 0. The printed product, firmware, and serial
match `MODEL.txt`.

### Phase 2 — Pull one recording (sanitised)

**Action**: `plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE files pull-one <id> -o ~/plaud`.

**Expected**: `<id>.wav` + `<id>.asr` land in `~/plaud`. The WAV's
`pad ` chunk bytes `0x2C..0x42` are **not** yet zeroed (pull-one is
a faithful copy in M10).

### Phase 3 — Sync everything, scrubbed

**Action**: `plaude-cli --backend usb --mount /run/media/alice/PLAUD_NOTE sync ~/plaud-mirror --sanitise`.

**Expected**: every WAV written to `~/plaud-mirror` has the SN:
region replaced with `0x00`. The file size is unchanged. The rest
of the WAV (fmt, data, audio samples) is byte-identical to the
source. The `.ASR` sidecars are copied through unchanged.

### Phase 4 — Pre-flight sanity dry run

**Action**: same but with `--dry-run`.

**Expected**: standard M9 dry-run output. `--sanitise` is silently
accepted and has no effect (no files are written in a dry-run).

### Phase 5 — Deprecation notice

**Action**: any USB-backend run prints a one-line deprecation
notice to stderr.

**Expected**: the line contains the marker string
`Plaud has announced USB will be disabled`.

## Scope

**In scope:**

- `plaud_transport_usb::UsbTransport` implementing `Transport` (real
  `device_info`, `list_recordings`, `read_recording`,
  `read_recording_asr`; every other method returns `Unsupported`)
- `plaud_transport_usb::model_txt::parse` — `MODEL.txt` parser
- `plaud_transport_usb::wav::WavSanitiser` — in-place scrubber for
  the `pad ` chunk SN region
- `plaud_transport_usb::listing` — walks NOTES + CALLS and pairs
  WAV+ASR into `Recording` values
- `Backend::Usb` variant + global `--mount <PATH>` flag
- `--sanitise` flag on `sync`
- First-line stderr deprecation notice on every USB-backend run
- `docs/usage/usb.md`

**Out of scope (deferred):**

- Auto-discovery of the VFAT mount path
- Platform-specific mount helpers for macOS / Windows
- Once-per-install deprecation notice (would need a persisted flag)
- USB transport for commands beyond `device info`, `files *`,
  `sync` — `battery`, `set_privacy`, `start_recording`, etc.
  return `Unsupported` because they have no USB analogue

## Test plan

| Path | Focus | Proves |
|---|---|---|
| `crates/plaud-transport-usb/src/model_txt.rs` unit tests | `parse` | happy path pulls product / firmware / serial; whitespace handling; error variants for malformed input |
| `crates/plaud-transport-usb/src/wav.rs` unit tests | `WavSanitiser` | byte equality before + after sanitise-region; size preserved; non-WAV input rejected; idempotent |
| `crates/plaud-transport-usb/tests/transport_usb.rs` | `UsbTransport` | `device_info` from a fixture `MODEL.txt`; `list_recordings` walks a fixture dir tree; `read_recording` + `read_recording_asr` return exact bytes; missing id → `NotFound`; every unsupported method pins its `Unsupported` contract |
| `crates/plaude-cli/tests/e2e_usb.rs` | CLI | `files list --backend usb --mount <fixture>`; `files pull-one --backend usb --mount <fixture>`; `sync --backend usb --mount <fixture> --sanitise` writes a sanitised WAV; deprecation notice on stderr |

Target coverage: ≥ 90 % on new code. All tests hermetic.

## Definition of Ready

- [x] M1 closed (`Transport` trait, domain types)
- [x] M6 closed (CLI backend plumbing)
- [x] M9 closed (sync + `TransportProvider`)
- [x] `docs/protocol/file-formats.md` pins the WAV + `pad ` chunk + MODEL.txt layouts

## Definition of Done

Mirror of the M10 DoD in `specs/plaude-cli-v1/ROADMAP.md`, adjusted
for the scope-reductions above. Updated at close.

## Implementation (closed 2026-04-06)

### Sources (new)

- [`crates/plaud-transport-usb/src/constants.rs`](../../../crates/plaud-transport-usb/src/constants.rs)
- [`crates/plaud-transport-usb/src/model_txt.rs`](../../../crates/plaud-transport-usb/src/model_txt.rs)
- [`crates/plaud-transport-usb/src/wav.rs`](../../../crates/plaud-transport-usb/src/wav.rs)
- [`crates/plaud-transport-usb/src/listing.rs`](../../../crates/plaud-transport-usb/src/listing.rs)
- [`crates/plaud-transport-usb/src/transport.rs`](../../../crates/plaud-transport-usb/src/transport.rs)
- [`crates/plaud-transport-usb/tests/transport_usb.rs`](../../../crates/plaud-transport-usb/tests/transport_usb.rs)

### Sources (modified)

- [`crates/plaud-transport-usb/src/lib.rs`](../../../crates/plaud-transport-usb/src/lib.rs) — full rewrite from stub to real module tree + re-exports
- [`crates/plaud-transport-usb/Cargo.toml`](../../../crates/plaud-transport-usb/Cargo.toml) — deps added
- [`crates/plaude-cli/src/commands/backend.rs`](../../../crates/plaude-cli/src/commands/backend.rs) — `Backend::Usb` variant, `UsbProvider`, `provider()` takes `mount: Option<&Path>`
- [`crates/plaude-cli/src/commands/sync/mod.rs`](../../../crates/plaude-cli/src/commands/sync/mod.rs) — `--sanitise` flag, `WavSanitiser` applied before WAV write
- [`crates/plaude-cli/src/commands/auth.rs`](../../../crates/plaude-cli/src/commands/auth.rs) — `Backend::Usb` match arm (usage error pointing user at sim/ble)
- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — `--mount <PATH>` global flag, `USB_DEPRECATION_NOTICE` on stderr
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — `plaud-transport-usb` dep

### Docs

- [`docs/usage/usb.md`](../../../docs/usage/usb.md)
- [`docs/usage/index.md`](../../../docs/usage/index.md) — ✅ row

### Deferred

- Auto-discovery of the VFAT mount path (Linux/macOS/Windows)
- Platform-specific mount helpers

### Quality gates

- 302 tests pass, 0 fail.
- clippy clean, fmt clean, zero unwraps/expects in production, zero `#[allow]`, every public item documented.
