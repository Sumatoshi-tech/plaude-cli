//! `BulkReassembler` — state machine that validates and concatenates
//! a sequence of `plaud_proto::Frame::Bulk` frames until it sees a
//! terminal `Frame::BulkEnd`.
//!
//! Semantics lifted from
//! [`docs/protocol/ble-commands.md`](../../../docs/protocol/ble-commands.md)
//! §2 and the evidence in
//! [`specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md`](../../../specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md).

use plaud_proto::Frame;
use plaud_transport::{Error, Result};

/// Error message emitted when bulk frames arrive out of order.
const ERR_NON_MONOTONE_OFFSET: &str = "bulk frame offset went backwards";

/// Error message emitted when a non-bulk frame appears inside a
/// bulk stream.
const ERR_UNEXPECTED_FRAME: &str = "unexpected non-bulk frame inside bulk stream";

/// Error message emitted when a bulk stream contains frames for a
/// different `file_id` than the first frame.
const ERR_FILE_ID_MISMATCH: &str = "bulk stream file_id changed mid-stream";

/// Sentinel returned by [`BulkReassembler::feed`] to tell the caller
/// whether the stream is still open or the terminal frame arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedStatus {
    /// The stream is still open. `BulkReassembler::feed` may be
    /// called again.
    InProgress,
    /// `BulkEnd` received. The caller should now call
    /// [`BulkReassembler::finish`] to take ownership of the bytes.
    Done,
}

/// Incrementally accumulates bulk-frame payloads until a `BulkEnd`
/// terminates the stream.
#[derive(Debug)]
pub struct BulkReassembler {
    file_id: Option<u32>,
    next_expected_offset: u32,
    buffer: Vec<u8>,
    done: bool,
}

impl BulkReassembler {
    /// Create an empty reassembler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_id: None,
            next_expected_offset: 0,
            buffer: Vec::new(),
            done: false,
        }
    }

    /// Feed one decoded frame into the reassembler.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Protocol`] if the frame is not a `Bulk` or
    /// `BulkEnd`, if its offset is lower than the last frame's end,
    /// or if its `file_id` differs from the stream's first frame.
    pub fn feed(&mut self, frame: Frame) -> Result<FeedStatus> {
        match frame {
            Frame::Bulk { file_id, offset, payload } => {
                self.check_file_id(file_id)?;
                if offset < self.next_expected_offset {
                    return Err(Error::Protocol(ERR_NON_MONOTONE_OFFSET.to_owned()));
                }
                self.buffer.extend_from_slice(&payload);
                // Use the actual payload length to advance the
                // expected offset — matches the device's 80-byte
                // default step but tolerates partial chunks.
                let advance = u32::try_from(payload.len()).unwrap_or(u32::MAX);
                self.next_expected_offset = offset.saturating_add(advance);
                Ok(FeedStatus::InProgress)
            }
            Frame::BulkEnd { file_id, .. } => {
                self.check_file_id(file_id)?;
                self.done = true;
                Ok(FeedStatus::Done)
            }
            other => Err(Error::Protocol(format!("{ERR_UNEXPECTED_FRAME}: {other:?}"))),
        }
    }

    /// Take ownership of the reassembled bytes. Returns an error if
    /// the stream has not reached its terminal frame yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Protocol`] if [`FeedStatus::Done`] has not
    /// been observed.
    pub fn finish(self) -> Result<Vec<u8>> {
        if !self.done {
            return Err(Error::Protocol("bulk stream finished without BulkEnd".to_owned()));
        }
        Ok(self.buffer)
    }

    fn check_file_id(&mut self, incoming: u32) -> Result<()> {
        match self.file_id {
            Some(existing) if existing != incoming => Err(Error::Protocol(ERR_FILE_ID_MISMATCH.to_owned())),
            Some(_) => Ok(()),
            None => {
                self.file_id = Some(incoming);
                Ok(())
            }
        }
    }
}

impl Default for BulkReassembler {
    fn default() -> Self {
        Self::new()
    }
}
