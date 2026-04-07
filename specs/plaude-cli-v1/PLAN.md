# plaude-cli — Plan

Offline, device-direct CLI for the **Plaud Note** (original model) and,
eventually, other tinnotech-SDK-based OEM voice recorders. After the
first-run bootstrap, the CLI talks to the device over BLE with **zero
ongoing cloud contact, zero phone-app dependency, and zero plaud.ai
traffic**. This is live-tested, not speculative.

---

## 1. Scope and non-goals

### In scope
- Discovering and connecting to a Plaud Note over BLE without the phone app.
- One-time auth-token bootstrap (see §4) with a clean, self-contained UX.
- Full vendor command set: list/read/delete recordings, start/stop/pause
  recording, read/write every setting in `CommonSettings`, battery and
  storage stats, device name, privacy flag.
- Wi-Fi Fast Transfer bulk download triggered from the CLI, for large files.
- USB-MSC as a convenience fallback on current firmware only (deprecated).
- Local-only transcription via `whisper.cpp` (optional).
- Hermetic CI via an in-tree device simulator (`plaud-sim`).
- Evidence-backed protocol spec in `docs/protocol/` and
  `specs/re/apk-notes/`.

### Out of scope
- plaud.ai cloud integration, account login, transcription-as-a-service.
- Firmware modification, signing bypass, OTA updates.
- Any feature that requires an active Plaud subscription.
- Writing to BLE characteristics whose semantics we have not
  source-confirmed in the tinnotech SDK.

### Ethics and legality
- Personal interoperability with hardware the user owns.
- We publish the protocol spec and the client; we never publish captured
  recordings, auth tokens, or device serials.
- `docs/ethics.md` documents this stance explicitly.
- The CLI must warn that BLE traffic is cleartext and recordings are
  forensically watermarked with the device serial; both are vendor
  design decisions we surface but do not fix.

---

## 2. Target device, transports, and auth

### Target device
Plaud Note (original model, credit-card form factor), firmware V0095
or newer on the plaintext auth path. Later, Note Pro and NotePin via
the same tinnotech SDK layer.

### Transports

| Transport | Status | Role | Notes |
|---|---|---|---|
| **BLE (primary)** | Live-working with replay auth | Discovery, control, small bulk transfers, battery | Full opcode dictionary from APK (`specs/re/apk-notes/3.14.0-620/ble-protocol.md`). Live-tested end-to-end (`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`). |
| **Wi-Fi Fast Transfer** | BLE-triggered, wire format pending | Bulk file transfer for multi-megabyte recordings | Triggered by one of opcodes `0x78`–`0x7D`; phone connects to device's open hotspot. One `re-wifi-probe` session needed to finalise the wire format. |
| **USB Mass Storage** | Works on V0095, **vendor-deprecated** | Convenience fallback on legacy firmware | In-app warning string confirms USB will be disabled in a firmware update. Useful today, not tomorrow. |

### Auth model (live-confirmed)
- **Type**: single per-device pre-shared 32-hex token, cleartext on the wire.
- **Properties**: per-device (not per-phone), static for the device's
  lifetime, no nonce, no rotation, no session key, no MAC binding,
  replayable verbatim from any BLE central.
- **Acquisition**: **once** per physical device, via `plaude auth bootstrap`
  (fake-peripheral capture) OR `plaude auth import <btsnoop.log>` OR
  `plaude auth set-token <hex>` for manual paste.
- **Storage**: OS keyring (Linux Secret Service / macOS Keychain /
  Windows Credential Manager) with a file fallback at
  `~/.config/plaude/token` (mode `0600`).
- **Validation**: `0x01 0x01 0x00 <status> …`; `status = 0x00` means
  success, `0x01` means rejected. Rejected connections are kept alive
  as silent-soft-reject sessions; the CLI must refuse to proceed.
- **Firmware fallback**: if a future firmware switches to RSA + ChaCha20
  (detectable by a `0xFE12` preamble notification at connect), the CLI
  performs the fully-spec'd handshake from
  `specs/re/apk-notes/3.14.0-620/architecture.md` instead.

---

## 3. Architecture

Workspace, clean-architecture layers, transport-agnostic domain.

```
plaude-cli/
├── crates/
│   ├── plaud-domain/          # Recording, DeviceId, FileRef, Battery, Storage, Settings
│   │                          # Pure types. No IO.
│   ├── plaud-transport/       # trait Transport, DeviceDiscovery, AuthStore, error types
│   ├── plaud-proto/           # Pure codec: control + bulk frame encode/decode,
│   │                          # the 45-opcode dictionary, fixtures from the btsnoop captures.
│   │                          # 100% unit-testable, no IO.
│   ├── plaud-auth/            # Token bootstrap (fake peripheral), import from
│   │                          # btsnoop.log, OS keyring integration, manual set-token.
│   ├── plaud-transport-ble/   # btleplug-based BLE transport. Auth frame writer,
│   │                          # CCCD enable, notification demuxer, soft-reject handling.
│   ├── plaud-transport-wifi/  # Fast Transfer hotspot client, triggered from BLE.
│   ├── plaud-transport-usb/   # USB-MSC fallback for pre-deprecation firmware.
│   ├── plaud-sim/             # Device simulator: emulates the GATT tree, the
│   │                          # opcode echo protocol, the bulk-stream magic-byte demux,
│   │                          # plus the auth status codes. First-class CI dependency.
│   ├── plaud-sync/            # High-level flows: idempotent sync, resumable downloads,
│   │                          # progress reporting, state file.
│   └── plaude-cli/            # clap-derive binary, tracing, CLIG exit codes.
├── docs/
│   ├── ethics.md
│   ├── protocol/              # The reverse-engineered spec (the deliverable)
│   │   ├── overview.md        # dual-SoC, transports, security model
│   │   ├── ble-gatt.md        # GATT tree + UUID constants
│   │   ├── ble-commands.md    # 45-opcode dictionary, frame format, status codes
│   │   ├── wifi-fast-transfer.md
│   │   └── file-formats.md    # MODEL.txt, WAV pad chunk, .ASR Opus
│   └── usage/
└── specs/
    └── re/                    # Reverse-engineering working area
        ├── backlog.md
        ├── apk-notes/3.14.0-620/
        │   ├── architecture.md
        │   ├── ble-protocol.md
        │   └── auth-token.md
        └── captures/
            ├── ble-gatt/      # R0 passive recon
            ├── btsnoop/       # R2 dynamic capture (gitignored logs)
            ├── ble-live-tests/    # live token-validation tests + scripts
            ├── apk/           # APK binaries + jadx output (gitignored)
            └── usb/           # USB-MSC baseline captures
```

### Tech picks
- **btleplug** — cross-platform BLE central (BlueZ / CoreBluetooth / WinRT).
- **bluer** — BlueZ D-Bus bindings, used in `plaud-auth` for the fake-peripheral
  GATT server side (Linux-first; macOS/Windows peripheral role comes later).
- **reqwest** (rustls) for Wi-Fi hotspot HTTP where applicable; **tokio** raw
  sockets if the hotspot speaks a custom protocol.
- **axum** inside `plaud-sim` to emulate the hotspot side.
- **clap** (derive), **indicatif**, **tracing**, **thiserror**.
- **keyring** crate for token storage, with a file fallback behind a trait.
- **wiremock**/**insta** for CLI and HTTP tests.
- No `unwrap` in production. No dead code. `make lint` clean per AGENTS.md.

---

## 4. Auth bootstrap UX (the UX spec you signed off on)

### First-run command: `plaude auth bootstrap`
1. Launches a **temporary BLE peripheral** advertising `PLAUD_NOTE` with
   the Nordic manufacturer ID (`0x0059`) and the tinnotech manufacturer
   data shape we captured in R0.
2. Hosts a GATT server with vendor service `0x1910`, write char
   `0x2BB1`, notify char `0x2BB0`, CCCD `0x0011`, and the standard
   Battery Service.
3. Prints clear instructions: put your Plaud device on its side so it
   stops advertising, then open the Plaud app on your phone.
4. Waits for the phone app to connect to our fake peripheral, enable
   notifications on `0x2BB0`, and write the auth frame to `0x2BB1`.
5. Extracts the 32-hex token from the offset-6 field of that first
   write, stores it in the OS keyring, prints a success message, and
   exits cleanly (stops advertising, disconnects the phone).
6. Total time: ~60 seconds. One command. Runs exactly once per device
   lifetime.

### Alternative on-ramps
- `plaude auth import <btsnoop.log>` — extract the token from an
  existing Android HCI snoop log.
- `plaude auth set-token <hex>` — manual paste.
- `plaude auth show` — display the token fingerprint (hash prefix) for
  debugging, never the raw value.
- `plaude auth clear` — remove stored token (for device resale).

### Opportunistic capture
During `auth bootstrap`, the fake peripheral also records any
`sendHttpToken` / `sendFindMyToken` / `setSoundPlusToken` writes the
phone app may issue while it thinks it is talking to a real device.
These are bonus credentials the user can later use for the Wi-Fi
self-hosted HTTP sink work (§7 M13 below). They are stored alongside
the primary token but marked as optional.

---

## 5. Reverse-engineering state (informational)

The hard RE work is substantially done. Remaining questions are bounded
and mostly optional for v1.

- **R0 passive BLE recon**: complete. GATT tree, vendor service,
  advertising profile, RPA behaviour all documented with evidence.
- **R1 APK static analysis**: complete. 45-opcode dictionary, frame
  format, enums, two-tier security model, Flutter action API surface
  (81 actions), auth call chain all source-referenced.
- **R2 dynamic BLE capture**: complete. Resumption session, 0day re-pair
  session, annotated walkthroughs committed.
- **R2.5 live BLE interaction**: complete. Token validation tested end
  to end. The CLI's BLE transport has a known-working reference
  implementation (the Python bleak scripts under
  `specs/re/captures/ble-live-tests/scripts/`).
- **R3 Wi-Fi Fast Transfer probe**: **pending**. One `re-wifi-probe`
  session is needed to finalise the hotspot wire format and identify
  the exact opcode that triggers it. Non-blocking for M0–M9.

---

## 6. CLI surface (v1)

```
plaude auth bootstrap          # one-time fake-peripheral capture
plaude auth import <log>       # extract from existing btsnoop
plaude auth set-token <hex>    # manual paste
plaude auth show               # fingerprint only (never the raw)
plaude auth clear

plaude devices scan            # BLE + USB discovery
plaude devices info [<id>]

plaude files list [--json]
plaude files pull [--since …] [--sanitise]
plaude files pull-one <id>
plaude files delete <id>

plaude sync <dir>              # idempotent mirror, uses BLE or Wi-Fi based on size
                               # thresholds; resumable; progress bars

plaude battery                 # NO AUTH — standard SIG service
plaude device name [set <name>]
plaude device storage
plaude device privacy on|off

plaude settings list
plaude settings get <name>
plaude settings set <name> <value>
  # name ∈ {enable-vad, rec-mode, vpu-gain, mic-gain, auto-power-off,
  #        save-raw-file, auto-sync, find-my, battery-mode, …}
  # (enum decoded from tinnotech SDK)

plaude record start|stop|pause|resume

plaude transcribe <file> --model <path>   # optional whisper.cpp wrapper
```

Global flags: `--transport {auto,ble,usb,wifi}`, `--output {json,text}`,
`-v/-vv`, `--no-color`, `--config <path>`.

---

## 7. Roadmap

Each milestone ends with green e2e tests against `plaud-sim`, not real
hardware. Real-hardware bring-up runs alongside on a branch per
milestone.

| # | Milestone | Output | DoD |
|---|---|---|---|
| **M0** | Scaffold | Cargo workspace, `make lint/test/build`, CI, `plaude --help` e2e | pipeline green |
| **M1** | Domain + transport traits | `plaud-domain`, `plaud-transport` | compiles, doc tests |
| **M2** | `plaud-proto` codec (control + bulk) | 45-opcode encode/decode, fixtures from btsnoop walkthroughs | fixtures round-trip byte-for-byte |
| **M3** | `plaud-sim` v0 (in-process fake) | Canned responses for auth + common opcodes, CCCD state machine, soft-reject on bad auth | downstream tests compile against it |
| **M4** | `plaud-auth` — storage + import | Keyring, file fallback, `plaude auth import`, `plaude auth set-token`, `plaude auth show/clear` | e2e with fixture btsnoop |
| **M5** | `plaud-transport-ble` (btleplug) | Scan, connect, CCCD enable, auth frame write, status byte check, notification demux, opcode send/receive | e2e against `plaud-sim`; smoke test against real hardware on a side branch |
| **M6** | `plaude battery` + `plaude device info` | Standard SIG battery (no auth) + opcodes `0x06 GetStorage`, `0x6C GetDeviceName`, `0x03 GetState` | real-hardware battery read verified |
| **M7** | `plaude files list` + `plaude files pull-one` (BLE) | Metadata sweep via opcode `0x08 CommonSettings` iteration, `0x1C ReadFileChunk` + bulk `0x02` stream, end-of-stream sentinel handling | real-hardware small-file pull verified |
| **M8** | `plaude-auth` — fake-peripheral bootstrap | `bluer` GATT server, PLAUD_NOTE advertising, capture write on `0x2BB1`, opportunistic sidechannel capture | e2e bootstrap against a second BLE adapter emulating "the phone app" |
| **M9** | `plaude sync <dir>` | Resumable, idempotent, concurrent-safe, progress UI, partial-failure recovery, state file | e2e sync of 10 simulated recordings from sim |
| **M10** | `plaud-transport-usb` fallback | USB-MSC transport for pre-deprecation firmware; sanitise export | e2e against mounted fixture directory |
| **M11** | Settings + recording control | `plaude settings` subtree, `plaude record start/stop/pause/resume`, `plaude device privacy`, full enum-decoded `CommonSettings` | e2e for every opcode that has a matching Flutter action |
| **M12** | Hardening | Retry/backoff, timeouts, soft-reject handling, multi-device discovery, CLIG exit codes audit, man pages, `docs/usage/` complete | `make lint` clean, docs green |
| **M13** (stretch) | `re-wifi-probe` session → Wi-Fi Fast Transfer transport | Finalise hotspot wire format, identify trigger opcode, implement `plaud-transport-wifi`, `plaude sync` auto-uses Wi-Fi above size threshold | real-hardware large-file sync verified |
| **M14** (stretch) | Self-hosted HTTP sink | `plaude serve-http-sink` embedded `axum`, `plaude sync-in-idle configure`, push local credentials + HTTP token to device, device auto-uploads during charging | device walks up to laptop overnight, files appear with no user action |
| **M15** (stretch) | `plaude transcribe` | `whisper.cpp` wrapper | offline transcription e2e |
| **M16** (stretch) | Newer-firmware RSA/ChaCha20 path | Implement the alternative handshake from APK spec for firmware that no longer accepts plaintext tokens | compat with future firmware updates |

---

## 8. Stressors (residuality step 1, refreshed)

1. Plaud firmware update forces the RSA + ChaCha20 handshake path. **Mitigation**: M16, spec already extracted from APK.
2. Plaud firmware update disables USB-MSC (already announced in-app). **Mitigation**: USB is demoted to fallback from day one; BLE is primary.
3. Plaud rotates the BLE UUIDs or opcode values in a firmware update. **Mitigation**: low probability (the UUIDs are baked into a third-party SDK shared with other OEMs), but the codec crate is versioned and can ship per-firmware profiles.
4. Token storage corruption or keyring unavailable on headless systems. **Mitigation**: file fallback at `~/.config/plaude/token` mode 0600.
5. The fake peripheral's GATT server misbehaves under BlueZ's less common corner cases (multi-MTU, secure pairing demands, aggressive reconnect). **Mitigation**: `bluer` has reasonable BlueZ coverage; fall back to `bluetoothctl` scripted mode if needed.
6. The user's phone app refuses to connect to our fake peripheral because it implements some identity check beyond name + manufacturer data. **Mitigation**: advertise the exact manufacturer-data payload we observed in R0; if that's not enough, fall back to `auth import <btsnoop.log>` and document it as plan B.
7. Silent soft-reject makes bad-token errors look like "nothing happened". **Mitigation**: the transport layer checks the status byte on every auth and surfaces a specific error, never a timeout.
8. Concurrent BLE connections (phone app running at the same time as CLI) cause the CLI to fail to connect. **Mitigation**: detect and report; CLI error message tells the user to force-stop the phone app.
9. RPA rotation changes the visible MAC mid-session. **Mitigation**: match on local name + manufacturer data key, not MAC; cache the last-seen RPA as a connection hint only.
10. The CLI could be used to record a Plaud device the user does not own. **Mitigation**: documented in README as out-of-scope / user responsibility; no technical countermeasure.
11. A passive sniffer near the user during sync captures the cleartext token and recordings. **Mitigation**: documented as a privacy disclosure; no technical countermeasure short of implementing the ChaCha20 path, which requires newer firmware anyway.
12. Hermetic CI must not depend on physical hardware. **Mitigation**: `plaud-sim` is mandatory and built in M3.

---

## 9. Working agreements

- **Micro-TDD** per AGENTS.md: one failing test → minimal code → reflect → repeat.
- **No git operations** from skills or agents. All commits are explicit user acts.
- **No TODOs in code.** Implement, or carve a spec item.
- **Every `docs/protocol/` claim cites evidence.** No hand-waving.
- **No real-hardware dependency in CI.** `plaud-sim` is the north star.
- **No `unwrap` in production. No clippy warnings. `make lint` is the gate.**
- **Token is a secret.** Never print, never log, never commit.
- **Evidence tree is precious.** Raw APK, raw btsnoop, raw live captures stay
  under gitignore. Committable analysis lives under `specs/re/apk-notes/` and
  `specs/re/captures/*/*.md`.
