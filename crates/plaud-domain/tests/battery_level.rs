//! Tests for [`plaud_domain::BatteryLevel`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{BatteryLevel, BatteryLevelError};

const EMPTY_PERCENT: u8 = 0;
const HALF_PERCENT: u8 = 50;
const FULL_PERCENT: u8 = 100;
const OVER_FULL_BY_ONE: u8 = 101;
const OVER_FULL_MAX_U8: u8 = 255;

#[test]
fn new_accepts_boundary_values() {
    assert!(BatteryLevel::new(EMPTY_PERCENT).is_ok());
    assert!(BatteryLevel::new(FULL_PERCENT).is_ok());
}

#[test]
fn new_rejects_values_above_hundred() {
    let err = BatteryLevel::new(OVER_FULL_BY_ONE).unwrap_err();
    assert!(matches!(err, BatteryLevelError::OutOfRange { .. }));
    let err = BatteryLevel::new(OVER_FULL_MAX_U8).unwrap_err();
    assert!(matches!(err, BatteryLevelError::OutOfRange { .. }));
}

#[test]
fn percent_returns_stored_value() {
    let level = BatteryLevel::new(HALF_PERCENT).expect("valid");
    assert_eq!(level.percent(), HALF_PERCENT);
}

#[test]
fn try_from_u8_matches_new() {
    let level: BatteryLevel = FULL_PERCENT.try_into().expect("valid");
    assert_eq!(level.percent(), FULL_PERCENT);
}

#[test]
fn try_from_u8_rejects_out_of_range() {
    let err: Result<BatteryLevel, _> = OVER_FULL_BY_ONE.try_into();
    assert!(err.is_err());
}

#[test]
fn constants_match_boundary_percentages() {
    assert_eq!(BatteryLevel::EMPTY.percent(), EMPTY_PERCENT);
    assert_eq!(BatteryLevel::FULL.percent(), FULL_PERCENT);
}

#[test]
fn display_appends_percent_sign() {
    let level = BatteryLevel::new(HALF_PERCENT).expect("valid");
    assert_eq!(level.to_string(), "50%");
}

#[test]
fn ordering_is_by_percent_ascending() {
    let low = BatteryLevel::new(EMPTY_PERCENT).expect("valid");
    let mid = BatteryLevel::new(HALF_PERCENT).expect("valid");
    let high = BatteryLevel::new(FULL_PERCENT).expect("valid");
    assert!(low < mid);
    assert!(mid < high);
}
