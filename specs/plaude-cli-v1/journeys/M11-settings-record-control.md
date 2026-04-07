# M11 — Settings + recording remote control

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M11 |
| **Journey name** | "I can read and write device settings, start/stop/pause/resume recordings, toggle privacy, and read/set the device name — all from the CLI against `plaud-sim`, with text and JSON output." |
| **Primary actor** | CLI end-user managing their Plaud Note device configuration and recording pipeline. |
| **Dependencies** | M5 (BLE transport skeleton), M6 (battery + device info + `TransportProvider`). |
| **Blocks** | M12 (hardening). |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md` → M11 row. |

## Context

M6 shipped `battery` and `device info` and established the
`TransportProvider` abstraction. M7–M10 built out the file management
surface. **M11 is the long tail of vendor opcodes**: settings CRUD,
recording pipeline control, and device identity / privacy management.

The sim already implements every `Transport` method needed for M11:
`read_setting`, `write_setting`, `start_recording`, `stop_recording`,
`pause_recording`, `resume_recording`, `set_privacy`. The `BleTransport`
still returns `Error::Unsupported` for all of these — that remains until
the btleplug backend ships. M11's job is to wire the CLI surface to
the transport, test it against the sim, and document it.

### Scope-reductions (documented upfront)

- **SetDeviceName opcode (0x006B)**: not yet observed on the wire and
  not currently in the proto encoder. M11 ships `device name` (read)
  as it uses the existing `device_info().local_name`. The `--set`
  variant is deferred until we have a wire example or APK confirmation
  of the exact payload shape.
- **`plaude settings list` full-sweep iteration**: the sim returns
  `NotFound` for uninitialized settings. The CLI lists only the keys
  that have a stored value rather than iterating all 20 keys.
- **Real BLE wiring**: remains `Unsupported` on `BleTransport`.

## Customer journey (CJM)

### Phase 1 — "What settings does my device have?"

**Action**: `plaude-cli --backend sim settings list`

**Expected**: exit 0, stdout prints a table of setting name + value
pairs for every setting that has a stored value on the device. Supports
`--output text|json`.

### Phase 2 — "Read a specific setting"

**Action**: `plaude-cli --backend sim settings get enable-vad`

**Expected**: exit 0, stdout prints the setting key and its current
value. Unknown setting name → exit 2 (usage). Setting not stored on
device → exit 1 (runtime) with a message naming the key.

### Phase 3 — "Change a setting"

**Action**: `plaude-cli --backend sim settings set enable-vad true`

**Expected**: exit 0, stdout prints `enable-vad = true`. Invalid value
type → exit 2.

### Phase 4 — "Start a recording remotely"

**Action**: `plaude-cli --backend sim record start`

**Expected**: exit 0, stdout prints `recording started`. Running
`start` again → exit 1 (protocol error, already recording).

### Phase 5 — "Pause and resume"

**Action**: `plaude-cli --backend sim record pause`, then
`plaude-cli --backend sim record resume`

**Expected**: exit 0 for both. Pausing when idle → exit 1. Resuming
when not paused → exit 1.

### Phase 6 — "Stop the recording"

**Action**: `plaude-cli --backend sim record stop`

**Expected**: exit 0, stdout prints `recording stopped`. Stopping when
idle → exit 1.

### Phase 7 — "Toggle privacy"

**Action**: `plaude-cli --backend sim device privacy on` then `off`

**Expected**: exit 0 for both.

### Phase 8 — "Read the device name"

**Action**: `plaude-cli --backend sim device name`

**Expected**: exit 0, stdout prints the device local name from
`device_info().local_name`.

### Phase 9 — "Auth failure paths"

Same exit-code contract as M6: missing token → 77, rejected token → 78.
Applies to all new commands (they all require auth).

## Acceptance criteria

- [x] `plaude-cli settings list` prints name=value pairs (text + json)
- [x] `plaude-cli settings get <name>` reads a single setting
- [x] `plaude-cli settings set <name> <value>` writes a single setting
- [x] `plaude-cli record start|stop|pause|resume` controls the pipeline
- [x] `plaude-cli device privacy on|off` toggles privacy
- [x] `plaude-cli device name` reads the device name
- [x] Every command requires auth (exit 77/78 on failure)
- [x] E2e tests against `plaud-sim` for every subcommand
- [x] `docs/usage/settings.md` and `docs/usage/record.md` added
- [x] `make lint` clean, `make test` green, zero dead code

## Implementation

### Files created

- `crates/plaude-cli/src/commands/settings.rs` — settings list/get/set subcommands
- `crates/plaude-cli/src/commands/record.rs` — record start/stop/pause/resume subcommands
- `crates/plaude-cli/tests/e2e_settings.rs` — 7 e2e tests for settings
- `crates/plaude-cli/tests/e2e_record.rs` — 5 e2e tests for record
- `crates/plaude-cli/tests/e2e_device_privacy.rs` — 6 e2e tests for device privacy + name
- `docs/usage/settings.md` — settings command documentation
- `docs/usage/record.md` — record command documentation

### Files modified

- `crates/plaud-domain/src/setting.rs` — added `CommonSettingKey::from_name`, `SettingValue::parse`, `SettingValue::Display`, `UnknownSettingName`, `SettingValueParseError`
- `crates/plaud-domain/src/lib.rs` — re-exported new types
- `crates/plaud-domain/tests/setting.rs` — 10 new tests for from_name, display, parse
- `crates/plaude-cli/src/commands/mod.rs` — added `record` and `settings` modules
- `crates/plaude-cli/src/commands/device.rs` — added `Privacy` and `Name` subcommands with `OnOff` wrapper
- `crates/plaude-cli/src/commands/backend.rs` — preloaded 3 default settings in sim device
- `crates/plaude-cli/src/main.rs` — wired `Record` and `Settings` into dispatch
- `docs/usage/index.md` — updated with M11 command rows
