# Reverse-engineering backlog

Working list of Plaud Note capabilities we want to understand and document.
Each row carries an evidence matrix across four axes: USB filesystem, BLE
GATT dump, btsnoop dynamic capture, APK static analysis, Wi-Fi pcap. A row
is *resolved* once at least two independent axes have evidence and `re-spec`
has produced a spec section citing both.

Legend: ✅ done, 🟡 partial, ⬜ not started, — not applicable.

## Resolved

| Capability | USB | BLE GATT | btsnoop | APK | Wi-Fi | Spec |
|---|---|---|---|---|---|---|
| `DeviceInfo` (USB) | ✅ `MODEL.txt` schema | ⬜ | — | ⬜ | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| `ListRecordings` (USB) | ✅ VFAT walk of `{NOTES,CALLS}/<YYYYMMDD>/` | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| `ReadRecording` (USB) | ✅ paired `<epoch>.{WAV,ASR}` | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| Filename scheme | ✅ `<MODE>/<YYYYMMDD>/<unix_seconds>.<EXT>` | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| WAV container layout | ✅ stereo PCM 16-bit 16 kHz + `pad ` chunk at `0x24`, data at `0x200` | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| WAV `pad ` chunk | ✅ `SN:<serial>\0` + 438 B zero padding | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |
| `.ASR` sidecar (hypothesis) | 🟡 Opus CELT WB 20 ms mono CBR 32 kbit/s; 866×80 B frames, TOC=`0xB8` | — | — | — | — | [`file-formats.md`](../../docs/protocol/file-formats.md) |

## Candidates — USB transport (demoted to fallback)

**USB is deprecated by vendor.** An in-app warning string in `libapp.so`
announces "USB file access and RAW file features will be permanently
disabled" in an upcoming firmware update. Treat USB as a convenience
fallback for current firmware, not a primary transport for new code.
BLE is the future. See [`specs/re/apk-notes/3.14.0-620/auth-token.md`](apk-notes/3.14.0-620/auth-token.md)
for the string reference.

| Capability | Status | Next action |
|---|---|---|
| `DeleteRecording` (USB) | ⬜ | Verify VFAT unlink reclaims space cleanly, device re-enumerates with no complaints. Requires a throwaway recording. |
| `--sanitise` export mode | ⬜ | Design the write path that zeros the `SN:` bytes in `pad ` without shifting audio offsets. Unit-testable in `plaud-proto` against a fixture. |
| Pad-chunk trailing-bytes content | ⬜ | Capture a longer recording (>1 min) and a second shorter one; diff the last 438 bytes of `pad ` across both. Low priority given USB deprecation. |
| `.ASR` Opus verification | ⬜ | Synthesise an Ogg-Opus wrapper, run `opusinfo`, confirm codec parameters. No audio decoding. Unblocked now that `libopus.so` in the APK confirms the SDK uses Opus encoding. |
| Basename timezone semantics | ⬜ | Second recording on a different calendar day to disambiguate UTC vs device-local. |
| `NOTES/` layout confirmation | ⬜ | Flip the mode slider, record once, verify path scheme is identical to `CALLS/`. |

## Candidates — BLE control channel

| Capability | Status | Next action |
|---|---|---|
| Advertising profile (name, flags, manuf data key) | ✅ | Resolved. |
| GATT baseline (services + characteristics + descriptors) | ✅ | Fully mapped. Vendor service `0x1910`, write char `0x000D`/`0x2BB1`, notify char `0x0010`/`0x2BB0`, vendor notify CCCD at handle `0x0011` confirmed. |
| Vendor frame format (control + bulk) | ✅ | Magic `0x01` control, magic `0x02` bulk. Bulk spec extended with range-addressability and `0xFFFFFFFF` end-of-stream sentinel. |
| `BatteryLevel` (BLE) | ✅ | Standard SIG service. |
| `GetDeviceName` (opcode `0x006C`) | ✅ | Returns ASCII `PLAUD_NOTE`. |
| **Security model** | ✅ | **No BLE bonding, no link-layer encryption.** Access control is application-layer via opcode `0x0001` token only. Traffic is cleartext on the air. Spec'd in [`ble-commands.md`](../../docs/protocol/ble-commands.md#security-model-evidence-backed). |
| **Auth token staticness** | ✅ | Byte-identical across sessions, across full forget+re-pair, across BT restarts. Static per device. |
| Initial pairing flow (Just Works / Passkey / OOB) | ✅ **N/A** | Resolved as "does not exist at BLE layer". Plaud does not do SMP pairing. Application-layer `0x0001` handshake replaces it. |
| `0x0008` full shape | ✅ | `01 08 00 <type> <field> 00 <u32 cursor>`. Type values: `0x01`, `0x02`. Field ids: `0x0F 0x11 0x13 0x14 0x17 0x18 0x1A 0x1B`. Cursor `0xFFFFFFFF` = exhausted. |
| `0x0001` auth token — is derivation necessary? | ✅ **NO, replay is sufficient** | Live tests prove the token is per-device static, replayable verbatim from any BLE central, no nonce, no session key, no MAC binding. Day-to-day CLI operation is fully offline after a one-time bootstrap capture. Evidence: [`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`](captures/ble-live-tests/2026-04-05-token-validation.md). |
| Auth status byte semantics | ✅ | Byte 3 of the `0x01 0x01 0x00 …` response: `0x00` = accepted, `0x01` = rejected. Remaining 13 bytes are a stable device capability tuple (`0a 00 03 00 01 01 00 00 56 5f 00 00` on V0095). |
| Rejection policy | ✅ | Silent soft-reject. Bad auth does NOT cause disconnect; device keeps the BLE link alive, keeps SIG services reachable, silently drops every subsequent vendor opcode. CLI must check `status == 0x00` and abort on any other value with a clear error. |
| Battery level without auth | ✅ | Standard SIG service `0x180F` / char `0x2A19` at handle `0x0015` is readable WITHOUT sending any vendor auth token. `plaude battery` is a free no-auth capability. |
| `0x0001` auth token derivation algorithm (Dart) | 🟡 **deprioritised** | Still blocked on Dart AOT analysis, but no longer on the critical path. Resolving it would enable a hypothetical "fully cold-start offline" first-run UX for users who refuse to install the Plaud app even once; given that owning a Plaud device already requires running the app at least once, this is a nice-to-have, not a feature gate. |
| `0x0067` = `SetPrivacy(bool)` | ✅ | Builder `p257nh/C9563e0(boolean)`. Flutter action `action/setPrivacy`. 0day capture sent `01 67 00 01` during fresh-pair setup (privacy = enabled). |
| `0x0008` full decoding (ActionType + SettingType) | ✅ | Builder `C9568h(ActionType, fieldId, long, long)`. Every wire-captured field id maps to a named device setting in `Constants$CommonSettings$SettingType`: ENABLE_VAD, REC_MODE, VPU_GAIN, MIC_GAIN, AUTO_POWER_OFF, SAVE_RAW_FILE, AUTO_SYNC, FIND_MY. |
| `ReadFileChunk` (opcode `0x001C` + bulk stream) | ✅ | Builder `C9591s0(long file_id, long offset, long length)` — field semantics source-confirmed. File-id↔recording basename mapping is still an open question and needs a capture where we know the recording being transferred. |
| Opcodes `0x04, 0x09, 0x16, 0x18, 0x19, 0x1A, 0x1E, 0x26, 0x6D` | 🟡 | Source-backed constructor signatures now in [`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../specs/re/apk-notes/3.14.0-620/ble-protocol.md). Exact names still need caller-method analysis in `BleAgentImpl.java` (4615-line file) — deferred as a low-priority follow-up since all of them have a known shape. |
| Complete tinnotech SDK opcode inventory | ✅ | 45 total opcodes enumerated from `p257nh/*.java` with constructor signatures. Wi-Fi transfer flow (`0x79`, `0x7A-0x7D`, `0x78`), OTA (`0x72` with raw bytes), file upload from app (`0x32`), RSA pre-handshake (`0xFE12`) all identified but not yet observed on wire. |
| **Plaud uses the tinnotech pen-BLE SDK** | ✅ | Major OEM revelation: the entire BLE wire protocol is `com.tinnotech.penblesdk`, a third-party SDK used by multiple OEM voice-recorder/smart-pen products. Plaud provides only the auth token (from Dart) and the UI on top. See [`specs/re/apk-notes/3.14.0-620/architecture.md`](../../specs/re/apk-notes/3.14.0-620/architecture.md). |
| Two-tier security model | ✅ | Newer firmware (not V0095) uses a full **RSA + ChaCha20-Poly1305** handshake triggered by notification type `0xFE12 (STICK_PREHANDSHAKE_CNF)`. Plaintext check: decrypted value must equal ASCII `"PLAUD.AI"`. Legacy firmware (V0095) uses cleartext + the static 32-hex token. Code: `C10595r4.java:347-435`, crypto helpers in `th/AbstractC11067m.java`. |
| **`0x0026` = 3-int config** | 🟡 | Builder `C9594u(int, int, int)`. Three int parameters. Still unknown if this is the Fast Transfer trigger specifically (the APK has 7 Wi-Fi-related opcodes: `0x78-0x7D`, `0x79`) or a different configuration. Caller analysis in `BleAgentImpl.java` line ~3618 would resolve. |
| **Fast Transfer trigger opcode** | 🟡 | **NEW candidate**: `0x0079` = `C9583o0(int, long, String wifiSSID, String wifiPassword)` — matches the Flutter action `action/connectWiFi` for pushing STA credentials. The **hotspot opening** (device-side) is likely a different nullary opcode in `0x78-0x7D`. Concrete target for the next `re-wifi-probe` session. |
| **Wi-Fi transfer BLE choreography** | 🟡 | Flutter actions `openWiFi`, `connectWiFi`, `closeWiFi`, `syncWiFiFile`, `stopSyncWiFiFile`, `extendWifiExitTime`, `isWiFiConnect`, `isWifiOpen`, `getWiFiFileList`, `deleteWiFiFile` all exist. Each maps to an SDK opcode in range `0x78-0x7D`. Full mapping requires reading the `BleAgentImpl` methods that implement them. |
| **OTA firmware update path** | 🟡 | Opcode `0x72` = `C9580n(int, long, int, byte[])` — raw byte buffer write. Action `action/pushFotaInfo`. We will never implement this in the CLI (firmware modification is out of scope), but documenting it prevents accidentally touching it. |
| RPA rotation period | 🟡 | One RPA (`D1:A6:DE:62:DF:14`) stable for ≥397 s across all 0day sessions. Full rotation period unknown. Low priority. |
| Manufacturer-data payload decode | 🟡 | One candidate: bytes 9–17 of the payload decode as hex-digit ASCII `888317442836358884`, whose first 6 digits match the serial prefix and whose last 12 digits look like a 6-byte factory MAC. Needs a second capture (different day or different device state) to diff against. |
| `StartRecording` / `StopRecording` (BLE remote control) | ⬜ | Not observed in any capture. Nice to have, not critical. |
| Internal Nordic ↔ Realtek bridge | ⬜ | Deferred (hardware teardown required). |
| Bulk file-id ↔ recording basename mapping | ⬜ | Needs a correlated capture where we know which recording is being transferred AND can observe the file_id in bulk frames. Partially unblocked now that we have multiple recordings on the device. |
| What file was transferred in each session | ⬜ | Session A: 34.56 kB; Session C: 75.76 kB. Neither matches a known file exactly. `re-apk-dive` + a fresh controlled capture with known file sizes will resolve. |

## Candidates — Wi-Fi Fast Transfer

| Capability | Status | Next action |
|---|---|---|
| Hotspot discovery (SSID pattern, DHCP, gateway) | ⬜ | `re-wifi-probe` once `StartFastTransfer` over BLE is understood, or triggered manually from the phone app. |
| `ListRecordings` (Wi-Fi) | ⬜ | `re-wifi-probe` + `re-apk-dive`. |
| `DownloadRecording` (Wi-Fi) | ⬜ | `re-wifi-probe`; capture framing and chunking. |

## Candidates — APK static analysis

| Capability | Status | Next action |
|---|---|---|
| Vendor BLE UUID list | ⬜ | `re-apk-dive` on the current Plaud Android APK. |
| BLE command dispatcher | ⬜ | `re-apk-dive`; locate the class that maps opcodes to method calls. |
| Wi-Fi Fast Transfer client class | ⬜ | `re-apk-dive`; grep for URL paths, `OkHttp`, `Retrofit`, `WifiManager`. |

## Notes

- **Device is currently empty** — no recordings. Any capability that touches
  `/NOTES/` or `/CALLS/` content is blocked on a single test recording.
- **Firmware pinned**: all evidence collected so far is against V0095
  (Feb 28 2024). Future firmware captures must record their version in the
  same pattern so cross-firmware diffs stay sane.
- **Serial `888317302431017881`** must be redacted from anything committed.
  Gitignore rules under `specs/re/captures/` cover raw dumps.
