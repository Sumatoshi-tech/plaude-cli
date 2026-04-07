//! Tests for [`plaud_proto::encode::file::read_file_chunk`].
//!
//! The expected wire bytes come from the 0day session-C capture
//! (`specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md`), where
//! the phone asked for `(file_id=0x69D26658, offset=85632, length=88000)`
//! and produced this exact frame:
//!
//! ```text
//! 01 1c 00 58 66 d2 69 80 4c 01 00 c0 57 01 00
//! ```
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use plaud_proto::{Frame, encode::file::read_file_chunk, parse_notification};

const CAPTURED_FILE_ID: u32 = 0x69D26658;
const CAPTURED_OFFSET: u32 = 0x00014C80;
const CAPTURED_LENGTH: u32 = 0x000157C0;
const EXPECTED_CAPTURED_FRAME: &[u8] = &[
    0x01, 0x1c, 0x00, // control header + opcode 0x001C LE
    0x58, 0x66, 0xd2, 0x69, // file_id u32 LE
    0x80, 0x4c, 0x01, 0x00, // offset u32 LE
    0xc0, 0x57, 0x01, 0x00, // length u32 LE
];

#[test]
fn read_file_chunk_matches_captured_session_c_wire_bytes() {
    let bytes = read_file_chunk(CAPTURED_FILE_ID, CAPTURED_OFFSET, CAPTURED_LENGTH);
    assert_eq!(bytes.as_ref(), EXPECTED_CAPTURED_FRAME);
}

#[test]
fn read_file_chunk_round_trips_through_parse_notification() {
    let encoded = read_file_chunk(CAPTURED_FILE_ID, CAPTURED_OFFSET, CAPTURED_LENGTH);
    let parsed = parse_notification(encoded).expect("parses");
    match parsed {
        Frame::Control { opcode, payload } => {
            assert_eq!(opcode, 0x001C);
            assert_eq!(payload.len(), 12);
        }
        other => panic!("expected Control, got {other:?}"),
    }
}
