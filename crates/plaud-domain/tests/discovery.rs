//! Tests for [`plaud_domain::DeviceCandidate`] and
//! [`plaud_domain::TransportHint`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{DeviceCandidate, TransportHint};

const LOCAL_NAME: &str = "PLAUD_NOTE";
const NORDIC_MANUFACTURER_ID: u16 = 0x0059;
const RSSI_DBM: i16 = -67;

#[test]
fn transport_hint_name_is_stable() {
    assert_eq!(TransportHint::Ble.name(), "ble");
    assert_eq!(TransportHint::Usb.name(), "usb");
    assert_eq!(TransportHint::Wifi.name(), "wifi");
}

#[test]
fn device_candidate_stores_every_field() {
    let cand = DeviceCandidate::new(LOCAL_NAME.to_owned(), NORDIC_MANUFACTURER_ID, Some(RSSI_DBM), TransportHint::Ble);
    assert_eq!(cand.local_name, LOCAL_NAME);
    assert_eq!(cand.manufacturer_id, NORDIC_MANUFACTURER_ID);
    assert_eq!(cand.rssi_dbm, Some(RSSI_DBM));
    assert_eq!(cand.transport_hint, TransportHint::Ble);
}

#[test]
fn device_candidate_supports_transport_without_rssi() {
    let cand = DeviceCandidate::new(LOCAL_NAME.to_owned(), 0, None, TransportHint::Usb);
    assert!(cand.rssi_dbm.is_none());
    assert_eq!(cand.transport_hint, TransportHint::Usb);
}
