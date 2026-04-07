//! End-to-end tests for `plaude transcribe`.
//!
//! Tests verify CLI argument handling and error messages. Actual
//! transcription tests require a whisper model and are feature-gated.
//!
//! Journey: specs/transcription/ROADMAP.md — T1

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const EX_USAGE: i32 = 2;
const EX_RUNTIME: i32 = 1;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
fn transcribe_requires_at_least_one_file_arg() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp).args(["transcribe", "--model", "/tmp/fake.bin"]).assert().code(EX_USAGE);
}

#[test]
fn transcribe_missing_wav_exits_runtime_with_tip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["transcribe", "--model", "/tmp/fake.bin", "/nonexistent/recording.wav"])
        .assert()
        .code(EX_RUNTIME)
        .stderr(contains("WAV file not found"))
        .stderr(contains("plaude files list"));
}

#[test]
fn transcribe_missing_model_with_no_download_exits_runtime() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wav = tmp.path().join("test.wav");
    std::fs::write(&wav, b"RIFF\x00\x00\x00\x00WAVEfmt ").expect("write wav");
    cmd(&tmp)
        .args([
            "transcribe",
            "--model",
            "/nonexistent/model.bin",
            "--no-download",
            wav.to_str().unwrap(),
        ])
        .assert()
        .code(EX_RUNTIME)
        .stderr(contains("model not found"));
}

#[test]
fn transcribe_accepts_quality_flag() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wav = tmp.path().join("test.wav");
    std::fs::write(&wav, b"RIFF\x00\x00\x00\x00WAVEfmt ").expect("write wav");
    // This will fail because the model doesn't exist, but the error
    // message will contain the model filename — proving the quality
    // preset was resolved correctly.
    let output = cmd(&tmp)
        .args(["transcribe", "--quality", "fast", wav.to_str().unwrap()])
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ggml-tiny.en.bin"),
        "expected fast preset to resolve to ggml-tiny.en.bin, got: {stderr}"
    );
}

#[test]
fn transcribe_list_models_prints_table() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["transcribe", "--list-models"])
        .assert()
        .success()
        .stdout(contains("PRESET"))
        .stdout(contains("ggml-tiny.en.bin"))
        .stdout(contains("ggml-distil-medium.en.bin"))
        .stdout(contains("ggml-large-v3-turbo.bin"))
        .stdout(contains("huggingface.co"));
}

#[test]
fn transcribe_multilingual_language_uses_different_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wav = tmp.path().join("test.wav");
    std::fs::write(&wav, b"RIFF\x00\x00\x00\x00WAVEfmt ").expect("write wav");
    let output = cmd(&tmp)
        .args(["transcribe", "--quality", "fast", "--language", "de", wav.to_str().unwrap()])
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // German should use multilingual model (no .en suffix)
    assert!(
        stderr.contains("ggml-tiny.bin"),
        "expected multilingual model for --language de, got: {stderr}"
    );
}
