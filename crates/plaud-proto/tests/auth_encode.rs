//! Tests for [`plaud_proto::encode::auth::authenticate`].
//!
//! The V0095 wire layout is pinned byte-for-byte against the live-tested
//! auth-replay script in
//! `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use plaud_domain::AuthToken;
use plaud_proto::encode::auth::authenticate;

const PLACEHOLDER_LONG_TOKEN: &str = "0123456789abcdef0123456789abcdef";
const PLACEHOLDER_SHORT_TOKEN: &str = "0123456789abcdef";

const EXPECTED_LONG_FRAME: &[u8] = &[
    // control header + opcode 0x0001 LE
    0x01, 0x01, 0x00, //
    // length constant `packInt(2)` + single version byte
    0x02, 0x00, 0x00, //
    // 32 ASCII chars (the placeholder long token)
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f', //
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

const EXPECTED_SHORT_FRAME: &[u8] = &[
    0x01, 0x01, 0x00, //
    0x02, 0x00, 0x00, //
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

const EXPECTED_LONG_FRAME_BYTE_COUNT: usize = 38;
const EXPECTED_SHORT_FRAME_BYTE_COUNT: usize = 22;

#[test]
fn authenticate_emits_the_v0095_wire_layout_for_a_32_char_token() {
    let token = AuthToken::new(PLACEHOLDER_LONG_TOKEN).expect("valid");
    let bytes = authenticate(&token);
    assert_eq!(bytes.as_ref(), EXPECTED_LONG_FRAME);
    assert_eq!(bytes.len(), EXPECTED_LONG_FRAME_BYTE_COUNT);
}

#[test]
fn authenticate_emits_a_22_byte_frame_for_a_16_char_token() {
    let token = AuthToken::new(PLACEHOLDER_SHORT_TOKEN).expect("valid");
    let bytes = authenticate(&token);
    assert_eq!(bytes.as_ref(), EXPECTED_SHORT_FRAME);
    assert_eq!(bytes.len(), EXPECTED_SHORT_FRAME_BYTE_COUNT);
}

#[test]
fn authenticate_frame_starts_with_the_control_header_and_auth_opcode() {
    let token = AuthToken::new(PLACEHOLDER_LONG_TOKEN).expect("valid");
    let bytes = authenticate(&token);
    // Control frame type byte, then opcode 0x0001 little-endian.
    assert_eq!(&bytes[..3], &[0x01, 0x01, 0x00]);
}
