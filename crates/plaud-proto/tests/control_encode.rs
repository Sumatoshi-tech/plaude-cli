//! Tests for [`plaud_proto::encode::control`] and
//! [`plaud_proto::encode::nullary`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use plaud_proto::{
    Frame,
    encode::{control, nullary},
    parse_notification,
};

const EXAMPLE_OPCODE: u16 = 0x1234;
const EXAMPLE_PAYLOAD: &[u8] = &[0xde, 0xad, 0xbe, 0xef];
const EXPECTED_CONTROL_BYTES: &[u8] = &[0x01, 0x34, 0x12, 0xde, 0xad, 0xbe, 0xef];
const NULLARY_OPCODE: u16 = 0x006c;
const EXPECTED_NULLARY_BYTES: &[u8] = &[0x01, 0x6c, 0x00];

#[test]
fn control_frame_has_type_byte_and_little_endian_opcode_and_payload() {
    let bytes = control(EXAMPLE_OPCODE, EXAMPLE_PAYLOAD);
    assert_eq!(bytes.as_ref(), EXPECTED_CONTROL_BYTES);
}

#[test]
fn nullary_frame_has_only_the_3_byte_header() {
    let bytes = nullary(NULLARY_OPCODE);
    assert_eq!(bytes.as_ref(), EXPECTED_NULLARY_BYTES);
}

#[test]
fn control_encoded_frame_round_trips_through_parse_notification() {
    let encoded = control(EXAMPLE_OPCODE, EXAMPLE_PAYLOAD);
    let parsed = parse_notification(encoded.clone()).expect("parses");
    match parsed {
        Frame::Control { opcode, payload } => {
            assert_eq!(opcode, EXAMPLE_OPCODE);
            assert_eq!(payload.as_ref(), EXAMPLE_PAYLOAD);
        }
        other => panic!("expected Control, got {other:?}"),
    }
}
