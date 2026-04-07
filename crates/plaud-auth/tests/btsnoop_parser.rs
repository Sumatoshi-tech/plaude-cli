//! Tests for [`plaud_auth::btsnoop::extract_auth_token`].
//!
//! Fixture logs are built in memory so no raw capture bytes land
//! in the repo.
//!
//! Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md

use plaud_auth::btsnoop::{BtsnoopError, extract_auth_token};

const BTSNOOP_MAGIC: &[u8] = b"btsnoop\0";
const BTSNOOP_VERSION_ONE: u32 = 1;
const DATALINK_HCI_UART: u32 = 1002;
const AUTH_OPCODE: u8 = 0x52;
const VENDOR_WRITE_HANDLE: u16 = 0x000D;
const HCI_PACKET_TYPE_ACL: u8 = 0x02;
const L2CAP_CID_ATT: u16 = 0x0004;
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";

/// Build a btsnoop v1 file containing a single ACL packet with an
/// ATT Write Command to `VENDOR_WRITE_HANDLE` carrying an auth
/// frame with `token_bytes` as its hex payload.
fn make_btsnoop_with_auth_write(token_bytes: &[u8]) -> Vec<u8> {
    // ---- ATT PDU: opcode(1) + handle(2) + value(N) ----
    let mut att = Vec::new();
    att.push(AUTH_OPCODE);
    att.extend_from_slice(&VENDOR_WRITE_HANDLE.to_le_bytes());
    // auth frame prefix: 01 01 00 02 00 00
    att.extend_from_slice(&[0x01, 0x01, 0x00, 0x02, 0x00, 0x00]);
    att.extend_from_slice(token_bytes);
    // ---- L2CAP: pdu_len(2) + cid(2) + att ----
    let mut l2cap = Vec::new();
    l2cap.extend_from_slice(&(att.len() as u16).to_le_bytes());
    l2cap.extend_from_slice(&L2CAP_CID_ATT.to_le_bytes());
    l2cap.extend_from_slice(&att);
    // ---- ACL: handle+flags(2) + data_len(2) + l2cap ----
    let mut acl = Vec::new();
    acl.extend_from_slice(&0x0001_u16.to_le_bytes()); // handle 1, flags 0
    acl.extend_from_slice(&(l2cap.len() as u16).to_le_bytes());
    acl.extend_from_slice(&l2cap);
    // ---- HCI H4: packet_type(1) + acl ----
    let mut h4 = Vec::new();
    h4.push(HCI_PACKET_TYPE_ACL);
    h4.extend_from_slice(&acl);
    // ---- btsnoop record header (24 bytes) + packet ----
    let mut record = Vec::new();
    let orig_len = h4.len() as u32;
    record.extend_from_slice(&orig_len.to_be_bytes()); // original_length
    record.extend_from_slice(&orig_len.to_be_bytes()); // included_length
    record.extend_from_slice(&0_u32.to_be_bytes()); // packet_flags
    record.extend_from_slice(&0_u32.to_be_bytes()); // cumulative_drops
    record.extend_from_slice(&0_i64.to_be_bytes()); // timestamp
    record.extend_from_slice(&h4);
    // ---- btsnoop file header ----
    let mut file = Vec::new();
    file.extend_from_slice(BTSNOOP_MAGIC);
    file.extend_from_slice(&BTSNOOP_VERSION_ONE.to_be_bytes());
    file.extend_from_slice(&DATALINK_HCI_UART.to_be_bytes());
    file.extend_from_slice(&record);
    file
}

#[test]
fn extracts_token_from_a_well_formed_btsnoop_log() {
    let log = make_btsnoop_with_auth_write(SAMPLE_TOKEN.as_bytes());
    let token = extract_auth_token(&log).expect("extraction succeeds");
    assert_eq!(token.as_str(), SAMPLE_TOKEN);
}

#[test]
fn rejects_log_with_wrong_magic_bytes() {
    let mut log = make_btsnoop_with_auth_write(SAMPLE_TOKEN.as_bytes());
    log[0] = b'x';
    let err = extract_auth_token(&log).unwrap_err();
    assert_eq!(err, BtsnoopError::InvalidMagic);
}

#[test]
fn rejects_log_with_unsupported_version() {
    let mut log = make_btsnoop_with_auth_write(SAMPLE_TOKEN.as_bytes());
    // Version field starts at offset 8, 4 bytes big-endian.
    log[8..12].copy_from_slice(&9_u32.to_be_bytes());
    let err = extract_auth_token(&log).unwrap_err();
    assert!(matches!(err, BtsnoopError::UnsupportedVersion { got: 9, .. }));
}

#[test]
fn rejects_log_that_is_too_short_for_the_header() {
    let short = [0u8; 4];
    let err = extract_auth_token(&short).unwrap_err();
    assert!(matches!(err, BtsnoopError::TruncatedHeader { .. }));
}

#[test]
fn returns_no_auth_frame_found_when_no_matching_write_exists() {
    // Build a log whose only record is a Write Command to a wrong
    // handle (`0xFFFF`).
    let mut att = Vec::new();
    att.push(AUTH_OPCODE);
    att.extend_from_slice(&0xFFFF_u16.to_le_bytes());
    att.extend_from_slice(&[0x01, 0x01, 0x00, 0x02, 0x00, 0x00]);
    att.extend_from_slice(SAMPLE_TOKEN.as_bytes());
    let mut l2cap = Vec::new();
    l2cap.extend_from_slice(&(att.len() as u16).to_le_bytes());
    l2cap.extend_from_slice(&L2CAP_CID_ATT.to_le_bytes());
    l2cap.extend_from_slice(&att);
    let mut acl = Vec::new();
    acl.extend_from_slice(&0x0001_u16.to_le_bytes());
    acl.extend_from_slice(&(l2cap.len() as u16).to_le_bytes());
    acl.extend_from_slice(&l2cap);
    let mut h4 = Vec::new();
    h4.push(HCI_PACKET_TYPE_ACL);
    h4.extend_from_slice(&acl);
    let orig_len = h4.len() as u32;
    let mut record = Vec::new();
    record.extend_from_slice(&orig_len.to_be_bytes());
    record.extend_from_slice(&orig_len.to_be_bytes());
    record.extend_from_slice(&0_u32.to_be_bytes());
    record.extend_from_slice(&0_u32.to_be_bytes());
    record.extend_from_slice(&0_i64.to_be_bytes());
    record.extend_from_slice(&h4);
    let mut file = Vec::new();
    file.extend_from_slice(BTSNOOP_MAGIC);
    file.extend_from_slice(&BTSNOOP_VERSION_ONE.to_be_bytes());
    file.extend_from_slice(&DATALINK_HCI_UART.to_be_bytes());
    file.extend_from_slice(&record);

    let err = extract_auth_token(&file).unwrap_err();
    assert_eq!(err, BtsnoopError::NoAuthFrameFound);
}

#[test]
fn rejects_truncated_record() {
    // Build a valid log, then chop bytes from the middle of its
    // packet data.
    let mut log = make_btsnoop_with_auth_write(SAMPLE_TOKEN.as_bytes());
    // Remove the last 10 bytes so the record claims more data than
    // is present.
    log.truncate(log.len() - 10);
    let err = extract_auth_token(&log).unwrap_err();
    assert_eq!(err, BtsnoopError::TruncatedRecord);
}
