//! `CommonSettings` opcode encoders — read and write device settings
//! via opcode `0x0008`.
//!
//! Wire format (from `specs/re/apk-notes/3.14.0-620/ble-protocol.md`):
//! `<ActionType u8> <SettingType u8> 00 <value u64 LE> <value2 u64 LE>`
//!
//! ActionType: `1 = READ`, `2 = SETTING (write)`.

use bytes::{BufMut, Bytes, BytesMut};
use plaud_domain::CommonSettingKey;

use crate::{encode::control, opcode::OPCODE_COMMON_SETTINGS};

/// ActionType byte for a read request.
const ACTION_READ: u8 = 1;

/// ActionType byte for a write request.
const ACTION_WRITE: u8 = 2;

/// Payload length: action(1) + setting_code(1) + reserved(1) + value(8) + value2(8) = 19.
const SETTINGS_PAYLOAD_LEN: usize = 19;

/// Encode a `CommonSettings READ` control frame that queries one
/// setting by key.
#[must_use]
pub fn read_setting(key: CommonSettingKey) -> Bytes {
    let mut payload = BytesMut::with_capacity(SETTINGS_PAYLOAD_LEN);
    payload.put_u8(ACTION_READ);
    payload.put_u8(key.code());
    payload.put_u8(0); // reserved
    payload.put_u64_le(0); // value (unused for reads)
    payload.put_u64_le(0); // value2 (unused for reads)
    control(OPCODE_COMMON_SETTINGS, &payload)
}

/// Encode a `CommonSettings SETTING` control frame that writes one
/// setting value.
#[must_use]
pub fn write_setting(key: CommonSettingKey, value: u64) -> Bytes {
    let mut payload = BytesMut::with_capacity(SETTINGS_PAYLOAD_LEN);
    payload.put_u8(ACTION_WRITE);
    payload.put_u8(key.code());
    payload.put_u8(0); // reserved
    payload.put_u64_le(value);
    payload.put_u64_le(0); // value2
    control(OPCODE_COMMON_SETTINGS, &payload)
}
