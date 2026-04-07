//! Tests for [`plaud_proto::encode::device`] — device-level encoders.
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use plaud_proto::encode::device::{close_session, get_device_name, get_state, get_storage_stats, set_privacy};

const EXPECTED_GET_DEVICE_NAME: &[u8] = &[0x01, 0x6c, 0x00];
const EXPECTED_GET_STATE: &[u8] = &[0x01, 0x03, 0x00];
const EXPECTED_GET_STORAGE_STATS: &[u8] = &[0x01, 0x06, 0x00];
const EXPECTED_SET_PRIVACY_ON: &[u8] = &[0x01, 0x67, 0x00, 0x01];
const EXPECTED_SET_PRIVACY_OFF: &[u8] = &[0x01, 0x67, 0x00, 0x00];
const EXPECTED_CLOSE_SESSION: &[u8] = &[0x01, 0x6d, 0x00, 0x00];

#[test]
fn get_device_name_encodes_nullary_opcode_0x006c() {
    assert_eq!(get_device_name().as_ref(), EXPECTED_GET_DEVICE_NAME);
}

#[test]
fn get_state_encodes_nullary_opcode_0x0003() {
    assert_eq!(get_state().as_ref(), EXPECTED_GET_STATE);
}

#[test]
fn get_storage_stats_encodes_nullary_opcode_0x0006() {
    assert_eq!(get_storage_stats().as_ref(), EXPECTED_GET_STORAGE_STATS);
}

#[test]
fn set_privacy_on_writes_one_byte() {
    assert_eq!(set_privacy(true).as_ref(), EXPECTED_SET_PRIVACY_ON);
}

#[test]
fn set_privacy_off_writes_zero_byte() {
    assert_eq!(set_privacy(false).as_ref(), EXPECTED_SET_PRIVACY_OFF);
}

#[test]
fn close_session_matches_captured_last_write() {
    assert_eq!(close_session().as_ref(), EXPECTED_CLOSE_SESSION);
}
