//! `frames_for` — synthesise the bulk-frame sequence a real Plaud
//! device would emit for a `ReadFileChunk` that covers `data`.
//!
//! This helper exists so M5's BLE-transport tests and M7's file-pull
//! tests can feed a known-good multi-frame stream through
//! [`plaud_proto::parse_notification`] without needing to capture
//! real device bytes for every fixture.

use bytes::{BufMut, Bytes, BytesMut};
use plaud_proto::Frame;

use crate::constants::BULK_FRAME_CHUNK_BYTES;

/// Bytes present in the 10-byte bulk frame header: magic + reserved
/// + file id (4) + offset (4).
const BULK_HEADER_LEN: usize = 10;

/// Magic byte marking a bulk data frame.
const BULK_MAGIC: u8 = 0x02;

/// End-of-stream sentinel used by the terminal frame.
const END_SENTINEL_OFFSET: u32 = 0xFFFF_FFFF;

/// Produce the wire-level byte sequence a real device would emit
/// for a bulk transfer of `data` under `file_id`, starting at
/// `start_offset`. The last element of the returned vector is the
/// terminal `Frame::BulkEnd` with the `0xFFFFFFFF` sentinel.
///
/// The bytes are **not** concatenated — tests typically want
/// frame-by-frame access so they can feed each one through
/// [`plaud_proto::parse_notification`] independently.
#[must_use]
pub fn frames_for(file_id: u32, start_offset: u32, data: &[u8]) -> Vec<Frame> {
    let mut frames = Vec::with_capacity(data.len().div_ceil(BULK_FRAME_CHUNK_BYTES) + 1);
    let mut local_offset: u32 = 0;
    for chunk in data.chunks(BULK_FRAME_CHUNK_BYTES) {
        let chunk_offset = start_offset.saturating_add(local_offset);
        frames.push(Frame::Bulk {
            file_id,
            offset: chunk_offset,
            payload: Bytes::copy_from_slice(chunk),
        });
        local_offset = local_offset.saturating_add(chunk.len() as u32);
    }
    frames.push(Frame::BulkEnd {
        file_id,
        payload: Bytes::new(),
    });
    frames
}

/// Serialise a frame produced by [`frames_for`] back into the
/// byte-level wire form the device would put on the notify
/// characteristic. Useful for round-trip tests that want to
/// verify `parse_notification(serialise_bulk(frame))` returns an
/// equivalent `Frame`.
#[must_use]
pub fn serialise_bulk(frame: &Frame) -> Bytes {
    let (file_id, offset, payload) = match frame {
        Frame::Bulk { file_id, offset, payload } => (*file_id, *offset, payload.clone()),
        Frame::BulkEnd { file_id, payload } => (*file_id, END_SENTINEL_OFFSET, payload.clone()),
        _ => return Bytes::new(),
    };
    let mut buf = BytesMut::with_capacity(BULK_HEADER_LEN + payload.len());
    buf.put_u8(BULK_MAGIC);
    buf.put_u32_le(file_id);
    buf.put_u32_le(offset);
    buf.put_u8(payload.len() as u8); // chunk_len
    buf.put_slice(&payload);
    buf.freeze()
}
