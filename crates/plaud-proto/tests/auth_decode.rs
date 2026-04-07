//! Tests for [`plaud_proto::decode::auth_response`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use bytes::Bytes;
use plaud_proto::{AuthStatus, DecodeError, Frame, auth_response, parse_notification};

const ACCEPTED_RESPONSE: &[u8] = &[
    0x01, 0x01, 0x00, // control header + opcode 0x0001 LE
    0x00, // status = accepted
    0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const REJECTED_RESPONSE: &[u8] = &[
    0x01, 0x01, 0x00, //
    0x01, // status = rejected
    0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const UNKNOWN_STATUS_BYTE: u8 = 0x05;
const UNKNOWN_STATUS_RESPONSE: &[u8] = &[0x01, 0x01, 0x00, UNKNOWN_STATUS_BYTE];
const EMPTY_AUTH_PAYLOAD: &[u8] = &[0x01, 0x01, 0x00];
const NON_AUTH_CONTROL: &[u8] = &[0x01, 0x6c, 0x00];

fn parse(bytes: &'static [u8]) -> Frame {
    parse_notification(Bytes::from_static(bytes)).expect("control frame parses")
}

#[test]
fn auth_response_accepted_for_status_byte_zero() {
    let frame = parse(ACCEPTED_RESPONSE);
    assert_eq!(auth_response(&frame), Ok(AuthStatus::Accepted));
}

#[test]
fn auth_response_rejected_for_status_byte_one() {
    let frame = parse(REJECTED_RESPONSE);
    assert_eq!(auth_response(&frame), Ok(AuthStatus::Rejected));
}

#[test]
fn auth_response_rejects_unknown_status_byte() {
    let frame = parse(UNKNOWN_STATUS_RESPONSE);
    assert_eq!(
        auth_response(&frame),
        Err(DecodeError::UnknownAuthStatus { byte: UNKNOWN_STATUS_BYTE })
    );
}

#[test]
fn auth_response_rejects_empty_payload_with_too_short() {
    let frame = parse(EMPTY_AUTH_PAYLOAD);
    let err = auth_response(&frame).unwrap_err();
    assert!(matches!(err, DecodeError::TooShort { .. }));
}

#[test]
fn auth_response_rejects_non_auth_opcode() {
    let frame = parse(NON_AUTH_CONTROL);
    assert_eq!(auth_response(&frame), Err(DecodeError::NotAuthResponse));
}

#[test]
fn auth_response_rejects_non_control_frame() {
    let bulk = Bytes::from_static(&[
        0x02, 0x00, // bulk magic
        0x58, 0x66, 0xd2, 0x69, // file id
        0x00, 0x00, 0x00, 0x00, // offset
    ]);
    let frame = parse_notification(bulk).expect("bulk parses");
    assert_eq!(auth_response(&frame), Err(DecodeError::NotAuthResponse));
}
