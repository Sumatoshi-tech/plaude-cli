//! Tests for `list_recordings`, `read_recording`, and
//! `delete_recording`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::{Recording, RecordingId, RecordingKind};
use plaud_sim::SimDevice;

const BASENAME_A: &str = "1775393534";
const BASENAME_B: &str = "1775393540";
const WAV_A: &[u8] = b"A-WAV";
const WAV_B: &[u8] = b"BBBB-WAV";
const ASR_A: &[u8] = b"A-ASR";
const ASR_B: &[u8] = b"BBBB-ASR";

fn id(basename: &str) -> RecordingId {
    RecordingId::new(basename).expect("hand-validated basename")
}

fn recording_for(basename: &str, wav: &[u8], asr: &[u8]) -> Recording {
    Recording::new(id(basename), RecordingKind::Note, wav.len() as u64, asr.len() as u64)
}

#[tokio::test]
async fn list_recordings_returns_every_preloaded_entry_sorted_by_id() {
    let sim = SimDevice::builder()
        .preload_recording(recording_for(BASENAME_B, WAV_B, ASR_B), WAV_B.to_vec(), ASR_B.to_vec())
        .preload_recording(recording_for(BASENAME_A, WAV_A, ASR_A), WAV_A.to_vec(), ASR_A.to_vec())
        .build();
    let transport = sim.authenticated_transport();
    let list = transport.list_recordings().await.expect("authenticated");
    let ids: Vec<_> = list.iter().map(|r| r.id().as_str().to_owned()).collect();
    assert_eq!(ids, vec![BASENAME_A.to_owned(), BASENAME_B.to_owned()]);
}

#[tokio::test]
async fn read_recording_returns_preloaded_wav_bytes() {
    let sim = SimDevice::builder()
        .preload_recording(recording_for(BASENAME_A, WAV_A, ASR_A), WAV_A.to_vec(), ASR_A.to_vec())
        .build();
    let transport = sim.authenticated_transport();
    let bytes = transport.read_recording(&id(BASENAME_A)).await.expect("authenticated");
    assert_eq!(bytes, WAV_A);
}

#[tokio::test]
async fn read_recording_missing_id_returns_not_found() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.read_recording(&id(BASENAME_A)).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::NotFound(_)));
}

#[tokio::test]
async fn delete_recording_removes_it_from_the_listing() {
    let sim = SimDevice::builder()
        .preload_recording(recording_for(BASENAME_A, WAV_A, ASR_A), WAV_A.to_vec(), ASR_A.to_vec())
        .build();
    let transport = sim.authenticated_transport();
    transport.delete_recording(&id(BASENAME_A)).await.expect("delete");
    let list = transport.list_recordings().await.expect("authenticated");
    assert!(list.is_empty());
}

#[tokio::test]
async fn delete_recording_missing_id_returns_not_found() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.delete_recording(&id(BASENAME_A)).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::NotFound(_)));
}

#[tokio::test]
async fn read_recording_asr_returns_preloaded_asr_bytes() {
    let sim = SimDevice::builder()
        .preload_recording(recording_for(BASENAME_A, WAV_A, ASR_A), WAV_A.to_vec(), ASR_A.to_vec())
        .build();
    let transport = sim.authenticated_transport();
    let bytes = transport.read_recording_asr(&id(BASENAME_A)).await.expect("authenticated");
    assert_eq!(bytes, ASR_A);
}

#[tokio::test]
async fn read_recording_asr_missing_id_returns_not_found() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.read_recording_asr(&id(BASENAME_A)).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::NotFound(_)));
}

#[tokio::test]
async fn asr_bytes_for_returns_preloaded_asr() {
    let sim = SimDevice::builder()
        .preload_recording(recording_for(BASENAME_A, WAV_A, ASR_A), WAV_A.to_vec(), ASR_A.to_vec())
        .build();
    let bytes = sim.asr_bytes_for(&id(BASENAME_A)).expect("preloaded");
    assert_eq!(bytes, ASR_A);
}
