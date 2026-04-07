# BLE protocol — direct APK cross-reference (Plaud Android v3.14.0 / Tinnotech pen-BLE SDK)

Every fact below is backed by a direct `file:line` citation into the jadx
output at `specs/re/captures/apk/decompiled/3.14.0-620/sources/`.

## GATT — UUID constants

All BLE UUIDs are defined in one place:
[`p343rh/InterfaceC10589q4.java`](../../captures/apk/decompiled/3.14.0-620/sources/p343rh/InterfaceC10589q4.java).

| SDK field | UUID | Meaning |
|---|---|---|
| `f30101a` | `00001910-0000-1000-8000-00805f9b34fb` | Vendor service |
| `f30102b` | `00002BB0-0000-1000-8000-00805f9b34fb` | Vendor notify characteristic |
| `f30103c` | `00002BB1-0000-1000-8000-00805f9b34fb` | Vendor write characteristic |
| `f30104d` | `0000180f-0000-1000-8000-00805f9b34fb` | SIG Battery service |
| `f30105e` | `00002a19-0000-1000-8000-00805f9b34fb` | SIG Battery Level char |
| `f30106f` | `00002902-0000-1000-8000-00805f9b34fb` | SIG CCCD descriptor |

These are **byte-for-byte identical** to what we captured in the
btsnoop walkthroughs. The vendor "service + notify + write" UUIDs
using SIG-reserved 16-bit values (`0x1910`, `0x2BB0`, `0x2BB1`) is a
**deliberate tinnotech SDK design choice**, not a Plaud modification.
`0x2BB0`/`0x2BB1` happen to be SIG-registered for BLE 5.1 Constant
Tone Extension direction-finding — the tinnotech SDK squats on these
UUIDs for its vendor protocol.

## Frame format — source-confirmed

### Control frame (magic byte `0x01`)

Defined by the abstract builder base class `AbstractC9560d` and every
concrete opcode builder in `p257nh/`. Every builder's
`mo35318c()` returns the **frame type byte**:

- `return 1;` → control frame (all opcode builders seen)

Every builder's `mo35319d()` returns the **16-bit opcode** as an `int`.
The base class's `m35321e()` emits the header
`<type:u8> <opcode:u16 LE>` (3 bytes), and `mo35317b()` appends
opcode-specific fields.

### Bulk frame (magic byte `0x02`)

Not a builder — generated directly by the device in response to
opcode `0x001C` reads. The receiver path is at
[`p343rh/C10595r4.java:236-278`](../../captures/apk/decompiled/3.14.0-620/sources/p343rh/C10595r4.java)
(`m38606l`), where the first byte is read and dispatched on:

- `1` → control frame parser (opcode-based callbacks)
- **`2` → file sync bulk frame** (passed to `m38552J(29)` callback)
- `4` → BLE rate test frame (passed to `m38552J(203)` callback)
- `5` → log package frame (passed to `m38552J(143)` callback)

**This confirms our wire-capture interpretation of the first byte as a
frame-type selector**, and adds two frame types we never exercised
(`4` = rate test, `5` = log package).

### TntBleCommUtils packing helpers

`com/tinnotech/penblesdk/utils/TntBleCommUtils.java` is a thin JNI
wrapper around `libtnt_ble_utils.so`. Method symbols with native
equivalents:

| Java | JNI | Role |
|---|---|---|
| `m26100c(byte[], int offset, int/long value)` | `Java_..._packInt` | Write little-endian integer into buffer, return new offset |
| `m26107j(int)` | — | Convert int to single byte |
| `m26108k(byte[], int offset)` | — | Read byte at offset |
| `m26109l(byte[], int offset)` | `Java_..._readInt` | Read little-endian u16/u32 from buffer |
| `tntGetCrc(...)` | `Java_..._tntGetCrc` | CRC of a byte buffer |
| `tntGetFileCrc(...)` | `Java_..._tntGetFileCrc` | CRC of a file (post-reassembly integrity) |

`tntGetFileCrc` taking a FILE pointer (not a byte[]) confirms
**file-level integrity verification happens after the bulk stream is
fully reassembled to disk**, not per-frame.

## Opcode dictionary — complete SDK inventory

Source: the 50 builder classes in
`specs/re/captures/apk/decompiled/3.14.0-620/sources/p257nh/`.
Each class corresponds to one opcode (some opcodes share a builder
via inheritance). Reading each builder's `mo35319d()` and constructor
signature gives the full wire format.

### Opcodes observed on the wire (V0095 captures)

Every opcode from both btsnoop captures is confirmed in the source.

| Opcode (hex) | Dec | Builder class | Constructor | APK evidence | Plaud action |
|---|---|---|---|---|---|
| `0x0001` | 1 | `C9555a0` / `C9603z` | `(version, flag, String token, protocolVersion[, phoneModel, sdkVersion])` | `p257nh/C9555a0.java:24`, `C9603z.java:15` | **Authenticate** — called from `FlutterDeviceManager.configMethodChannel` → `getBleAgent().mo32142S(device, token, ...)` at `ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java:5221`. Token comes from `methodCall.argument("token")` at line 5193. |
| `0x0003` | 3 | `C9596v` | (nullary) | `p257nh/C9596v.java` | get device state |
| `0x0004` | 4 | `C9601x0` | `(int, int)` | `p257nh/C9601x0.java` | two-int query, carries Unix timestamp in low int |
| `0x0006` | 6 | `C9598w` | (nullary) | `p257nh/C9598w.java` | storage/counter stats (response is 27-byte tuple) |
| `0x0008` | 8 | `C9568h` | `(Constants$CommonSettings$ActionType action, int fieldId, long j10, long j11)` | `p257nh/C9568h.java` | **GetCommonSettings / SetCommonSettings** — iterates the `SettingType` enum |
| `0x0009` | 9 | `C9562e` | (nullary) | `p257nh/C9562e.java` | 1-byte percentage response |
| `0x0016` | 22 | `C9565f0` | `(long, int)` | `p257nh/C9565f0.java` | timestamp-based query |
| `0x0018` | 24 | `C9590s` | (nullary) | `p257nh/C9590s.java` | |
| `0x0019` | 25 | `C9579m0` | `(boolean)` | `p257nh/C9579m0.java` | 1-bool setter |
| `0x001A` | 26 | `C9592t` | `(long, long, boolean)` | `p257nh/C9592t.java` | timestamp + timestamp + bool |
| `0x001C` | 28 | `C9591s0` | **`(long file_id, long offset, long length)`** | `p257nh/C9591s0.java:18` | **ReadFileChunk** — triggers bulk stream on magic `0x02`. Called from `BleAgentImpl` at `p343rh/C10558l3.java:3208` with `new int[]{28, 29}` (command + bulk response opcode). |
| `0x001E` | 30 | `C9589r0` | `(long)` | `p257nh/C9589r0.java` | single-long query |
| `0x0026` | 38 | `C9594u` | `(int, int, int)` | `p257nh/C9594u.java` | 3-int config. Called from `BleAgentImpl.java:3618`. |
| `0x0067` | 103 | `C9563e0` | `(boolean privacy)` | `p257nh/C9563e0.java:12` | **`SetPrivacy`** — `FlutterDeviceManager.java:3475 "action/setPrivacy"` → `BleAgentImpl.mo32186s(boolean, …)` at `C10558l3.java:4291`. 0day capture saw `01 67 00 01` during fresh-pair setup. |
| `0x006C` | 108 | `C9584p` | (nullary) | `p257nh/C9584p.java` | **GetDeviceName** — nullary, response is ASCII padded name. Called from `BleAgentImpl.java:4154`. |
| `0x006D` | 109 | `C9554a` | `(boolean)` | `p257nh/C9554a.java` | 1-bool. Called from `BleAgentImpl.java:3224`. |

### Opcodes in the SDK but never exercised in V0095 captures

These are exposed by the SDK but not triggered by our particular
phone/device/firmware combination. Listed with their builder signatures
for future reference.

| Opcode | Dec | Builder | Constructor | Likely name |
|---|---|---|---|---|
| `0x000A` | 10 | `C9557b0` | `(int)` | |
| `0x000D` | 13 | `C9566g` | (nullary) | |
| `0x0014` | 20 | `C9569h0` | `(int, int)` | |
| `0x0015` | 21 | `C9565f0` | `(long, int)` | |
| `0x0017` | 23 | `C9571i0` | `(int, int)` | |
| `0x001D` | 29 | `C9593t0` | (nullary) | (file-sync bulk callback opcode — paired with `0x001C`) |
| `0x0032` | 50 | `C9558c` | `(long, String, String, int, long, int)` | file-transfer-related (6-arg signature suggests upload from app to device) |
| `0x0033` | 51 | `C9556b` | `(long, int)` | |
| `0x003D` | 61 | `C9595u0` | `(int)` | |
| `0x0068` | 104 | `C9564f` | (nullary) | |
| `0x006B` | 107 | `C9577l0` | `(String name)` | **`SetDeviceName(String)`** |
| `0x0070` | 112 | `C9582o` | `(int, long)` | |
| `0x0072` | 114 | `C9580n` | `(int, long, int, byte[])` | **OTA chunk write** (raw byte buffer) |
| `0x0074` | 116 | `C9578m` | `(int, short)` | |
| `0x0078` | 120 | `C9600x` | `(long)` | |
| `0x0079` | 121 | `C9583o0` | **`(int, long, String wifiSSID, String wifiPassword)`** | **`ConnectWiFi` / `SetWifiConfig`** — push STA credentials to device for cloud sync |
| `0x007A` | 122 | `C9602y` | (nullary) | Wi-Fi adjacent |
| `0x007B` | 123 | `C9597v0` | `(long)` | Wi-Fi adjacent |
| `0x007C` | 124 | `C9599w0` | `(long)` | Wi-Fi adjacent |
| `0x007D` | 125 | `C9572j` | `(List wifiIndex)` | List configured Wi-Fi networks by index |
| `0x0080` | 128 | `C9573j0` | (nullary) | |
| `0x0082` | 130 | `C9581n0` | `(String key)` | String-keyed setter |
| `0x0083` | 131 | `C9575k0` | (nullary) | |
| `0x008A` | 138 | `C9576l` | (nullary) | |
| `0x008D` | 141 | `C9588r` | (nullary) | |
| `0x008E` | 142 | `C9586q` | `(int)` | |
| `0x008F` | 143 | `C9585p0` | `(int, int)` | paired with log package frames (type `5`) |
| `0x0091` | 145 | `C9587q0` | (nullary) | |
| `0x0092` | 146 | `C9570i` | `(int, int)` | |
| `0xFE12` | 65042 | `C9561d0` | `(int, int, byte[])` | **RSA pre-handshake packet** (newer firmware only) |

**Total**: 45 distinct opcodes in the SDK. Our V0095 captures exercised
16 of them. The rest cover Wi-Fi Fast Transfer setup, OTA firmware
update, file upload (app → device), the RSA handshake, log transfer,
and various BLE settings not touched during a routine sync.

## `0x0008` — full decode

The most frequently observed opcode now has full source-backed semantics.

**Builder** [`p257nh/C9568h.java`](../../captures/apk/decompiled/3.14.0-620/sources/p257nh/C9568h.java):

```java
public C9568h(Constants$CommonSettings$ActionType actionType, int fieldId, long j10, long j11)
```

**Wire layout**:

```
01 08 00 <action u8> <field u8> 00 <long u64 LE> <long u64 LE>
```

Except the wire we captured shows a shorter form (6 bytes + 4-byte u32 trailer
like `01 08 00 01 0f 00 00 00 00 00`) — so `j10` and `j11` may be optional
or conditional on a flag. Exact trailer layout: 4 bytes zero in every observed
call, 4 more bytes that were `0xFFFFFFFF` in one 0day frame (likely a
cursor/pagination value).

**Action type enum** ([`Constants$CommonSettings$ActionType.java`](../../captures/apk/decompiled/3.14.0-620/sources/com/tinnotech/penblesdk/Constants$CommonSettings$ActionType.java)):

| Value | Name |
|---|---|
| `1` | `READ` |
| `2` | `SETTING` (write) |

**Setting type enum** ([`Constants$CommonSettings$SettingType.java`](../../captures/apk/decompiled/3.14.0-620/sources/com/tinnotech/penblesdk/Constants$CommonSettings$SettingType.java)):

| Code | Name | Notes |
|---|---|---|
| `1` | `BACK_LIGHT_TIME` | |
| `2` | `BACK_LIGHT_BRIGHTNESS` | |
| `3` | `LANGUAGE` | |
| `4` | `AUTO_DELETE_RECORD_FILE` | |
| **`15`** | **`ENABLE_VAD`** | seen on wire as field `0x0F` |
| `16` | `REC_SCENE` | |
| **`17`** | **`REC_MODE`** | seen on wire as field `0x11` |
| `18` | `VAD_SENSITIVITY` | |
| **`19`** | **`VPU_GAIN`** | seen on wire as field `0x13` |
| **`20`** | **`MIC_GAIN`** | seen on wire as field `0x14` |
| `21` | `WIFI_CHANNEL` | |
| `22` | `SWITCH_HANDLER_ID` | |
| **`23`** | **`AUTO_POWER_OFF`** | seen on wire as field `0x17` |
| **`24`** | **`SAVE_RAW_FILE`** | seen on wire as field `0x18` |
| `25` | `AUTO_RECORD` | |
| **`26`** | **`AUTO_SYNC`** | seen on wire as field `0x1A` |
| **`27`** | **`FIND_MY`** | seen on wire as field `0x1B` (0day only) |
| `30` | `VPU_CLK` | |
| `31` | `AUTO_STOP_RECORD` | |
| `32` | `BATTERY_MODE` | |

**Every wire-captured `0x0008` field id now has a name**. The phone
iterates through 8 settings on every sync: ENABLE_VAD, REC_MODE,
VPU_GAIN, MIC_GAIN, AUTO_POWER_OFF, SAVE_RAW_FILE, AUTO_SYNC, and
(on fresh pair) FIND_MY.

## Multi-frame type routing

`onCharacteristicChanged` in `C10595r4.java` routes incoming
notifications by the first byte:

| First byte | Name | Handler |
|---|---|---|
| `1` | Control frame | opcode dispatch table via `m38552J(opcode)` lookup |
| `2` | Bulk file sync | `m38552J(29)` callback |
| `4` | BLE rate test | `m38552J(203)` callback |
| `5` | Log package | `m38552J(143)` callback |
| `0xFE12` (as u16 at offset 0) | RSA pre-handshake CNF | new-firmware RSA+ChaCha20 path |
| `0xFE11` (as u16 at offset 0) | (unknown, also pre-auth) | file-sync-related |

`0x0001` and `0x0002` are the **type byte** (`1` = control, `2` = bulk).
The `0xFE11`/`0xFE12` values are read as a **16-bit u16 at offset 0**,
which only matches because those high bytes (`0xFE`) cannot collide with
type byte values `1`–`5`. This is an unusual multiplexing scheme: the
same first byte doubles as (a) an 8-bit frame type and (b) part of a
16-bit "handshake type" marker. Parsers must handle this by reading
`bArr[0]` first, and if it is `0xFE`, re-reading the two bytes as a
u16 LE to distinguish the RSA handshake packets.
