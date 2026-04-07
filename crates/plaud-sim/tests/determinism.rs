//! Determinism check — two independently-built sims with identical
//! configuration must produce identical traces for the same sequence
//! of operations.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::{BatteryLevel, CommonSettingKey, Recording, RecordingId, RecordingKind, SettingValue};
use plaud_sim::SimDevice;

const WAV_A: &[u8] = b"aaaa";
const ASR_A: &[u8] = b"AAAA";
const BASENAME: &str = "1775393534";
const BATTERY_PERCENT: u8 = 73;

fn build_configured_sim() -> SimDevice {
    let recording = Recording::new(
        RecordingId::new(BASENAME).expect("valid"),
        RecordingKind::Call,
        WAV_A.len() as u64,
        ASR_A.len() as u64,
    );
    SimDevice::builder()
        .with_battery(BatteryLevel::new(BATTERY_PERCENT).expect("valid"))
        .preload_recording(recording, WAV_A.to_vec(), ASR_A.to_vec())
        .with_setting(CommonSettingKey::EnableVad, SettingValue::Bool(true))
        .build()
}

async fn run_trace(sim: &SimDevice) -> Vec<String> {
    let transport = sim.authenticated_transport();
    let mut trace = Vec::new();
    trace.push(format!("{:?}", transport.battery().await));
    trace.push(format!("{:?}", transport.list_recordings().await));
    trace.push(format!(
        "{:?}",
        transport.read_recording(&RecordingId::new(BASENAME).expect("valid")).await
    ));
    trace.push(format!("{:?}", transport.read_setting(CommonSettingKey::EnableVad).await));
    trace
}

#[tokio::test]
async fn two_sims_with_the_same_builder_chain_produce_identical_traces() {
    let sim_a = build_configured_sim();
    let sim_b = build_configured_sim();
    let trace_a = run_trace(&sim_a).await;
    let trace_b = run_trace(&sim_b).await;
    assert_eq!(trace_a, trace_b);
}
