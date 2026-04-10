//! End-to-end tests for `plaude correct` command.
//!
//! Journey: `specs/journeys/JOURNEY-L6-transcript-correction.md`

#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;

const BIN_NAME: &str = "plaude";

fn cmd() -> Command {
    Command::cargo_bin(BIN_NAME).expect("binary is built by cargo test")
}

#[test]
fn correct_help_exits_zero() {
    cmd()
        .args(["correct", "--help"])
        .assert()
        .success()
        .stdout(contains("--glossary"))
        .stdout(contains("--model"))
        .stdout(contains("--no-stream"));
}

#[test]
fn correct_no_args_shows_usage() {
    cmd()
        .args(["correct"])
        .assert()
        .failure()
        .stderr(contains("no transcript path supplied"));
}

#[test]
fn correct_missing_file_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "correct",
            "/nonexistent/transcript.txt",
        ])
        .assert()
        .failure()
        .stderr(contains("file not found"));
}

#[test]
fn correct_missing_glossary_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let transcript = tmp.path().join("recording.txt");
    std::fs::write(&transcript, "Hello world").expect("write");

    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "correct",
            "--glossary",
            "/nonexistent/glossary.txt",
            transcript.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("failed to load glossary"));
}
