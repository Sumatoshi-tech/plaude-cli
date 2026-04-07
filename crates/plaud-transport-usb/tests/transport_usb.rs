//! Integration tests for [`plaud_transport_usb::UsbTransport`].
//!
//! Every test builds a fixture directory tree that mirrors the
//! documented VFAT layout and points the transport at it. No real
//! block device is involved.
//!
//! Journey: specs/plaude-cli-v1/journeys/M10-transport-usb.md

use std::{fs, path::Path};

use plaud_domain::RecordingId;
use plaud_transport::{Error, Transport};
use plaud_transport_usb::UsbTransport;
use tempfile::TempDir;

const MODEL_TXT_CONTENTS: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024\nSerial No.:123456789012345678\n";
const BASENAME: &str = "1775393534";
const DATE_DIR: &str = "20260405";
const WAV_BODY: &[u8] = b"FAKE-WAV-BODY-BYTES";
const ASR_BODY: &[u8] = b"FAKE-ASR-BODY-BYTES";
const EXPECTED_SERIAL: &str = "123456789012345678";
const EXPECTED_BUILD: &str = "0095";

fn build_fixture() -> TempDir {
    let tmp = TempDir::new().expect("tmp");
    fs::write(tmp.path().join("MODEL.txt"), MODEL_TXT_CONTENTS).expect("model.txt");
    let notes_day = tmp.path().join("NOTES").join(DATE_DIR);
    fs::create_dir_all(&notes_day).expect("mkdir");
    fs::write(notes_day.join(format!("{BASENAME}.WAV")), WAV_BODY).expect("wav");
    fs::write(notes_day.join(format!("{BASENAME}.ASR")), ASR_BODY).expect("asr");
    tmp
}

fn id() -> RecordingId {
    RecordingId::new(BASENAME).expect("valid")
}

fn transport(root: &Path) -> UsbTransport {
    UsbTransport::new(root)
}

#[tokio::test]
async fn device_info_reads_model_txt_and_surfaces_serial_and_build() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let info = t.device_info().await.expect("ok");
    assert_eq!(info.serial.reveal(), EXPECTED_SERIAL);
    assert_eq!(info.firmware.build(), EXPECTED_BUILD);
}

#[tokio::test]
async fn list_recordings_returns_the_fixture_pair() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let list = t.list_recordings().await.expect("ok");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id().as_str(), BASENAME);
    assert_eq!(list[0].wav_size(), WAV_BODY.len() as u64);
    assert_eq!(list[0].asr_size(), ASR_BODY.len() as u64);
}

#[tokio::test]
async fn read_recording_returns_exact_wav_bytes() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let bytes = t.read_recording(&id()).await.expect("ok");
    assert_eq!(bytes, WAV_BODY);
}

#[tokio::test]
async fn read_recording_asr_returns_exact_asr_bytes() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let bytes = t.read_recording_asr(&id()).await.expect("ok");
    assert_eq!(bytes, ASR_BODY);
}

#[tokio::test]
async fn read_recording_unknown_id_returns_not_found() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let bogus = RecordingId::new("2000000000").expect("valid");
    let err = t.read_recording(&bogus).await.unwrap_err();
    assert!(matches!(err, Error::NotFound(_)));
}

#[tokio::test]
async fn battery_is_unsupported_on_usb_transport() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let err = t.battery().await.unwrap_err();
    assert!(matches!(err, Error::Unsupported { .. }));
}

#[tokio::test]
async fn set_privacy_is_unsupported_on_usb_transport() {
    let fixture = build_fixture();
    let t = transport(fixture.path());
    let err = t.set_privacy(true).await.unwrap_err();
    assert!(matches!(err, Error::Unsupported { .. }));
}
