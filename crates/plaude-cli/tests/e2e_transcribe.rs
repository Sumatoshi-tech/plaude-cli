//! End-to-end tests for `plaude transcribe`.
//!
//! Tests use a mock whisper binary (shell script) that echoes back
//! predictable output instead of actually running whisper.cpp.
//!
//! Journey: specs/plaude-v1/journeys/M15-whisper-transcribe.md

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const EX_USAGE: i32 = 2;
const EX_UNAVAILABLE: i32 = 69;
const EX_RUNTIME: i32 = 1;
const MOCK_TRANSCRIPT: &str = "Hello world from mock whisper";

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

/// Create a mock whisper binary that prints a fixed transcript to
/// stdout and exits 0.
fn create_mock_whisper(tmp: &TempDir) -> std::path::PathBuf {
    let script = tmp.path().join("mock-whisper");
    #[cfg(unix)]
    {
        fs::write(&script, format!("#!/bin/sh\necho '{MOCK_TRANSCRIPT}'\n")).expect("write mock");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    #[cfg(not(unix))]
    {
        // On non-Unix, create a .bat or skip — tests are Unix-primary.
        fs::write(&script, format!("@echo off\necho {MOCK_TRANSCRIPT}\n")).expect("write mock");
    }
    script
}

/// Create a mock whisper binary that always fails.
fn create_failing_whisper(tmp: &TempDir) -> std::path::PathBuf {
    let script = tmp.path().join("fail-whisper");
    #[cfg(unix)]
    {
        fs::write(&script, "#!/bin/sh\necho 'whisper error' >&2\nexit 1\n").expect("write mock");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    #[cfg(not(unix))]
    {
        fs::write(&script, "@echo off\necho whisper error 1>&2\nexit /b 1\n").expect("write mock");
    }
    script
}

/// Create a dummy WAV file (content doesn't matter for mock tests).
fn create_dummy_wav(tmp: &TempDir, name: &str) -> std::path::PathBuf {
    let wav = tmp.path().join(name);
    fs::write(&wav, b"RIFF\x00\x00\x00\x00WAVEfmt ").expect("write wav");
    wav
}

/// Create a dummy model file.
fn create_dummy_model(tmp: &TempDir) -> std::path::PathBuf {
    let model = tmp.path().join("model.bin");
    fs::write(&model, b"dummy-model-data").expect("write model");
    model
}

#[test]
fn transcribe_happy_path_prints_transcript() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let whisper = create_mock_whisper(&tmp);
    let model = create_dummy_model(&tmp);
    let wav = create_dummy_wav(&tmp, "recording.wav");

    cmd(&tmp)
        .args([
            "transcribe",
            "--whisper-bin",
            whisper.to_str().unwrap(),
            "--model",
            model.to_str().unwrap(),
            wav.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains(MOCK_TRANSCRIPT));
}

#[test]
fn transcribe_missing_whisper_binary_exits_unavailable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let model = create_dummy_model(&tmp);
    let wav = create_dummy_wav(&tmp, "recording.wav");

    cmd(&tmp)
        .args([
            "transcribe",
            "--whisper-bin",
            "/nonexistent/whisper-bin",
            "--model",
            model.to_str().unwrap(),
            wav.to_str().unwrap(),
        ])
        .assert()
        .code(EX_UNAVAILABLE)
        .stderr(contains("whisper binary not found"));
}

#[test]
fn transcribe_missing_model_exits_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let whisper = create_mock_whisper(&tmp);
    let wav = create_dummy_wav(&tmp, "recording.wav");

    cmd(&tmp)
        .args([
            "transcribe",
            "--whisper-bin",
            whisper.to_str().unwrap(),
            "--model",
            "/nonexistent/model.bin",
            wav.to_str().unwrap(),
        ])
        .assert()
        .code(EX_USAGE)
        .stderr(contains("model file not found"));
}

#[test]
fn transcribe_missing_wav_exits_runtime() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let whisper = create_mock_whisper(&tmp);
    let model = create_dummy_model(&tmp);

    cmd(&tmp)
        .args([
            "transcribe",
            "--whisper-bin",
            whisper.to_str().unwrap(),
            "--model",
            model.to_str().unwrap(),
            "/nonexistent/recording.wav",
        ])
        .assert()
        .code(EX_RUNTIME)
        .stderr(contains("WAV file not found"));
}

#[test]
fn transcribe_whisper_failure_exits_runtime() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let whisper = create_failing_whisper(&tmp);
    let model = create_dummy_model(&tmp);
    let wav = create_dummy_wav(&tmp, "recording.wav");

    cmd(&tmp)
        .args([
            "transcribe",
            "--whisper-bin",
            whisper.to_str().unwrap(),
            "--model",
            model.to_str().unwrap(),
            wav.to_str().unwrap(),
        ])
        .assert()
        .code(EX_RUNTIME)
        .stderr(contains("whisper exited"));
}

#[test]
fn transcribe_requires_at_least_one_file_arg() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let model = create_dummy_model(&tmp);

    cmd(&tmp)
        .args(["transcribe", "--model", model.to_str().unwrap()])
        .assert()
        .code(EX_USAGE);
}
