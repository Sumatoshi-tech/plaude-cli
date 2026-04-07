//! Pure-Rust btsnoop log parser that extracts a V0095-style Plaud
//! auth token from the first ATT Write Command to the vendor write
//! characteristic.
//!
//! # Scope
//!
//! * Accepts btsnoop **version 1** files only (the only format
//!   currently defined).
//! * Walks records looking for the first HCI ACL packet whose L2CAP
//!   channel is `0x0004` (ATT), whose ATT opcode is `0x52`
//!   (Write Command), whose handle is
//!   [`constants::VENDOR_WRITE_HANDLE`], and whose value starts with
//!   [`constants::AUTH_FRAME_PREFIX`].
//! * **Does not** reassemble fragmented L2CAP PDUs. Our V0095 captures
//!   come from Samsung phones that negotiate MTU ≥ 247, so the
//!   ≤ 38-byte auth frame always fits in a single ACL. A capture from
//!   an MTU-23 phone would fragment and we fall through to
//!   [`BtsnoopError::NoAuthFrameFound`].
//!
//! # Evidence
//!
//! Frame layout is specified in `docs/protocol/ble-commands.md` §1.
//! Reference Python implementation:
//! `specs/re/captures/ble-live-tests/scripts/plaud-test2c.py`.

use plaud_domain::{AuthToken, AuthTokenError};
use thiserror::Error;

use crate::constants::{
    ACL_HEADER_LEN, ATT_OPCODE_WRITE_COMMAND, ATT_WRITE_VALUE_OFFSET, AUTH_FRAME_PREFIX, BTSNOOP_HEADER_LEN, BTSNOOP_MAGIC,
    BTSNOOP_RECORD_HEADER_LEN, BTSNOOP_VERSION, HCI_PACKET_TYPE_ACL, L2CAP_CID_ATT, L2CAP_HEADER_LEN, VENDOR_WRITE_HANDLE,
};

/// Errors produced by [`extract_auth_token`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum BtsnoopError {
    /// Input did not start with the `"btsnoop\0"` magic bytes.
    #[error("file is not a btsnoop log (bad magic bytes)")]
    InvalidMagic,

    /// The btsnoop version field was not `1`. Only v1 is supported.
    #[error("unsupported btsnoop version {got} (only version {expected} is supported)")]
    UnsupportedVersion {
        /// Observed version.
        got: u32,
        /// Version this implementation supports.
        expected: u32,
    },

    /// The btsnoop file header is shorter than the 16-byte minimum.
    #[error("btsnoop header truncated: need {expected} bytes, got {got}")]
    TruncatedHeader {
        /// Required minimum length.
        expected: usize,
        /// Observed length.
        got: usize,
    },

    /// A record announces a packet longer than the remaining bytes in
    /// the file. The log is corrupt or truncated.
    #[error("btsnoop record truncated inside packet data")]
    TruncatedRecord,

    /// The log parsed cleanly but contained no ATT Write Command to
    /// the vendor auth handle with the expected V0095 prefix.
    #[error("no auth frame found in btsnoop log")]
    NoAuthFrameFound,

    /// A candidate auth frame was found but the trailing token bytes
    /// did not satisfy [`AuthToken::new`]'s invariants.
    #[error("auth frame bytes do not form a valid AuthToken: {0}")]
    InvalidToken(#[from] AuthTokenError),
}

/// Walk a btsnoop log and return the first auth token found.
///
/// # Errors
///
/// Returns [`BtsnoopError`] with a specific variant describing which
/// step failed — header validation, record walking, or the final
/// token-construction step.
pub fn extract_auth_token(log_bytes: &[u8]) -> Result<AuthToken, BtsnoopError> {
    validate_header(log_bytes)?;
    let mut offset = BTSNOOP_HEADER_LEN;
    while offset < log_bytes.len() {
        let (packet, next_offset) = read_record(log_bytes, offset)?;
        if let Some(token_bytes) = try_extract_auth_from_hci(packet) {
            return Ok(AuthToken::new(token_bytes)?);
        }
        offset = next_offset;
    }
    Err(BtsnoopError::NoAuthFrameFound)
}

fn validate_header(log_bytes: &[u8]) -> Result<(), BtsnoopError> {
    if log_bytes.len() < BTSNOOP_HEADER_LEN {
        return Err(BtsnoopError::TruncatedHeader {
            expected: BTSNOOP_HEADER_LEN,
            got: log_bytes.len(),
        });
    }
    if &log_bytes[..BTSNOOP_MAGIC.len()] != BTSNOOP_MAGIC {
        return Err(BtsnoopError::InvalidMagic);
    }
    let version = u32::from_be_bytes([log_bytes[8], log_bytes[9], log_bytes[10], log_bytes[11]]);
    if version != BTSNOOP_VERSION {
        return Err(BtsnoopError::UnsupportedVersion {
            got: version,
            expected: BTSNOOP_VERSION,
        });
    }
    Ok(())
}

fn read_record(log_bytes: &[u8], offset: usize) -> Result<(&[u8], usize), BtsnoopError> {
    if log_bytes.len() < offset + BTSNOOP_RECORD_HEADER_LEN {
        return Err(BtsnoopError::TruncatedRecord);
    }
    let header = &log_bytes[offset..offset + BTSNOOP_RECORD_HEADER_LEN];
    // original_length (0..4) is the wire length; included_length (4..8)
    // is how many bytes actually made it into the file. We use the
    // included length because that is what we can read.
    let included_length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let data_start = offset + BTSNOOP_RECORD_HEADER_LEN;
    let data_end = data_start.saturating_add(included_length);
    if log_bytes.len() < data_end {
        return Err(BtsnoopError::TruncatedRecord);
    }
    Ok((&log_bytes[data_start..data_end], data_end))
}

fn try_extract_auth_from_hci(packet: &[u8]) -> Option<String> {
    // 1) H4 packet type byte.
    let (&packet_type, rest) = packet.split_first()?;
    if packet_type != HCI_PACKET_TYPE_ACL {
        return None;
    }
    // 2) ACL header (4 bytes): handle+flags (u16 LE) + data length (u16 LE).
    if rest.len() < ACL_HEADER_LEN {
        return None;
    }
    let acl_data_len = u16::from_le_bytes([rest[2], rest[3]]) as usize;
    let l2cap_and_beyond = rest.get(ACL_HEADER_LEN..ACL_HEADER_LEN + acl_data_len)?;
    // 3) L2CAP header (4 bytes): pdu length (u16 LE) + cid (u16 LE).
    if l2cap_and_beyond.len() < L2CAP_HEADER_LEN {
        return None;
    }
    let l2cap_pdu_len = u16::from_le_bytes([l2cap_and_beyond[0], l2cap_and_beyond[1]]) as usize;
    let cid = u16::from_le_bytes([l2cap_and_beyond[2], l2cap_and_beyond[3]]);
    if cid != L2CAP_CID_ATT {
        return None;
    }
    // Reject fragmented PDUs — we don't reassemble in M4.
    let att_pdu = l2cap_and_beyond.get(L2CAP_HEADER_LEN..L2CAP_HEADER_LEN + l2cap_pdu_len)?;
    // 4) ATT: 1-byte opcode, 2-byte handle (LE), value bytes.
    let (&att_opcode, att_rest) = att_pdu.split_first()?;
    if att_opcode != ATT_OPCODE_WRITE_COMMAND {
        return None;
    }
    if att_rest.len() < 2 {
        return None;
    }
    let handle = u16::from_le_bytes([att_rest[0], att_rest[1]]);
    if handle != VENDOR_WRITE_HANDLE {
        return None;
    }
    // The value starts at offset 3 relative to att_pdu (opcode + 2 handle bytes).
    let value = att_pdu.get(ATT_WRITE_VALUE_OFFSET..)?;
    // 5) Auth frame prefix match.
    if !value.starts_with(AUTH_FRAME_PREFIX) {
        return None;
    }
    let token_bytes = &value[AUTH_FRAME_PREFIX.len()..];
    if !token_bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    String::from_utf8(token_bytes.to_vec()).ok()
}
