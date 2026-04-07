//! Tests for [`plaud_domain::DeviceSerial`].
//!
//! The single most important property under test here is the
//! non-leaking `Debug` impl — forgetting it would undo the CLI's
//! forensic-sanitisation guarantees.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{DeviceSerial, DeviceSerialError};

const REAL_LENGTH_PLACEHOLDER: &str = "123456789012345678"; // 18 digits, not the user's real serial
const MIN_LENGTH: &str = "12345678"; // 8 digits
const TOO_SHORT_7: &str = "1234567";
const TOO_LONG_33: &str = "123456789012345678901234567890123";
const NON_DIGIT_INPUT: &str = "12345678X";
const EXPECTED_REDACTION_MARKER: &str = "<redacted>";
const MIN_RUN_OF_DIGITS_THAT_WOULD_LEAK_SERIAL: usize = 8;

fn contains_long_digit_run(haystack: &str, min_run: usize) -> bool {
    let mut run = 0usize;
    for byte in haystack.as_bytes() {
        if byte.is_ascii_digit() {
            run += 1;
            if run >= min_run {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

#[test]
fn new_accepts_an_18_digit_string() {
    let serial = DeviceSerial::new(REAL_LENGTH_PLACEHOLDER).expect("18 digits is valid");
    assert_eq!(serial.reveal(), REAL_LENGTH_PLACEHOLDER);
    assert_eq!(serial.len(), REAL_LENGTH_PLACEHOLDER.len());
}

#[test]
fn new_accepts_minimum_length() {
    assert!(DeviceSerial::new(MIN_LENGTH).is_ok());
}

#[test]
fn new_rejects_empty_with_empty_variant() {
    assert_eq!(DeviceSerial::new(""), Err(DeviceSerialError::Empty));
}

#[test]
fn new_rejects_too_short() {
    let err = DeviceSerial::new(TOO_SHORT_7).unwrap_err();
    assert!(matches!(err, DeviceSerialError::InvalidLength { .. }));
}

#[test]
fn new_rejects_too_long() {
    let err = DeviceSerial::new(TOO_LONG_33).unwrap_err();
    assert!(matches!(err, DeviceSerialError::InvalidLength { .. }));
}

#[test]
fn new_rejects_non_digit() {
    assert_eq!(DeviceSerial::new(NON_DIGIT_INPUT), Err(DeviceSerialError::NonDigit));
}

#[test]
fn debug_does_not_contain_any_digit_run_that_would_leak_the_serial() {
    let serial = DeviceSerial::new(REAL_LENGTH_PLACEHOLDER).expect("valid");
    let debug = format!("{serial:?}");
    assert!(
        !contains_long_digit_run(&debug, MIN_RUN_OF_DIGITS_THAT_WOULD_LEAK_SERIAL),
        "Debug output leaks serial: {debug}"
    );
}

#[test]
fn debug_marks_value_as_redacted() {
    let serial = DeviceSerial::new(REAL_LENGTH_PLACEHOLDER).expect("valid");
    let debug = format!("{serial:?}");
    assert!(
        debug.contains(EXPECTED_REDACTION_MARKER),
        "Debug output missing redaction marker: {debug}"
    );
    assert!(debug.contains("DeviceSerial"));
}

#[test]
fn is_empty_is_false_for_valid_serial() {
    let serial = DeviceSerial::new(REAL_LENGTH_PLACEHOLDER).expect("valid");
    assert!(!serial.is_empty());
}

#[test]
fn clone_and_equality_are_consistent() {
    let a = DeviceSerial::new(REAL_LENGTH_PLACEHOLDER).expect("valid");
    let b = a.clone();
    assert_eq!(a, b);
}
