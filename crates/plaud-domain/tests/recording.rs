//! Tests for [`plaud_domain::Recording`] and [`plaud_domain::RecordingKind`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{Recording, RecordingId, RecordingKind};

const BASENAME: &str = "1775393534";
const WAV_BYTES: u64 = 1_108_992;
const ASR_BYTES: u64 = 69_280;
const UNIX_SECONDS: i64 = 1_775_393_534;

fn sample_recording(kind: RecordingKind) -> Recording {
    let id = RecordingId::new(BASENAME).expect("basename is valid");
    Recording::new(id, kind, WAV_BYTES, ASR_BYTES)
}

#[test]
fn getters_return_constructor_values() {
    let rec = sample_recording(RecordingKind::Call);
    assert_eq!(rec.id().as_str(), BASENAME);
    assert_eq!(rec.kind(), RecordingKind::Call);
    assert_eq!(rec.wav_size(), WAV_BYTES);
    assert_eq!(rec.asr_size(), ASR_BYTES);
}

#[test]
fn started_at_unix_seconds_delegates_to_id() {
    let rec = sample_recording(RecordingKind::Note);
    assert_eq!(rec.started_at_unix_seconds(), UNIX_SECONDS);
}

#[test]
fn recording_kind_name_is_stable() {
    assert_eq!(RecordingKind::Note.name(), "note");
    assert_eq!(RecordingKind::Call.name(), "call");
}

#[test]
fn recording_kind_display_matches_name() {
    assert_eq!(RecordingKind::Note.to_string(), "note");
    assert_eq!(RecordingKind::Call.to_string(), "call");
}

#[test]
fn recording_equality_includes_all_fields() {
    let a = sample_recording(RecordingKind::Note);
    let b = sample_recording(RecordingKind::Note);
    assert_eq!(a, b);

    let c = sample_recording(RecordingKind::Call);
    assert_ne!(a, c);
}

#[test]
fn recording_is_debuggable_without_panic() {
    let rec = sample_recording(RecordingKind::Call);
    let _ = format!("{rec:?}");
}
