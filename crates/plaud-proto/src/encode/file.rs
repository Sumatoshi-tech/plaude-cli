//! File-transfer control-frame encoders.

use bytes::Bytes;

use crate::{
    constants::{
        READ_FILE_CHUNK_FILE_ID_OFFSET, READ_FILE_CHUNK_LENGTH_OFFSET, READ_FILE_CHUNK_OFFSET_OFFSET, READ_FILE_CHUNK_PAYLOAD_LEN,
    },
    encode::control,
    opcode::OPCODE_READ_FILE_CHUNK,
};

/// Encode a `ReadFileChunk` (opcode `0x001C`) control frame.
///
/// Payload layout: `<file_id u32 LE> <offset u32 LE> <length u32 LE>`.
/// Triggers the subsequent bulk `0x02`-magic frame stream from the
/// device.
///
/// Example byte layout (from `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md`):
///
/// ```text
/// 01 1c 00  58 66 d2 69  80 4c 01 00  c0 57 01 00
/// ```
#[must_use]
pub fn read_file_chunk(file_id: u32, offset: u32, length: u32) -> Bytes {
    let mut payload = [0u8; READ_FILE_CHUNK_PAYLOAD_LEN];
    write_u32_le_into(&mut payload, READ_FILE_CHUNK_FILE_ID_OFFSET, file_id);
    write_u32_le_into(&mut payload, READ_FILE_CHUNK_OFFSET_OFFSET, offset);
    write_u32_le_into(&mut payload, READ_FILE_CHUNK_LENGTH_OFFSET, length);
    control(OPCODE_READ_FILE_CHUNK, &payload)
}

/// Write a little-endian `u32` into a mutable byte slice at `offset`.
/// Precondition: `offset + 4 <= buf.len()`.
fn write_u32_le_into(buf: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
    buf[offset + 2] = bytes[2];
    buf[offset + 3] = bytes[3];
}
