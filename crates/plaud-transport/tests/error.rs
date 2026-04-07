//! Tests for [`plaud_transport::Error`] and the `Result` alias.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use std::io;

use plaud_transport::Error;

const NOT_FOUND_WHAT: &str = "recording 1775393534";
const TIMEOUT_SECONDS: u64 = 30;
const AUTH_REJECTED_STATUS: u8 = 0x01;
const UNSUPPORTED_CAPABILITY: &str = "start_recording";
const PROTOCOL_DETAIL: &str = "bad magic byte";
const TRANSPORT_DETAIL: &str = "ble adapter missing";

#[test]
fn not_found_display_includes_the_query_string() {
    let err = Error::NotFound(NOT_FOUND_WHAT.to_owned());
    assert!(err.to_string().contains(NOT_FOUND_WHAT));
}

#[test]
fn auth_required_has_stable_message() {
    let err = Error::AuthRequired;
    assert!(err.to_string().contains("authentication required"));
}

#[test]
fn auth_rejected_display_contains_hex_status_byte() {
    let err = Error::AuthRejected {
        status: AUTH_REJECTED_STATUS,
    };
    let rendered = err.to_string();
    assert!(rendered.contains("0x01"), "expected `0x01` in {rendered}");
}

#[test]
fn timeout_display_contains_the_duration() {
    let err = Error::Timeout { seconds: TIMEOUT_SECONDS };
    assert!(err.to_string().contains(&TIMEOUT_SECONDS.to_string()));
}

#[test]
fn io_error_is_converted_via_from() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn protocol_error_display_includes_detail() {
    let err = Error::Protocol(PROTOCOL_DETAIL.to_owned());
    assert!(err.to_string().contains(PROTOCOL_DETAIL));
}

#[test]
fn transport_error_display_includes_detail() {
    let err = Error::Transport(TRANSPORT_DETAIL.to_owned());
    assert!(err.to_string().contains(TRANSPORT_DETAIL));
}

#[test]
fn unsupported_display_includes_capability() {
    let err = Error::Unsupported {
        capability: UNSUPPORTED_CAPABILITY,
    };
    assert!(err.to_string().contains(UNSUPPORTED_CAPABILITY));
}

#[test]
fn error_implements_std_error() {
    fn assert_is_error<E: std::error::Error>(_: &E) {}
    let err = Error::AuthRequired;
    assert_is_error(&err);
}
