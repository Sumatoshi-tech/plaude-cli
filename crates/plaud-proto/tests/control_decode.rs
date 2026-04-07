//! Integration tests for [`plaud_proto::parse_notification`] control path.
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use bytes::Bytes;
use plaud_proto::{DecodeError, Frame, parse_notification};

const GET_DEVICE_NAME_REQUEST: &[u8] = &[0x01, 0x6c, 0x00];
const GET_DEVICE_NAME_RESPONSE: &[u8] = &[
    0x01, 0x6c, 0x00, b'P', b'L', b'A', b'U', b'D', b'_', b'N', b'O', b'T', b'E', 0x00, 0x00, 0x00, 0x00,
];
const CONTROL_PAYLOAD_OFFSET: usize = 3;
const EXPECTED_OPCODE_GET_DEVICE_NAME: u16 = 0x006c;

#[test]
fn parse_empty_notification_returns_empty_error() {
    let err = parse_notification(Bytes::new()).unwrap_err();
    assert_eq!(err, DecodeError::Empty);
}

#[test]
fn parse_notification_below_control_header_length_returns_too_short() {
    let short = Bytes::from_static(&[0x01, 0x01]); // type + half of opcode
    let err = parse_notification(short).unwrap_err();
    assert!(matches!(err, DecodeError::TooShort { .. }));
}

#[test]
fn parse_unknown_frame_type_returns_unknown_frame_type_error() {
    // `0x07` is not used by any frame type family we know about.
    let bogus = Bytes::from_static(&[0x07, 0x00, 0x00]);
    let err = parse_notification(bogus).unwrap_err();
    assert_eq!(err, DecodeError::UnknownFrameType { byte: 0x07 });
}

#[test]
fn parse_nullary_get_device_name_request() {
    let frame = parse_notification(Bytes::from_static(GET_DEVICE_NAME_REQUEST)).expect("parses");
    match frame {
        Frame::Control { opcode, payload } => {
            assert_eq!(opcode, EXPECTED_OPCODE_GET_DEVICE_NAME);
            assert!(payload.is_empty());
        }
        other => panic!("expected Control, got {other:?}"),
    }
}

#[test]
fn parse_get_device_name_response_preserves_payload() {
    let bytes = Bytes::from_static(GET_DEVICE_NAME_RESPONSE);
    let frame = parse_notification(bytes.clone()).expect("parses");
    match frame {
        Frame::Control { opcode, payload } => {
            assert_eq!(opcode, EXPECTED_OPCODE_GET_DEVICE_NAME);
            assert_eq!(payload.as_ref(), &GET_DEVICE_NAME_RESPONSE[CONTROL_PAYLOAD_OFFSET..]);
        }
        other => panic!("expected Control, got {other:?}"),
    }
}
