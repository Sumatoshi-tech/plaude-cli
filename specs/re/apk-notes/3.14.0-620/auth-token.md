# Auth token — derivation is irrelevant, replay works

## Status — RESOLVED (by live test, not by static analysis)

**The algorithm question is moot.** Live tests on a V0095 device
([`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`](../../captures/ble-live-tests/2026-04-05-token-validation.md))
proved that the token is a pure **per-device static secret**, replayable
verbatim from any BLE central. There is no session key, no nonce, no MAC
binding. A token captured once is valid for the device's lifetime. The
CLI does not need to derive the token — it needs to capture it **once**
per physical device and store it.

The content below was written before the live tests and is retained for
historical context; the Java → Dart call chain is still accurate as a
description of how the Plaud app itself produces the token, but the
"blocker" framing has been obsoleted. **Replay was always the answer.**

## Call chain — what the Java side reveals

### 1. Dart emits a Flutter MethodChannel call

Dart code in `libapp.so` invokes a method on `plaud.flutter/audioManager`
(the MethodChannel name, visible as a string in the Dart AOT strings dump).
The arguments include a string named `"token"` and a string named `"sn"`
(serial number).

### 2. Java side receives the call

[`ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java:5193`](../../captures/apk/decompiled/3.14.0-620/sources/ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java)

```java
String str2 = (String) methodCall.argument("token");
if (sn == null || str2 == null) {
    result.error("-1", "sn == null || token == null", null);
    return;
}
// …
```

The Java handler does **no computation** on the token. It is passed
straight through.

### 3. Java forwards to the tinnotech SDK

[`FlutterDeviceManager.java:5221`](../../captures/apk/decompiled/3.14.0-620/sources/ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java)

```java
getBleAgent().mo32142S(next, str2, null, null, 30000L, 60000L, bool.booleanValue());
//                          ↑
//                          the token
```

### 4. SDK stores the token for later use in auth frame

[`p343rh/C10558l3.java:3538-3549`](../../captures/apk/decompiled/3.14.0-620/sources/p343rh/C10558l3.java) (`BleAgentImpl.mo32142S`):

```java
public void mo32142S(BleDevice bleDevice, String str, String str2, String str3,
                     long j10, long j11, boolean z10) {
    // …
    this.f30016g = str;      // token
    this.f30017h = z10;
    this.f30018i = str2;     // phoneModel (null in the Plaud case)
    this.f30019j = str3;     // sdkVersion (null in the Plaud case)
    this.f30029t = bleDevice;
    // …
}
```

### 5. SDK serializes the token when building the auth frame

[`p343rh/C10558l3.java:3309`](../../captures/apk/decompiled/3.14.0-620/sources/p343rh/C10558l3.java):

```java
c10595r4.m38584p0(new int[]{1},
    new C9603z(c10595r4.m38555M(), 1, this.f30016g, this.f30029t.m26055f(),
               this.f30018i, this.f30019j).mo35317b(), …);
```

### 6. Builder writes the token into the wire frame

[`p257nh/C9555a0.java:33-71`](../../captures/apk/decompiled/3.14.0-620/sources/p257nh/C9555a0.java):

```java
public byte[] mo35317b() {
    byte[] header = m35321e();              // "01 01 00" (type + opcode LE)
    int length = header.length;
    byte[] bArr = new byte[255];

    System.arraycopy(header, 0, bArr, 0, header.length);
    int offset = TntBleCommUtils.packInt(bArr, length, 2L);     // write constant 2 as u16 LE
    offset = TntBleCommUtils.packInt(bArr, offset, f27268b);    // write version u16 LE
    if (f27271e >= 3) {
        offset = TntBleCommUtils.packInt(bArr, offset, f27269c);  // write flag u16 LE
    }

    // Pad/truncate token to 16 or 32 chars based on protocol version
    int targetLen = (f27271e >= 9) ? 32 : 16;
    // … pad with '0' or truncate …

    ByteBuffer.wrap(bArr, offset, 255 - offset).put(this.f27270d.getBytes());
    return AbstractC11066l.m39996c(bArr);   // trim trailing nulls
}
```

The token `f27270d` is written into the frame **as a UTF-8 byte string**
— it is not hashed, not re-encoded, not transformed. Whatever the Dart
layer computed is what goes on the wire.

**Our V0095 wire capture matches this exactly**: the 32 ASCII-hex bytes
on the wire are a direct serialization of a 32-character Dart string.

## Why we think the token is cloud-issued

- It is **static per phone** across every BLE session we captured.
- It **survives a full phone-side forget + re-pair flow**, so it is not
  cached in Android's bond database.
- It is computed by Dart before the first BLE connection attempt, which
  rules out deriving from anything the device itself advertises beyond
  what is in the scan-response manufacturer data.
- Common derivations (MD5 of serial, MD5 of factory-MAC candidate,
  MD5/HMAC of scan-response bytes) all tested negative.
- The Plaud app has a well-known cloud backend (`api.plaud.ai` and
  `api-euc1.plaud.ai`) with an `/auth/access-token` endpoint visible in
  `libapp.so` strings. A cloud-issued long-lived credential matches all
  the observed invariants perfectly.

This is the **same cloud-issued-token pattern** used by many consumer
IoT apps: the "pairing" that happens in the app is a cloud registration,
and the BLE-side authentication is just replay of a key the cloud gave
you during setup.

## What this means for an offline CLI

The offline CLI cannot run the full Dart code to compute the token, and
cannot obtain a new token from the cloud without compromising the
"offline" guarantee. **Three viable options**, ranked:

### Option 1 (recommended): user-supplied token

The CLI accepts the token as a `--token <hex>` argument or reads it
from a config file / OS keyring on first run. The user extracts it
once from their phone (see Option 3 instructions below) and pastes it
into the CLI.

- **Pros**: zero complexity on our side, honest about the constraint,
  no Dart reverse engineering required, no cloud contact.
- **Cons**: user has to do a one-time extraction.
- **Implementation**: standard `plaude auth set-token <hex>` subcommand;
  token stored via the `keyring` crate.

### Option 2: Dart AOT snapshot analysis

Attempt to locate the token-building Dart function in `libapp.so` using
`reflutter` (a tool that patches the Flutter engine to expose Dart
internals) or a Dart snapshot parser. If the token is locally derived
from the serial + an app-embedded constant, we can re-implement the
algorithm in Rust.

- **Pros**: cleanest end-user experience if it succeeds.
- **Cons**: high effort, may fail if the token is truly cloud-issued,
  and the result is version-specific (a Plaud app update could change
  the algorithm).
- **Next step**: run `reflutter` against `libapp.so` to produce a
  debuggable snapshot, then hook the MethodChannel arguments to log the
  inputs and output of whatever Dart function computes the token.

### Option 3: extract from phone storage

If the token is stored in the app's local data (Isar / MMKV / Android
Keystore), we can document a one-time extraction procedure:

- **Rooted Android**: read the app's data directory directly.
- **Unrooted Android**: `adb shell run-as ai.plaud.android.plaud` if
  the app is debuggable (it is probably not), or full device backup
  with `adb backup` (if allowed by the manifest — modern Plaud
  versions likely disable this).
- **iOS**: jailbroken only; otherwise impossible without the app being
  cooperative.

The CLI could ship a documented `extract-token` helper that runs the
adb commands and parses the Isar/MMKV store.

## Capability flags the device returns

While tracing token handling, a secondary finding surfaced: the device
reports **which pre-shared tokens it currently holds** via opcode
`0x0003` (`GetState`). The response is deserialized into a `GetStateResult`
Kotlin data class at
[`FlutterDeviceManager.java:6353-6402`](../../captures/apk/decompiled/3.14.0-620/sources/ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java) with fields including:

- `hasFindMyToken` — device has an Apple Find My / AirTag-style token
- `hasHttpToken` — device has an HTTP token (probably for uploading to
  Plaud cloud during sync-in-idle)
- `hasSoundPlusToken` — device has a SoundPlus license token (for
  third-party audio enhancement)
- `sessionId`, `state`, `keyState`, `privacy`, `uDisk`, `versionType`,
  `versionCode`

Each of these tokens has a corresponding `action/send*Token` Flutter
action that pushes a token to the device. These are **not the same as
our auth token** — they are additional credentials the device stores
for its own cloud/third-party features.

The existence of `hasHttpToken` is interesting for our project: it
means the device **can upload recordings to a configurable HTTP endpoint
without going through the phone**. If we can push an HTTP token that
points at our own local server, we might be able to set up a
"self-hosted cloud sync" path that completely bypasses the BLE protocol
and avoids both the auth-token problem and the Wi-Fi Fast Transfer
complexity. Filed as a future research item.

## Evidence references

- [`FlutterDeviceManager.java`](../../captures/apk/decompiled/3.14.0-620/sources/ai/plaud/android/plaud/anew/flutter/device/FlutterDeviceManager.java)
  lines 5193, 5221, 6353-6402.
- [`p343rh/C10558l3.java`](../../captures/apk/decompiled/3.14.0-620/sources/p343rh/C10558l3.java)
  lines 3538-3565 (`mo32142S`), 3309 (auth build call), 3545 (token store).
- [`p257nh/C9555a0.java`](../../captures/apk/decompiled/3.14.0-620/sources/p257nh/C9555a0.java)
  lines 24-72 (token serializer).
- [`p257nh/C9603z.java`](../../captures/apk/decompiled/3.14.0-620/sources/p257nh/C9603z.java)
  lines 15-41 (extended serializer appending phone model + SDK version).
