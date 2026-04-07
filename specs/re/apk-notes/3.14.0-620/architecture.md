# Plaud Android app architecture — v3.14.0 (vc 620)

## APK structure

- **Package**: `ai.plaud.android.plaud`
- **Version**: 3.14.0, versionCode 620
- **Min SDK**: 24 (Android 7.0), **Target SDK**: 35
- **Distribution**: split APK (`base.apk` + `split_config.arm64_v8a.apk` +
  language + density splits)
- **Java class count** (from jadx): 10 714

## Runtime stack

This is a **Flutter app** with a thin Java layer. The Java code is almost
entirely a bridge between Flutter's `MethodChannel` and third-party native
libraries.

### Native libraries (arm64 split, 102 MB total)

Key libs and their roles:

| Lib | Size | Role |
|---|---|---|
| `libapp.so` | 28 MB | **The entire Dart code** compiled to AOT native. This is where the Plaud-specific business logic (UI, state, auth token computation, cloud sync) lives. |
| `libflutter.so` | 11 MB | Flutter engine. |
| `libonnxruntime.so` | 17 MB | ONNX Runtime for on-device ML inference (VAD, wake-word, or transcription preview — model location TBD). |
| `libavcodec.so` + friends | ~30 MB | Full FFmpeg kit for media processing. |
| `liblame.so` | 477 kB | LAME MP3 encoder — used to transcode the device's raw WAV to MP3 for UI playback/export. |
| `libopus.so` + `libopusJni.so` + `libjni_ogg.so` | ~445 kB | **Opus encoder/decoder** — directly confirms the `.ASR` sidecar files are Opus streams. |
| `libvad_jni.so` | 65 kB | Voice activity detection (probably used to drive the `ENABLE_VAD` device setting). |
| `libSoundPlus.so` | 218 kB | Audio enhancement (noise suppression / beamforming). |
| `libisar.so` | 1.1 MB | Isar NoSQL database used for local app storage. |
| `libmmkv.so` | 718 kB | Tencent MMKV key-value store. |
| **`libtnt_ble_utils.so`** | **6.6 kB** | **Tinnotech BLE helpers** — the JNI bridge for the tinnotech BLE SDK (see [`ble-protocol.md`](ble-protocol.md)). |
| `libdatadog-native-lib.so` + `libcrashlytics*` | ~2 MB | Telemetry. |
| `libconscrypt_jni.so` | 2.1 MB | Modern TLS/crypto provider. |
| `librive_text.so` | 3.5 MB | Rive animation text rendering. |
| `libsoundtouch.so` | 956 kB | Audio time/pitch manipulation. |

## The OEM revelation

Plaud is **not** running their own custom BLE stack. The BLE code is the
**Tinnotech pen-BLE SDK** (`com.tinnotech.penblesdk`), a third-party
library used by multiple OEM voice-recorder/smart-pen products. Evidence:

1. `libtnt_ble_utils.so` exposes only JNI functions under
   `Java_com_tinnotech_penblesdk_utils_TntBleCommUtils_*`, including
   `packInt`, `readInt`, `readFloat`, `tntGetCrc`, `tntGetFileCrc`.
2. The Java tree `com/tinnotech/penblesdk/` contains the public API
   (enums, data classes, CRC wrapper, Opus wrapper).
3. The tinnotech runtime implementation lives in jadx-obfuscated
   packages `p343rh/` (BLE manager + agent) and `p257nh/` (opcode
   builder classes) — this is the SDK's own ProGuard'd output,
   separate from the public data types.
4. Thread pool prefixes use the name `"tnt-connectionBLE-pool-%d"`.
5. The CRC native symbols are `tnt_get_crc` and `tnt_get_file_crc`.

The Plaud-specific code sits ABOVE the tinnotech SDK in
`ai.plaud.android.plaud.anew.flutter.device.FlutterDeviceManager` (a
6 713-line Flutter MethodChannel handler) and in Dart code inside
`libapp.so`. Plaud provides the auth token (see [`auth-token.md`](auth-token.md)),
the UI, the cloud integration, and the file format conversion
(WAV→MP3 for export). The wire protocol itself is entirely tinnotech's.

**Implication for offline CLI**: the BLE wire protocol we are targeting
is a **generic tinnotech pen-BLE protocol**. Other OEMs shipping
tinnotech-based products should speak the same format. Documentation
or sample code for the SDK may exist publicly if a less
privacy-conscious OEM has published their integration.

## Dart ↔ Java bridge

Plaud's Dart code talks to Java via a Flutter MethodChannel named
`plaud.flutter/audioManager` (seen as a string in `libapp.so`). The
Java side is `FlutterDeviceManager.configMethodChannel(...)`, which
receives method calls with string IDs of the form `action/<name>` and
dispatches to either Plaud-specific logic or the tinnotech SDK via
`kh.InterfaceC8637a`.

### Full Flutter action surface (81 actions)

Observed in `FlutterDeviceManager.java` as literal string constants.
This is the complete device-control API exposed to the Plaud Dart code.

**Connection lifecycle**
`startBleScan`, `stopBleScan`, `connectDevice`, `disconnectDevice`,
`depairDevice`, `isBleConnect`, `isDeviceConnect`, `getBluetoothPermission`,
`getBluetoothPowerOn`, `getCurrentDevice`

**File operations (BLE path)**
`getFileList`, `syncFile`, `stopSyncFile`, `deleteFile`, `clearAllFile`,
`sendSyncFileInd`, `getRecordTags`

**File operations (Wi-Fi Fast Transfer path)**
`openWiFi`, `closeWiFi`, `connectWiFi`, `disconnectWiFi`, `isWiFiConnect`,
`isWifiOpen`, `extendWifiExitTime`, `syncWiFiFile`, `stopSyncWiFiFile`,
`deleteWiFiFile`, `getWiFiFileList`

**Recording control**
`startRecord`, `pauseRecord`, `resumeRecord`, `stopRecord`

**Device info / state**
`getDeviceName`, `setDeviceName`, `getDeviceScene`, `getDeviceStatus`,
`getBatteryState`, `getState`, `getStorage`, `getSDFlashCID`

**Settings** (all use opcode `0x0008` with the `SettingType` enum)
`getMicGain`/`setMicGain`, `getAutoPowerOff`/`setAutoPowerOff`,
`getRawWaveEnabled`/`setRawWaveEnabled`, `getVPUCLKState`/`setVPUCLKState`,
`getVPUSensitivity`/`setVPUSensitivity`, `getAutoRecordEnabled`/`setAutoRecordEnabled`,
`getAutoStopRecordEnabled`/`setAutoStopRecordEnabled`, `getBatteryMode`/`setBatteryMode`,
`getLedState`/`setLedState`, `setPrivacy` (opcode `0x0067`)

**Sync-in-idle (cloud sync while charging)**
`getSyncInIdleState`/`setSyncInIdleState`, `getSyncInIdleWifiConfig`/`setSyncInIdleWifiConfig`,
`deleteSyncInIdleWifiConfig`, `getSyncInIdleWifiList`, `setSyncInIdleWifiTest`,
`getSyncInIdleWifiTestResult`

**Find My**
`getFindMyState`, `setFindMyState`, `sendFindMyToken`, `resetFindmy`

**Third-party tokens**
`sendHttpToken`, `setSoundPlusToken`

**OTA / diagnostics**
`pushFotaInfo`, `getDeviceLogFileList`, `startSyncDeviceLogFile`,
`stopSyncDeviceLogFile`, `deleteDeviceLogFile`

**Misc**
`sendGlobalData`

## Two-tier security model (revealed by APK)

The tinnotech SDK supports **two independent security modes** and the
app picks based on device firmware:

### Mode A — Legacy plaintext (our captured V0095 firmware)

- All BLE traffic is **cleartext**.
- Authentication is a **pre-shared token** sent in opcode `0x0001`.
- Token is passed into the SDK as a `String` from the Plaud Flutter layer
  via `InterfaceC8637a.mo32142S(device, **token**, phoneModel, sdkVersion, …)`.
- Token derivation is **entirely in Dart** (see [`auth-token.md`](auth-token.md)).

### Mode B — RSA + ChaCha20-Poly1305 (newer firmware)

Triggered when the device sends a preamble notification with type
`0xFE12` (`STICK_PREHANDSHAKE_CNF`). Sequence:

1. Device sends multi-packet encrypted "secretPackages" with header
   `[type:u16 LE] [count:u8] [index:u8] <payload>`.
2. App collects packets, sorts by the index byte, strips 4-byte headers,
   concatenates payloads.
3. App **RSA-decrypts** the concatenation with a per-user private key
   stored as `"userRSAPrivateKey"` in the app's key-value store.
4. First 32 bytes of plaintext = **ChaCha20 key**.
5. Bytes 32–44 = **ChaCha20 nonce** (12 bytes).
6. Bytes 44–56 = **ChaCha20 associated data** (12 bytes).
7. Bytes 56+ = **ChaCha20-Poly1305 ciphertext**.
8. App ChaCha20-Poly1305-decrypts the ciphertext and verifies the
   plaintext equals ASCII `"PLAUD.AI"`.
9. If valid, device is authenticated; subsequent BLE frames are
   ChaCha20-decrypted before being passed to the normal frame parser.

**Code location**: `p343rh/C10595r4.java` `onCharacteristicChanged` handler,
lines 347–435. Crypto helpers at `th/AbstractC11067m.java`
(`m40001d` = RSA decrypt, `m39999b` = ChaCha20-Poly1305 decrypt).

**Our V0095 firmware does not trigger Mode B** — all captured traffic
was Mode A. This is why our wire captures show cleartext frames.

## File layout in jadx output

```
specs/re/captures/apk/decompiled/3.14.0-620/sources/
├── ai/plaud/android/plaud/anew/flutter/device/
│   └── FlutterDeviceManager.java        ← 6713 lines, Flutter ↔ SDK bridge
├── com/tinnotech/penblesdk/              ← public SDK data types (17 files)
│   ├── Constants$CommonSettings$ActionType.java   (enum: READ, SETTING)
│   ├── Constants$CommonSettings$SettingType.java  (enum: 20 settings)
│   ├── Constants$ConnectBleFailed.java
│   ├── Constants$ConnectWifiFailed.java
│   ├── Constants$OtaUpgradeStatus.java
│   ├── Constants$ScanFailed.java
│   ├── entity/
│   │   ├── BleDevice.java
│   │   ├── BleFile.java
│   │   ├── BluetoothStatus.java
│   │   ├── WifiStatus.java
│   │   └── bean/BleRequestBean.java    ← wire frame carrier
│   └── utils/
│       ├── OpusUtils.java              ← Opus codec JNI wrapper
│       └── TntBleCommUtils.java        ← int packing + CRC JNI wrapper
├── kh/InterfaceC8637a.java              ← 266-line SDK public interface
├── p343rh/                              ← obfuscated SDK runtime
│   ├── C10558l3.java                   ← 4615 lines, BleAgentImpl (command dispatcher)
│   ├── C10595r4.java                   ← 1570 lines, BluetoothLeOperation (GATT manager)
│   ├── C10558l3.java                   ← owns BLE manager + implements SDK interface
│   ├── InterfaceC10589q4.java          ← UUID constants
│   ├── InterfaceC10571n4.java          ← listener interfaces
│   ├── InterfaceC10577o4.java
│   └── InterfaceC10583p4.java
├── p257nh/                              ← 50 opcode-builder classes
│   ├── AbstractC9560d.java             ← base class with header emitter
│   └── C955xx...java                    ← one per opcode
├── th/                                  ← crypto + utilities
│   ├── AbstractC11066l.java            ← byte helpers
│   ├── AbstractC11067m.java            ← RSA + ChaCha20-Poly1305
│   └── AbstractC11072r.java            ← logger
└── sh/C10821t.java                      ← 909-line WebSocket client (unrelated to BLE)
```
