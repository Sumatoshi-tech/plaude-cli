//! Recording control encoders — start, stop, pause, resume.
//!
//! Opcodes decoded from APK static analysis:
//! - `0x0014` StartRecording — `C9569h0(int action, int scene)`
//! - `0x0015` PauseRecording — `C9565f0(long timestamp, int mode)`
//! - `0x0016` ResumeRecording — `C9567g0(long timestamp, int mode)`
//! - `0x0017` StopRecording — `C9571i0(int scene, int action)` (args swapped)
//!
//! Evidence: `specs/re/apk-notes/3.14.0-620/ble-protocol.md`,
//! `p257nh/C9569h0.java`, `p343rh/C10558l3.java:3114-3131`.

use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    encode::control,
    opcode::{OPCODE_PAUSE_RECORDING, OPCODE_RESUME_RECORDING, OPCODE_START_RECORDING, OPCODE_STOP_RECORDING},
};

/// Default action byte for start/stop (from the phone app call chain:
/// `m38434D3(1, scene, ...)` passes `1` as the action).
const ACTION_START: u8 = 1;

/// Default scene byte (0 = default scene).
const SCENE_DEFAULT: u8 = 0;

/// Default mode byte for pause/resume.
const MODE_DEFAULT: u8 = 0;

/// Encode a `StartRecording` (opcode `0x0014`) control frame.
///
/// Wire layout: `01 14 00 <action:u8> <scene:u8>`.
/// The phone app always sends `action=1, scene=0` for a standard start.
#[must_use]
pub fn start_recording() -> Bytes {
    control(OPCODE_START_RECORDING, &[ACTION_START, SCENE_DEFAULT])
}

/// Encode a `StopRecording` (opcode `0x0017`) control frame.
///
/// Wire layout: `01 17 00 <scene:u8> <action:u8>`.
/// Note: args are **swapped** compared to start — the APK builder
/// `C9571i0` takes `(scene, action)` not `(action, scene)`.
#[must_use]
pub fn stop_recording() -> Bytes {
    control(OPCODE_STOP_RECORDING, &[SCENE_DEFAULT, ACTION_START])
}

/// Encode a `PauseRecording` (opcode `0x0015`) control frame.
///
/// Wire layout: `01 15 00 <timestamp:u32 LE> <mode:u8>`.
/// Timestamp is current Unix epoch seconds.
#[must_use]
pub fn pause_recording(epoch_seconds: u32) -> Bytes {
    let mut payload = BytesMut::with_capacity(5);
    payload.put_u32_le(epoch_seconds);
    payload.put_u8(MODE_DEFAULT);
    control(OPCODE_PAUSE_RECORDING, &payload)
}

/// Encode a `ResumeRecording` (opcode `0x0016`) control frame.
///
/// Wire layout: `01 16 00 <timestamp:u32 LE> <mode:u8>`.
#[must_use]
pub fn resume_recording(epoch_seconds: u32) -> Bytes {
    let mut payload = BytesMut::with_capacity(5);
    payload.put_u32_le(epoch_seconds);
    payload.put_u8(MODE_DEFAULT);
    control(OPCODE_RESUME_RECORDING, &payload)
}
