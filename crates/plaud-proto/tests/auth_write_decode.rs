//! Tests for [`plaud_proto::decode::parse_auth_write`].
//!
//! Exercises the phone→device direction of the auth handshake: the
//! inverse of [`plaud_proto::encode::auth::authenticate`], used by the
//! M8 `plaude-cli auth bootstrap` fake peripheral to capture tokens
//! written by a connecting phone app.
//!
//! Journey: specs/plaude-cli-v1/journeys/M08-auth-bootstrap-peripheral.md

use plaud_domain::AuthToken;
use plaud_proto::{DecodeError, decode::parse_auth_write, encode::auth::authenticate};

const SAMPLE_LONG_TOKEN: &str = "0123456789abcdef0123456789abcdef";
const SAMPLE_SHORT_TOKEN: &str = "0123456789abcdef";
const MISMATCHED_PREFIX_FRAME: &[u8] = &[
    0xAA, 0xBB, 0xCC, 0x02, 0x00, 0x00, b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

#[test]
fn parse_auth_write_round_trips_a_long_token() {
    let token = AuthToken::new(SAMPLE_LONG_TOKEN).expect("valid");
    let wire = authenticate(&token);
    let parsed = parse_auth_write(wire.as_ref()).expect("parse");
    assert_eq!(parsed.as_str(), SAMPLE_LONG_TOKEN);
}

#[test]
fn parse_auth_write_round_trips_a_short_token() {
    let token = AuthToken::new(SAMPLE_SHORT_TOKEN).expect("valid");
    let wire = authenticate(&token);
    let parsed = parse_auth_write(wire.as_ref()).expect("parse");
    assert_eq!(parsed.as_str(), SAMPLE_SHORT_TOKEN);
}

#[test]
fn parse_auth_write_rejects_a_mismatched_prefix() {
    let err = parse_auth_write(MISMATCHED_PREFIX_FRAME).unwrap_err();
    assert!(matches!(err, DecodeError::InvalidAuthPrefix));
}

#[test]
fn parse_auth_write_rejects_an_input_shorter_than_the_prefix() {
    let err = parse_auth_write(&[0x01, 0x01]).unwrap_err();
    assert!(matches!(err, DecodeError::TooShort { .. }));
}

#[test]
fn parse_auth_write_rejects_a_non_hex_token() {
    // Valid prefix but the trailing token bytes are not ASCII hex.
    let bytes: Vec<u8> = b"\x01\x01\x00\x02\x00\x00zzzzzzzzzzzzzzzz".to_vec();
    let err = parse_auth_write(&bytes).unwrap_err();
    assert!(matches!(err, DecodeError::InvalidAuthToken { .. }));
}
