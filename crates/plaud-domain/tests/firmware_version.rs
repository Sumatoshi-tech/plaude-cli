//! Tests for [`plaud_domain::FirmwareVersion`].
//!
//! Fixture sourced from
//! `specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{FirmwareVersion, FirmwareVersionError};

const REAL_MODEL_TXT_LINE: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024";
const EXPECTED_BUILD: &str = "0095";
const EXPECTED_STAMP: &str = "00:47:14 Feb 28 2024";

const LINE_WITHOUT_STAMP: &str = "PLAUD NOTE V0096";
const LINE_WITHOUT_V_TOKEN: &str = "PLAUD NOTE 0095";
const LINE_WITH_EMPTY_BUILD: &str = "PLAUD NOTE V@00:47:14";

#[test]
fn parses_the_real_v0095_line_byte_for_byte() {
    let fw = FirmwareVersion::parse_model_txt_line(REAL_MODEL_TXT_LINE).expect("parses");
    assert_eq!(fw.build(), EXPECTED_BUILD);
    assert_eq!(fw.build_stamp(), Some(EXPECTED_STAMP));
}

#[test]
fn parses_a_line_without_a_build_stamp() {
    let fw = FirmwareVersion::parse_model_txt_line(LINE_WITHOUT_STAMP).expect("parses");
    assert_eq!(fw.build(), "0096");
    assert_eq!(fw.build_stamp(), None);
}

#[test]
fn rejects_a_line_with_no_v_prefixed_token() {
    let err = FirmwareVersion::parse_model_txt_line(LINE_WITHOUT_V_TOKEN).unwrap_err();
    assert_eq!(err, FirmwareVersionError::MissingBuildToken);
}

#[test]
fn rejects_a_line_with_empty_build() {
    let err = FirmwareVersion::parse_model_txt_line(LINE_WITH_EMPTY_BUILD).unwrap_err();
    assert_eq!(err, FirmwareVersionError::EmptyBuild);
}

#[test]
fn display_includes_build_and_stamp_when_both_present() {
    let fw = FirmwareVersion::parse_model_txt_line(REAL_MODEL_TXT_LINE).expect("parses");
    let rendered = fw.to_string();
    assert!(rendered.contains(EXPECTED_BUILD));
    assert!(rendered.contains(EXPECTED_STAMP));
}

#[test]
fn display_omits_stamp_when_absent() {
    let fw = FirmwareVersion::parse_model_txt_line(LINE_WITHOUT_STAMP).expect("parses");
    assert_eq!(fw.to_string(), "0096");
}
