//! Tests for `read_setting` / `write_setting`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::{CommonSettingKey, SettingValue};
use plaud_sim::SimDevice;

const INITIAL_MIC_GAIN: u8 = 12;
const NEW_MIC_GAIN: u8 = 34;

#[tokio::test]
async fn read_setting_returns_preloaded_value() {
    let sim = SimDevice::builder()
        .with_setting(CommonSettingKey::MicGain, SettingValue::U8(INITIAL_MIC_GAIN))
        .build();
    let transport = sim.authenticated_transport();
    let value = transport.read_setting(CommonSettingKey::MicGain).await.expect("authenticated");
    assert_eq!(value, SettingValue::U8(INITIAL_MIC_GAIN));
}

#[tokio::test]
async fn write_setting_persists_across_subsequent_reads() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    transport
        .write_setting(CommonSettingKey::EnableVad, SettingValue::Bool(true))
        .await
        .expect("authenticated");
    let value = transport.read_setting(CommonSettingKey::EnableVad).await.expect("authenticated");
    assert_eq!(value, SettingValue::Bool(true));
}

#[tokio::test]
async fn write_setting_overwrites_preloaded_value() {
    let sim = SimDevice::builder()
        .with_setting(CommonSettingKey::MicGain, SettingValue::U8(INITIAL_MIC_GAIN))
        .build();
    let transport = sim.authenticated_transport();
    transport
        .write_setting(CommonSettingKey::MicGain, SettingValue::U8(NEW_MIC_GAIN))
        .await
        .expect("authenticated");
    let value = transport.read_setting(CommonSettingKey::MicGain).await.expect("authenticated");
    assert_eq!(value, SettingValue::U8(NEW_MIC_GAIN));
}

#[tokio::test]
async fn read_setting_returns_not_found_for_unset_key() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.read_setting(CommonSettingKey::AutoPowerOff).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::NotFound(_)));
}

#[tokio::test]
async fn settings_require_authentication() {
    let sim = SimDevice::builder().build();
    let transport = sim.unauthenticated_transport();
    let err = transport.read_setting(CommonSettingKey::MicGain).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::AuthRequired));
}
