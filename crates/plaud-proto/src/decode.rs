//! Notification decoding.
//!
//! The single entry point is [`parse_notification`]. It takes a
//! zero-copy [`Bytes`] view of the raw notification payload and
//! returns a typed [`Frame`] or a [`DecodeError`] that explains why
//! the input could not be understood. There are **no panics** on any
//! input, regardless of length or content.
//!
//! The function allocates at most one `Bytes` slice (via the
//! zero-copy [`Bytes::slice`]) per call — the hot path is fast
//! enough to sit under a BLE notification handler without
//! introducing per-frame overhead.

use bytes::Bytes;
use plaud_domain::AuthToken;

use crate::{
    constants::{
        AUTH_PREFIX, AUTH_RESPONSE_MIN_PAYLOAD_LEN, AUTH_STATUS_ACCEPTED, AUTH_STATUS_REJECTED, BULK_END_OFFSET_SENTINEL,
        BULK_FILE_ID_OFFSET, BULK_HEADER_LEN, BULK_OFFSET_OFFSET, CONTROL_HEADER_LEN, FRAME_TYPE_BULK, FRAME_TYPE_CONTROL,
        HANDSHAKE_SIGNATURE_HIGH_BYTE, HANDSHAKE_TYPE_LEN,
    },
    error::DecodeError,
    frame::{AuthStatus, Frame},
    opcode::OPCODE_AUTHENTICATE,
};

/// Parse a single BLE notification into a typed [`Frame`].
///
/// # Errors
///
/// Returns [`DecodeError`] if the input is empty, is shorter than
/// the minimum header of its frame family, or starts with a byte
/// that does not identify any known frame type.
pub fn parse_notification(data: Bytes) -> Result<Frame, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::Empty);
    }
    // The handshake preamble (`0xFE11` / `0xFE12`) is read as a `u16`
    // at offset 0 with high byte `0xFE`. Because the control and
    // bulk frame-type bytes are all in the `[0x01, 0x05]` range,
    // looking at byte 1 for `0xFE` is an unambiguous way to detect
    // a handshake before the single-byte demux runs.
    if data.len() >= HANDSHAKE_TYPE_LEN && data[1] == HANDSHAKE_SIGNATURE_HIGH_BYTE {
        return parse_handshake(data);
    }
    let first = data[0];
    match first {
        FRAME_TYPE_CONTROL => parse_control(data),
        FRAME_TYPE_BULK => parse_bulk(data),
        other => Err(DecodeError::UnknownFrameType { byte: other }),
    }
}

/// Parse the result byte of an auth response control frame.
///
/// # Errors
///
/// Returns [`DecodeError::NotAuthResponse`] if the frame is not a
/// control frame with opcode `0x0001`, [`DecodeError::TooShort`] if
/// the payload is empty, or [`DecodeError::UnknownAuthStatus`] if
/// the status byte is outside `{0x00, 0x01}`.
pub fn auth_response(frame: &Frame) -> Result<AuthStatus, DecodeError> {
    let Frame::Control { opcode, payload } = frame else {
        return Err(DecodeError::NotAuthResponse);
    };
    if *opcode != OPCODE_AUTHENTICATE {
        return Err(DecodeError::NotAuthResponse);
    }
    if payload.len() < AUTH_RESPONSE_MIN_PAYLOAD_LEN {
        return Err(DecodeError::TooShort {
            expected: AUTH_RESPONSE_MIN_PAYLOAD_LEN,
            got: payload.len(),
        });
    }
    match payload[0] {
        AUTH_STATUS_ACCEPTED => Ok(AuthStatus::Accepted),
        AUTH_STATUS_REJECTED => Ok(AuthStatus::Rejected),
        other => Err(DecodeError::UnknownAuthStatus { byte: other }),
    }
}

/// Parse the bytes of an auth *write* (phone → device) frame and
/// extract the [`AuthToken`] the phone sent.
///
/// This is the inverse of [`crate::encode::auth::authenticate`] and
/// is used by the M8 `plaude-cli auth bootstrap` flow: a local fake
/// peripheral captures the write, feeds the raw bytes into this
/// function, and stores the returned token.
///
/// # Errors
///
/// Returns [`DecodeError::TooShort`] if the input is shorter than
/// [`AUTH_PREFIX`] + the minimum token length, [`DecodeError::InvalidAuthPrefix`]
/// if the leading bytes do not match [`AUTH_PREFIX`], or
/// [`DecodeError::InvalidAuthToken`] if the trailing bytes fail
/// [`AuthToken`] validation.
pub fn parse_auth_write(data: &[u8]) -> Result<AuthToken, DecodeError> {
    if data.len() < AUTH_PREFIX.len() {
        return Err(DecodeError::TooShort {
            expected: AUTH_PREFIX.len(),
            got: data.len(),
        });
    }
    if &data[..AUTH_PREFIX.len()] != AUTH_PREFIX {
        return Err(DecodeError::InvalidAuthPrefix);
    }
    let token_bytes = &data[AUTH_PREFIX.len()..];
    let token_str = std::str::from_utf8(token_bytes).map_err(|e| DecodeError::InvalidAuthToken { reason: e.to_string() })?;
    AuthToken::new(token_str).map_err(|e| DecodeError::InvalidAuthToken { reason: e.to_string() })
}

fn parse_control(data: Bytes) -> Result<Frame, DecodeError> {
    if data.len() < CONTROL_HEADER_LEN {
        return Err(DecodeError::TooShort {
            expected: CONTROL_HEADER_LEN,
            got: data.len(),
        });
    }
    let opcode = u16::from_le_bytes([data[1], data[2]]);
    let payload = data.slice(CONTROL_HEADER_LEN..);
    Ok(Frame::Control { opcode, payload })
}

fn parse_bulk(data: Bytes) -> Result<Frame, DecodeError> {
    if data.len() < BULK_HEADER_LEN {
        return Err(DecodeError::TooShort {
            expected: BULK_HEADER_LEN,
            got: data.len(),
        });
    }
    // Safe indexing: length checked above. `BULK_FILE_ID_OFFSET + 4`
    // and `BULK_OFFSET_OFFSET + 4` are both `<= BULK_HEADER_LEN`.
    let file_id = u32::from_le_bytes([
        data[BULK_FILE_ID_OFFSET],
        data[BULK_FILE_ID_OFFSET + 1],
        data[BULK_FILE_ID_OFFSET + 2],
        data[BULK_FILE_ID_OFFSET + 3],
    ]);
    let offset = u32::from_le_bytes([
        data[BULK_OFFSET_OFFSET],
        data[BULK_OFFSET_OFFSET + 1],
        data[BULK_OFFSET_OFFSET + 2],
        data[BULK_OFFSET_OFFSET + 3],
    ]);
    let payload = data.slice(BULK_HEADER_LEN..);
    if offset == BULK_END_OFFSET_SENTINEL {
        Ok(Frame::BulkEnd { file_id, payload })
    } else {
        Ok(Frame::Bulk { file_id, offset, payload })
    }
}

fn parse_handshake(data: Bytes) -> Result<Frame, DecodeError> {
    if data.len() < HANDSHAKE_TYPE_LEN {
        return Err(DecodeError::TooShort {
            expected: HANDSHAKE_TYPE_LEN,
            got: data.len(),
        });
    }
    let handshake_type = u16::from_le_bytes([data[0], data[1]]);
    Ok(Frame::Handshake {
        handshake_type,
        payload: data,
    })
}
