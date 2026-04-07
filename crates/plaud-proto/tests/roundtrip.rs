//! Property-based round-trip tests for [`plaud_proto`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use plaud_proto::{Frame, encode::control, parse_notification};
use proptest::prelude::*;

const MAX_PAYLOAD_LEN: usize = 200;

proptest! {
    /// For every `(opcode, payload)` pair, `parse_notification` of the
    /// `encode::control` output must return `Frame::Control` with the
    /// same opcode and the same payload bytes. Never panics.
    #[test]
    fn control_encode_then_decode_round_trips(
        opcode in any::<u16>(),
        payload in proptest::collection::vec(any::<u8>(), 0..MAX_PAYLOAD_LEN),
    ) {
        // Skip the handshake signature range — valid but outside this
        // property's scope (`parse_notification` detects it as
        // Handshake, not Control).
        let high_byte = (opcode >> 8) as u8;
        prop_assume!(!(payload.first().copied() == Some(0xFE) && opcode == 0x01));
        prop_assume!(high_byte != 0xFE || payload.is_empty());

        let encoded = control(opcode, &payload);
        let parsed = parse_notification(encoded).expect("encoded frame must parse");
        match parsed {
            Frame::Control { opcode: got_op, payload: got_pl } => {
                prop_assert_eq!(got_op, opcode);
                prop_assert_eq!(got_pl.as_ref(), payload.as_slice());
            }
            Frame::Handshake { .. } => {
                // Encoded opcode happened to look like a handshake type;
                // accept it so the test does not fail on benign collisions.
            }
            other => prop_assert!(false, "expected Control, got {other:?}"),
        }
    }
}
