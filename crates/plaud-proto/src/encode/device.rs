//! Device-level control-frame encoders: name queries, privacy flag,
//! session lifecycle.

use bytes::Bytes;

use crate::{
    constants::{CLOSE_SESSION_ARG, PRIVACY_OFF, PRIVACY_ON},
    encode::{control, nullary},
    opcode::{OPCODE_CLOSE_SESSION, OPCODE_GET_DEVICE_NAME, OPCODE_GET_STATE, OPCODE_GET_STORAGE_STATS, OPCODE_SET_PRIVACY},
};

/// Encode a `GetDeviceName` (opcode `0x006C`) control frame. Nullary.
/// The response is ASCII `PLAUD_NOTE` padded with `0x00` bytes.
#[must_use]
pub fn get_device_name() -> Bytes {
    nullary(OPCODE_GET_DEVICE_NAME)
}

/// Encode a `GetState` (opcode `0x0003`) control frame. Nullary.
#[must_use]
pub fn get_state() -> Bytes {
    nullary(OPCODE_GET_STATE)
}

/// Encode a `GetStorageStats` (opcode `0x0006`) control frame. Nullary.
#[must_use]
pub fn get_storage_stats() -> Bytes {
    nullary(OPCODE_GET_STORAGE_STATS)
}

/// Encode a `SetPrivacy` (opcode `0x0067`) control frame with a
/// single boolean payload. Corresponds to the Flutter action
/// `action/setPrivacy` and was observed in the 0day re-pair capture
/// with the wire value `0x01` during first-time setup.
#[must_use]
pub fn set_privacy(on: bool) -> Bytes {
    let flag = if on { PRIVACY_ON } else { PRIVACY_OFF };
    control(OPCODE_SET_PRIVACY, &[flag])
}

/// Encode a `CloseSession` (opcode `0x006D`) control frame.
/// Observed as the last control write of every captured sync.
#[must_use]
pub fn close_session() -> Bytes {
    control(OPCODE_CLOSE_SESSION, &[CLOSE_SESSION_ARG])
}
