//! Frame encoders.
//!
//! Every encoder returns a freshly-owned [`bytes::Bytes`] suitable for
//! passing to a BLE central's `write_gatt_char(...)`. The low-level
//! builder is [`control`], which prepends the 3-byte control header
//! to an arbitrary payload. Typed wrappers in the submodules cover
//! every opcode that has a concrete wire example in the btsnoop
//! walkthroughs.

pub mod auth;
pub mod device;
pub mod file;
pub mod metadata;
pub mod recording;
pub mod settings;

use bytes::{BufMut, Bytes, BytesMut};

use crate::constants::{CONTROL_HEADER_LEN, FRAME_TYPE_CONTROL};

/// Build a control frame with the given opcode and payload.
///
/// Layout: `01 <opcode u16 LE> <payload ..>`. Allocates exactly one
/// buffer sized to the final frame length.
#[must_use]
pub fn control(opcode: u16, payload: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(CONTROL_HEADER_LEN + payload.len());
    buf.put_u8(FRAME_TYPE_CONTROL);
    buf.put_u16_le(opcode);
    buf.put_slice(payload);
    buf.freeze()
}

/// Build a nullary control frame (opcode with zero-byte payload).
/// Convenience wrapper over [`control`] for the many opcodes observed
/// to be called with no arguments (`GetDeviceName`, `GetState`,
/// `GetStorageStats`, and friends).
#[must_use]
pub fn nullary(opcode: u16) -> Bytes {
    control(opcode, &[])
}
