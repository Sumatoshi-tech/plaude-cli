//! Tests for [`plaud_domain::CommonSettingKey`], [`Setting`] and
//! [`SettingValue`].
//!
//! The codes under test mirror
//! `Constants$CommonSettings$SettingType.java` from the tinnotech SDK.
//! Every single variant must round-trip through `code()`/`from_code()`
//! so we do not reproduce the SDK's own bug where several codes were
//! missing from the lookup table.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{CommonSettingKey, Setting, SettingValue, SettingValueParseError, UnknownSettingCode, UnknownSettingName};

const EXPECTED_VARIANT_COUNT: usize = 20;
const UNKNOWN_CODE_ZERO: u8 = 0;
const UNKNOWN_CODE_MIDDLE_GAP: u8 = 5;
const UNKNOWN_CODE_HIGH: u8 = 33;
const UNKNOWN_CODE_MAX: u8 = 255;

#[test]
fn all_returns_exactly_twenty_variants() {
    assert_eq!(CommonSettingKey::all().len(), EXPECTED_VARIANT_COUNT);
}

#[test]
fn code_and_from_code_round_trip_for_every_variant() {
    for &variant in CommonSettingKey::all() {
        let code = variant.code();
        let decoded = CommonSettingKey::from_code(code).expect("round-trips");
        assert_eq!(decoded, variant, "variant {variant:?} did not round-trip through code {code}");
    }
}

#[test]
fn name_is_unique_across_variants() {
    let names: Vec<_> = CommonSettingKey::all().iter().map(|k| k.name()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len(), "duplicate names in CommonSettingKey::all()");
}

#[test]
fn from_code_rejects_unknown_codes() {
    for code in [UNKNOWN_CODE_ZERO, UNKNOWN_CODE_MIDDLE_GAP, UNKNOWN_CODE_HIGH, UNKNOWN_CODE_MAX] {
        let err = CommonSettingKey::from_code(code).unwrap_err();
        assert_eq!(err, UnknownSettingCode { code });
    }
}

#[test]
fn specific_sdk_codes_map_to_documented_variants() {
    // Spot-check a handful of codes against the SDK enum table in
    // specs/re/apk-notes/3.14.0-620/ble-protocol.md.
    assert_eq!(CommonSettingKey::from_code(1).unwrap(), CommonSettingKey::BackLightTime);
    assert_eq!(CommonSettingKey::from_code(15).unwrap(), CommonSettingKey::EnableVad);
    assert_eq!(CommonSettingKey::from_code(17).unwrap(), CommonSettingKey::RecMode);
    assert_eq!(CommonSettingKey::from_code(26).unwrap(), CommonSettingKey::AutoSync);
    assert_eq!(CommonSettingKey::from_code(27).unwrap(), CommonSettingKey::FindMy);
    assert_eq!(CommonSettingKey::from_code(32).unwrap(), CommonSettingKey::BatteryMode);
}

#[test]
fn setting_pair_stores_key_and_value() {
    let pair = Setting::new(CommonSettingKey::EnableVad, SettingValue::Bool(true));
    assert_eq!(pair.key, CommonSettingKey::EnableVad);
    assert_eq!(pair.value, SettingValue::Bool(true));
}

#[test]
fn setting_value_variants_are_distinct() {
    assert_ne!(SettingValue::Bool(false), SettingValue::U8(0));
    assert_ne!(SettingValue::U8(1), SettingValue::U32(1));
}

#[test]
fn from_name_round_trips_for_every_variant() {
    for &variant in CommonSettingKey::all() {
        let name = variant.name();
        let decoded = CommonSettingKey::from_name(name).expect("round-trips");
        assert_eq!(decoded, variant, "variant {variant:?} did not round-trip through name {name:?}");
    }
}

#[test]
fn from_name_rejects_unknown_names() {
    let err = CommonSettingKey::from_name("no-such-setting").unwrap_err();
    assert_eq!(
        err,
        UnknownSettingName {
            name: "no-such-setting".to_owned()
        }
    );
}

#[test]
fn setting_value_display_bool() {
    assert_eq!(format!("{}", SettingValue::Bool(true)), "true");
    assert_eq!(format!("{}", SettingValue::Bool(false)), "false");
}

#[test]
fn setting_value_display_u8() {
    assert_eq!(format!("{}", SettingValue::U8(42)), "42");
}

#[test]
fn setting_value_display_u32() {
    assert_eq!(format!("{}", SettingValue::U32(100_000)), "100000");
}

#[test]
fn setting_value_parse_bool() {
    assert_eq!(SettingValue::parse("true").unwrap(), SettingValue::Bool(true));
    assert_eq!(SettingValue::parse("false").unwrap(), SettingValue::Bool(false));
}

#[test]
fn setting_value_parse_u8() {
    assert_eq!(SettingValue::parse("0").unwrap(), SettingValue::U8(0));
    assert_eq!(SettingValue::parse("255").unwrap(), SettingValue::U8(255));
}

#[test]
fn setting_value_parse_u32() {
    assert_eq!(SettingValue::parse("256").unwrap(), SettingValue::U32(256));
    assert_eq!(SettingValue::parse("4294967295").unwrap(), SettingValue::U32(u32::MAX));
}

#[test]
fn setting_value_parse_rejects_garbage() {
    let err = SettingValue::parse("not-a-value").unwrap_err();
    assert_eq!(
        err,
        SettingValueParseError {
            input: "not-a-value".to_owned()
        }
    );
}
