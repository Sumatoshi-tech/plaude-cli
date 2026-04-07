//! The [`Frame`] enum and the [`AuthStatus`] decision type.
//!
//! A `Frame` is the fully-parsed, transport-agnostic view of one
//! notification on the vendor notify characteristic `0x2BB0`.

use bytes::Bytes;

/// A single parsed notification from the Plaud device.
///
/// The variant is chosen by the first byte (or first two bytes, for
/// the `0xFE1x` handshake preamble range) of the notification.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Frame {
    /// Control frame carrying an opcode response or, on the reverse
    /// direction, an opcode request. Magic byte `0x01`.
    Control {
        /// 16-bit little-endian opcode.
        opcode: u16,
        /// Opcode payload (everything after the 3-byte header).
        payload: Bytes,
    },

    /// Bulk data frame carrying one chunk of an ongoing file
    /// transfer. Magic byte `0x02`.
    Bulk {
        /// File identifier (stable within a single bulk transfer).
        file_id: u32,
        /// Byte offset of this chunk inside the source file.
        offset: u32,
        /// Chunk payload (everything after the 10-byte header).
        payload: Bytes,
    },

    /// Terminal bulk frame signalling end-of-stream. Identified by
    /// its `offset` field being set to `0xFFFFFFFF` on the wire. The
    /// `payload` is preserved for diagnostics but should not be
    /// concatenated with the reassembled file bytes.
    BulkEnd {
        /// Same file identifier as the preceding `Bulk` frames.
        file_id: u32,
        /// Raw bytes carried by the terminal frame. Not file data.
        payload: Bytes,
    },

    /// Pre-authentication handshake frame (Mode B, RSA + ChaCha20).
    /// Decoded so the transport layer can detect when a newer
    /// firmware is in use; the payload is not yet parsed in M2.
    Handshake {
        /// Handshake type byte read as a `u16 LE` at offset 0.
        /// Observed values: `0xFE11`, `0xFE12`.
        handshake_type: u16,
        /// Raw bytes including the 2-byte handshake type prefix.
        payload: Bytes,
    },
}

/// Outcome of the device's response to an auth request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthStatus {
    /// Status byte `0x00` — token accepted; vendor opcodes will work.
    Accepted,
    /// Status byte `0x01` — token rejected; connection stays open but
    /// every subsequent vendor opcode is silently dropped.
    Rejected,
}
