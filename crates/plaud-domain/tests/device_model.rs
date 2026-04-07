//! Tests for [`plaud_domain::DeviceModel`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::DeviceModel;

const UNKNOWN_NAME: &str = "PlaudFuture";

#[test]
fn name_is_stable_for_known_variants() {
    assert_eq!(DeviceModel::Note.name(), "Plaud Note");
    assert_eq!(DeviceModel::NotePin.name(), "Plaud NotePin");
    assert_eq!(DeviceModel::NotePinS.name(), "Plaud NotePin S");
    assert_eq!(DeviceModel::NotePro.name(), "Plaud Note Pro");
}

#[test]
fn unknown_variant_forwards_its_raw_name() {
    let model = DeviceModel::Unknown(UNKNOWN_NAME.to_owned());
    assert_eq!(model.name(), UNKNOWN_NAME);
}

#[test]
fn equality_distinguishes_unknown_payloads() {
    let a = DeviceModel::Unknown("A".to_owned());
    let b = DeviceModel::Unknown("B".to_owned());
    assert_ne!(a, b);
}
