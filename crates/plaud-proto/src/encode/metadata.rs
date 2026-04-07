//! Metadata-sweep encoders — opcodes used during the recording
//! enumeration phase of a sync session.
//!
//! Wire evidence: `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` §5.

use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    encode::control,
    opcode::{OPCODE_1A_TIMESTAMP_WINDOW, OPCODE_1E_FILE_ID, OPCODE_04_TIMESTAMP, OPCODE_09_PERCENT, OPCODE_16_QUERY_BY_FILE_ID},
};

/// Encode opcode `0x0004` — timestamp query.
///
/// Wire layout: `01 04 00 <u32 LE unix_ts> <u16 LE trailer>`.
/// The trailer is always `0x0300` in captures.
#[must_use]
pub fn set_clock(unix_timestamp: u32) -> Bytes {
    let mut payload = BytesMut::with_capacity(6);
    payload.put_u32_le(unix_timestamp);
    payload.put_u16_le(0x0003);
    control(OPCODE_04_TIMESTAMP, &payload)
}

/// Encode opcode `0x0009` — get percent (nullary).
///
/// Response is a single byte (0–100).
#[must_use]
pub fn get_percent() -> Bytes {
    crate::encode::nullary(OPCODE_09_PERCENT)
}

/// Encode opcode `0x0016` — query by file id.
///
/// Wire layout: `01 16 00 <u32 LE file_id> <u8 trailer>`.
/// Response is a 13-byte tuple containing recording metadata.
#[must_use]
pub fn query_by_file_id(file_id: u32) -> Bytes {
    let mut payload = BytesMut::with_capacity(5);
    payload.put_u32_le(file_id);
    payload.put_u8(0x00);
    control(OPCODE_16_QUERY_BY_FILE_ID, &payload)
}

/// Encode opcode `0x001A` — getFileList request.
///
/// Wire layout (from `C9592t.mo35317b()`):
///   `01 1a 00 <epochSeconds:u32 LE> <sessionId:u32 LE> <single:u8>`
///
/// The device responds with one or more control frames (same opcode)
/// carrying file entries in the payload.
#[must_use]
pub fn get_file_list(epoch_seconds: u32, session_id: u32, single: bool) -> Bytes {
    let mut payload = BytesMut::with_capacity(9);
    payload.put_u32_le(epoch_seconds);
    payload.put_u32_le(session_id);
    payload.put_u8(if single { 1 } else { 0 });
    control(OPCODE_1A_TIMESTAMP_WINDOW, &payload)
}

/// Encode opcode `0x001E` — file-id query.
///
/// Wire layout: `01 1e 00 <u32 LE file_id>`.
/// Response is 8 bytes.
#[must_use]
pub fn query_file_id(file_id: u32) -> Bytes {
    let mut payload = BytesMut::with_capacity(4);
    payload.put_u32_le(file_id);
    control(OPCODE_1E_FILE_ID, &payload)
}
