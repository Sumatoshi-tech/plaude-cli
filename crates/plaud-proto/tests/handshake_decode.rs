//! Tests for the handshake preamble detection path.
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use bytes::Bytes;
use plaud_proto::{Frame, parse_notification};

const HANDSHAKE_PREHANDSHAKE_CNF: &[u8] = &[0x12, 0xfe, 0x01, 0x01, 0xaa, 0xbb]; // 0xFE12 LE + arbitrary payload
const HANDSHAKE_FILE_SYNC_PREAMBLE: &[u8] = &[0x11, 0xfe, 0x00, 0x00];
const HANDSHAKE_TYPE_PREHANDSHAKE_CNF: u16 = 0xFE12;
const HANDSHAKE_TYPE_FILE_SYNC_PREAMBLE: u16 = 0xFE11;

#[test]
fn detects_prehandshake_cnf_by_high_byte_fe() {
    let frame = parse_notification(Bytes::from_static(HANDSHAKE_PREHANDSHAKE_CNF)).expect("parses");
    match frame {
        Frame::Handshake { handshake_type, payload } => {
            assert_eq!(handshake_type, HANDSHAKE_TYPE_PREHANDSHAKE_CNF);
            assert_eq!(payload.len(), HANDSHAKE_PREHANDSHAKE_CNF.len());
        }
        other => panic!("expected Handshake, got {other:?}"),
    }
}

#[test]
fn detects_file_sync_preamble_by_high_byte_fe() {
    let frame = parse_notification(Bytes::from_static(HANDSHAKE_FILE_SYNC_PREAMBLE)).expect("parses");
    match frame {
        Frame::Handshake { handshake_type, .. } => {
            assert_eq!(handshake_type, HANDSHAKE_TYPE_FILE_SYNC_PREAMBLE);
        }
        other => panic!("expected Handshake, got {other:?}"),
    }
}

#[test]
fn handshake_with_only_one_byte_falls_through_to_control_or_unknown() {
    // A single byte `0x11` cannot be a handshake (we look at byte 1
    // for `0xFE`). It should be rejected as unknown frame type.
    let raw = Bytes::from_static(&[0x11]);
    let result = parse_notification(raw);
    assert!(result.is_err());
}
