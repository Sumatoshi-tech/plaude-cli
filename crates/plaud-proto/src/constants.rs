//! Wire-format constants for the tinnotech pen-BLE protocol used by
//! Plaud devices.
//!
//! Every magic byte, frame length, sentinel, and offset the codec
//! touches lives here as a named `const`, so the codec itself
//! contains no bare literals. Changing a protocol constant is a
//! single-edit-site operation.

/// Frame-type byte for control frames (`01 <opcode LE> <payload>`).
pub const FRAME_TYPE_CONTROL: u8 = 0x01;

/// Frame-type byte for bulk data frames
/// (`02 <file_id:4> <offset:4> <chunk_len:1> <payload>`).
///
/// Note: earlier protocol analysis assumed a reserved `0x00` byte at
/// offset 1, but live device testing shows the file_id starts
/// immediately after the magic byte. The `0x00` in the btsnoop was
/// the high byte of a small file_id.
pub const FRAME_TYPE_BULK: u8 = 0x02;

/// High byte (as seen on the wire at offset 1) that flags a
/// pre-auth handshake frame in the newer RSA + ChaCha20-Poly1305
/// auth mode. Pre-auth frames are read as a `u16` LE at offset 0 and
/// compared against [`HANDSHAKE_TYPE_FILE_SYNC_PREAMBLE`] /
/// [`HANDSHAKE_TYPE_PREHANDSHAKE_CNF`].
pub const HANDSHAKE_SIGNATURE_HIGH_BYTE: u8 = 0xFE;

/// Handshake type `0xFE11` — observed in APK analysis as the
/// file-sync pre-handshake preamble. Not yet encoded.
pub const HANDSHAKE_TYPE_FILE_SYNC_PREAMBLE: u16 = 0xFE11;

/// Handshake type `0xFE12` — `STICK_PREHANDSHAKE_CNF` in the SDK
/// source. Carries the RSA-encrypted ChaCha20-Poly1305 handshake
/// material for Mode B auth. Not yet encoded.
pub const HANDSHAKE_TYPE_PREHANDSHAKE_CNF: u16 = 0xFE12;

/// Offset value used by the device to terminate a bulk transfer
/// stream. A bulk frame carrying this offset is a `BulkEnd` rather
/// than a data chunk.
pub const BULK_END_OFFSET_SENTINEL: u32 = 0xFFFF_FFFF;

/// Length of the control-frame fixed header (`type + opcode u16 LE`).
pub const CONTROL_HEADER_LEN: usize = 3;

/// Minimum length of a bulk frame (header only, zero-byte payload).
/// Bulk header = type(1) + file_id(4) + offset(4) + chunk_len(1) = 10.
pub const BULK_HEADER_LEN: usize = 10;

/// Byte-offset where the `file_id u32 LE` starts inside a bulk frame.
/// Immediately after the magic byte `0x02`.
pub const BULK_FILE_ID_OFFSET: usize = 1;

/// Byte-offset where the `offset u32 LE` starts inside a bulk frame.
pub const BULK_OFFSET_OFFSET: usize = 5;

/// Length of a `u16` integer in bytes. Used to size header fields.
pub const U16_SIZE: usize = 2;

/// Length of a `u32` integer in bytes.
pub const U32_SIZE: usize = 4;

/// Length of a handshake preamble prefix (the `u16` type byte pair).
pub const HANDSHAKE_TYPE_LEN: usize = U16_SIZE;

// ---------------------------------------------------------------------
// Auth frame layout (opcode 0x0001, V0095 plaintext path)
// ---------------------------------------------------------------------
//
// Observed wire bytes (token redacted):
//
//     01 01 00 02 00 00 <32 ASCII hex chars>
//
// That is:
//   * `01 01 00` — control header (type + opcode 0x0001 LE)
//   * `02 00`    — constant u16 LE (the `packInt(2L)` in C9555a0.java)
//   * `00`       — single version byte (value 0 observed on V0095)
//
// The total prefix before the token is [`AUTH_PREFIX`].

/// Fixed prefix emitted before the token bytes in a V0095-compatible
/// auth frame.
pub const AUTH_PREFIX: &[u8] = &[
    FRAME_TYPE_CONTROL,
    OPCODE_AUTHENTICATE_LO,
    OPCODE_AUTHENTICATE_HI,
    AUTH_LENGTH_CONST_LO,
    AUTH_LENGTH_CONST_HI,
    AUTH_VERSION_BYTE,
];

/// Low byte of the [`OPCODE_AUTHENTICATE`] opcode when written in LE.
pub const OPCODE_AUTHENTICATE_LO: u8 = 0x01;

/// High byte of the [`OPCODE_AUTHENTICATE`] opcode when written in LE.
pub const OPCODE_AUTHENTICATE_HI: u8 = 0x00;

/// Low byte of the `packInt(2L)` length-constant field in V0095 auth.
pub const AUTH_LENGTH_CONST_LO: u8 = 0x02;

/// High byte of the `packInt(2L)` length-constant field in V0095 auth.
pub const AUTH_LENGTH_CONST_HI: u8 = 0x00;

/// Single-byte version field written after the length constant.
pub const AUTH_VERSION_BYTE: u8 = 0x00;

// ---------------------------------------------------------------------
// Auth response layout (device → phone, V0095 path)
// ---------------------------------------------------------------------

/// Length of the minimum auth-response control payload we can parse
/// (status byte + the stable 13-byte capability tuple observed on
/// V0095).
pub const AUTH_RESPONSE_MIN_PAYLOAD_LEN: usize = 1;

/// Status byte value indicating the device accepted the token.
pub const AUTH_STATUS_ACCEPTED: u8 = 0x00;

/// Status byte value indicating the device rejected the token but
/// kept the connection open for "silent soft-reject" behaviour.
pub const AUTH_STATUS_REJECTED: u8 = 0x01;

// ---------------------------------------------------------------------
// Opcode-specific payload sizes
// ---------------------------------------------------------------------

/// Payload length of the `ReadFileChunk` opcode (`file_id u32 + offset u32 + length u32`).
pub const READ_FILE_CHUNK_PAYLOAD_LEN: usize = 12;

/// Offset of the `file_id` field inside the `ReadFileChunk` payload.
pub const READ_FILE_CHUNK_FILE_ID_OFFSET: usize = 0;

/// Offset of the `offset` field inside the `ReadFileChunk` payload.
pub const READ_FILE_CHUNK_OFFSET_OFFSET: usize = 4;

/// Offset of the `length` field inside the `ReadFileChunk` payload.
pub const READ_FILE_CHUNK_LENGTH_OFFSET: usize = 8;

/// `SetPrivacy(on=true)` wire byte.
pub const PRIVACY_ON: u8 = 0x01;

/// `SetPrivacy(on=false)` wire byte.
pub const PRIVACY_OFF: u8 = 0x00;

/// `CloseSession` single-byte argument (value `0x00` observed on V0095).
pub const CLOSE_SESSION_ARG: u8 = 0x00;
