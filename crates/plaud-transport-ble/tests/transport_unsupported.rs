//! Tests that verify `BleTransport` method behaviour on an
//! unauthenticated loopback session.
//!
//! Methods wired to real opcodes return `AuthRequired` when the
//! session has not been authenticated. Methods still stubbed return
//! `Unsupported` with a capability string.
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use std::sync::Arc;

use plaud_domain::{BatteryLevel, CommonSettingKey, RecordingId, SettingValue};
use plaud_transport::{Error, Transport};
use plaud_transport_ble::{BleChannel, BleSession, BleTransport, FixedBatteryReader};
use tokio::sync::Mutex;

const BATTERY_PERCENT: u8 = 50;
const SAMPLE_BASENAME: &str = "1775393534";

fn build_transport() -> BleTransport {
    let (channel, _peer) = BleChannel::loopback_pair();
    let session = Arc::new(Mutex::new(BleSession::new(channel)));
    let battery = Arc::new(FixedBatteryReader::new(BatteryLevel::new(BATTERY_PERCENT).expect("valid")));
    BleTransport::from_parts(session, battery)
}

fn assert_unsupported(err: &Error) {
    assert!(matches!(err, Error::Unsupported { .. }), "expected Unsupported, got {err:?}");
}

fn assert_auth_required(err: &Error) {
    assert!(matches!(err, Error::AuthRequired), "expected AuthRequired, got {err:?}");
}

// --- Wired methods: return AuthRequired on unauthenticated session ---

#[tokio::test]
async fn device_info_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.device_info().await.unwrap_err());
}

#[tokio::test]
async fn storage_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.storage().await.unwrap_err());
}

#[tokio::test]
async fn list_recordings_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.list_recordings().await.unwrap_err());
}

#[tokio::test]
async fn read_recording_requires_auth() {
    let t = build_transport();
    let id = RecordingId::new(SAMPLE_BASENAME).expect("valid");
    assert_auth_required(&t.read_recording(&id).await.unwrap_err());
}

#[tokio::test]
async fn read_setting_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.read_setting(CommonSettingKey::MicGain).await.unwrap_err());
}

#[tokio::test]
async fn write_setting_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.write_setting(CommonSettingKey::MicGain, SettingValue::U8(5)).await.unwrap_err());
}

#[tokio::test]
async fn set_privacy_requires_auth() {
    let t = build_transport();
    assert_auth_required(&t.set_privacy(true).await.unwrap_err());
}

// --- Not-found: ASR sidecar download not yet supported ---

#[tokio::test]
async fn read_recording_asr_returns_not_found() {
    let t = build_transport();
    let id = RecordingId::new(SAMPLE_BASENAME).expect("valid");
    let err = t.read_recording_asr(&id).await.unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "expected NotFound, got {err:?}");
}

// --- Still-unsupported methods ---

#[tokio::test]
async fn delete_recording_is_unsupported() {
    let t = build_transport();
    let id = RecordingId::new(SAMPLE_BASENAME).expect("valid");
    assert_unsupported(&t.delete_recording(&id).await.unwrap_err());
}

#[tokio::test]
async fn recording_control_is_unsupported() {
    let t = build_transport();
    assert_unsupported(&t.start_recording().await.unwrap_err());
    assert_unsupported(&t.stop_recording().await.unwrap_err());
    assert_unsupported(&t.pause_recording().await.unwrap_err());
    assert_unsupported(&t.resume_recording().await.unwrap_err());
}
