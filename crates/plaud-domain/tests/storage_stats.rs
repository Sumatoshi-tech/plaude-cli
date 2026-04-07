//! Tests for [`plaud_domain::StorageStats`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{StorageStats, StorageStatsError};

const TOTAL_BYTES: u64 = 1_000;
const USED_BYTES: u64 = 400;
const RECORDING_COUNT: u32 = 7;
const EXPECTED_FREE: u64 = TOTAL_BYTES - USED_BYTES;
const EXPECTED_RATIO: f64 = 0.4;
const RATIO_EPSILON: f64 = 1e-9;

#[test]
fn new_accepts_used_below_total() {
    let s = StorageStats::new(TOTAL_BYTES, USED_BYTES, RECORDING_COUNT).expect("valid");
    assert_eq!(s.total_bytes(), TOTAL_BYTES);
    assert_eq!(s.used_bytes(), USED_BYTES);
    assert_eq!(s.recording_count(), RECORDING_COUNT);
}

#[test]
fn new_accepts_used_equal_to_total() {
    assert!(StorageStats::new(TOTAL_BYTES, TOTAL_BYTES, RECORDING_COUNT).is_ok());
}

#[test]
fn new_rejects_used_greater_than_total() {
    let err = StorageStats::new(TOTAL_BYTES, TOTAL_BYTES + 1, RECORDING_COUNT).unwrap_err();
    assert!(matches!(err, StorageStatsError::UsedExceedsTotal { .. }));
}

#[test]
fn free_bytes_is_total_minus_used() {
    let s = StorageStats::new(TOTAL_BYTES, USED_BYTES, RECORDING_COUNT).expect("valid");
    assert_eq!(s.free_bytes(), EXPECTED_FREE);
}

#[test]
fn used_ratio_is_used_over_total() {
    let s = StorageStats::new(TOTAL_BYTES, USED_BYTES, RECORDING_COUNT).expect("valid");
    assert!((s.used_ratio() - EXPECTED_RATIO).abs() < RATIO_EPSILON);
}

#[test]
fn used_ratio_of_empty_device_is_zero_not_nan() {
    let s = StorageStats::new(0, 0, 0).expect("valid");
    assert!(s.used_ratio() == 0.0);
}

#[test]
fn display_shows_used_total_and_count() {
    let s = StorageStats::new(TOTAL_BYTES, USED_BYTES, RECORDING_COUNT).expect("valid");
    let rendered = s.to_string();
    assert!(rendered.contains("400"));
    assert!(rendered.contains("1000"));
    assert!(rendered.contains("7"));
}
