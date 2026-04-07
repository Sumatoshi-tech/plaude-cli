//! Tests for [`plaud_domain::RecordingId`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{RecordingId, RecordingIdError};
use proptest::prelude::*;

const REAL_V0095_BASENAME: &str = "1775393534";
const MIN_VALID: &str = "100000000"; // 9 digits
const MAX_VALID_I64: &str = "9223372036854775807"; // 19 digits, i64::MAX
const TOO_SHORT_8: &str = "12345678";
const TOO_LONG_20: &str = "12345678901234567890";
const OUT_OF_RANGE_19: &str = "9999999999999999999"; // 19 digits but > i64::MAX
const NON_DIGIT_INPUT: &str = "17753x3534";
const LEADING_ZERO_INPUT: &str = "0001234567";

#[test]
fn new_accepts_the_real_v0095_basename() {
    let id = RecordingId::new(REAL_V0095_BASENAME).expect("v0095 basename is valid");
    assert_eq!(id.as_str(), REAL_V0095_BASENAME);
}

#[test]
fn new_accepts_minimum_length() {
    assert!(RecordingId::new(MIN_VALID).is_ok());
}

#[test]
fn new_accepts_maximum_length_at_i64_max() {
    assert!(RecordingId::new(MAX_VALID_I64).is_ok());
}

#[test]
fn new_accepts_leading_zeros() {
    let id = RecordingId::new(LEADING_ZERO_INPUT).expect("leading zeros valid");
    assert_eq!(id.as_unix_seconds(), 1_234_567);
}

#[test]
fn new_rejects_empty_with_empty_variant() {
    assert_eq!(RecordingId::new(""), Err(RecordingIdError::Empty));
}

#[test]
fn new_rejects_too_short_with_invalid_length() {
    let err = RecordingId::new(TOO_SHORT_8).unwrap_err();
    match err {
        RecordingIdError::InvalidLength { got, .. } => assert_eq!(got, TOO_SHORT_8.len()),
        other => panic!("expected InvalidLength, got {other:?}"),
    }
}

#[test]
fn new_rejects_too_long_with_invalid_length() {
    let err = RecordingId::new(TOO_LONG_20).unwrap_err();
    assert!(matches!(err, RecordingIdError::InvalidLength { .. }));
}

#[test]
fn new_rejects_non_digit_with_non_digit_variant() {
    assert_eq!(RecordingId::new(NON_DIGIT_INPUT), Err(RecordingIdError::NonDigit));
}

#[test]
fn new_rejects_19_digit_input_greater_than_i64_max() {
    let err = RecordingId::new(OUT_OF_RANGE_19).unwrap_err();
    assert!(matches!(err, RecordingIdError::OutOfRange { .. }));
}

#[test]
fn as_unix_seconds_parses_real_basename() {
    let id = RecordingId::new(REAL_V0095_BASENAME).expect("valid");
    assert_eq!(id.as_unix_seconds(), 1_775_393_534_i64);
}

#[test]
fn display_matches_raw_input() {
    let id = RecordingId::new(REAL_V0095_BASENAME).expect("valid");
    assert_eq!(id.to_string(), REAL_V0095_BASENAME);
}

#[test]
fn from_str_delegates_to_new() {
    let parsed: RecordingId = REAL_V0095_BASENAME.parse().expect("valid");
    assert_eq!(parsed.as_str(), REAL_V0095_BASENAME);
}

#[test]
fn from_str_propagates_errors() {
    let err: Result<RecordingId, _> = "abc".parse();
    assert!(err.is_err());
}

#[test]
fn debug_contains_type_name_and_raw_value() {
    let id = RecordingId::new(REAL_V0095_BASENAME).expect("valid");
    let debug = format!("{id:?}");
    assert!(debug.contains("RecordingId"));
    assert!(debug.contains(REAL_V0095_BASENAME));
}

#[test]
fn clone_and_equality_behave() {
    let a = RecordingId::new(REAL_V0095_BASENAME).expect("valid");
    let b = a.clone();
    assert_eq!(a, b);
}

proptest! {
    /// Roundtrip property: any digit string that the constructor
    /// accepts must `Display` as exactly the same string.
    #[test]
    fn display_roundtrips_for_any_accepted_input(
        s in "[0-9]{9,19}"
    ) {
        if let Ok(id) = RecordingId::new(s.clone()) {
            prop_assert_eq!(id.to_string(), s);
        }
    }
}
