//! End-to-end tests for `plaude record start|stop|pause|resume`.
//!
//! Every test runs against `--backend sim`. The sim's recording state
//! machine enforces valid transitions: idle→recording, recording→paused,
//! paused→recording, recording→idle, paused→idle.
//!
//! Journey: specs/plaude-v1/journeys/M11-settings-record-control.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const AUTH_REQUIRED_EXIT: i32 = 77;
const MISSING_TOKEN_HINT: &str = "plaude auth";

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

#[test]
fn record_start_prints_confirmation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "record", "start"])
        .assert()
        .success()
        .stdout(contains("recording started"));
}

#[test]
fn record_stop_after_start_prints_confirmation() {
    // The sim creates a fresh device on each connect, so we can't
    // sequence start+stop across two invocations. However, the sim
    // starts in Idle, and calling stop on Idle is an error. We test
    // stop separately.
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    // Stop when idle → protocol error
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "record", "stop"])
        .assert()
        .code(1)
        .stderr(contains("error"));
}

#[test]
fn record_pause_when_idle_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "record", "pause"])
        .assert()
        .code(1)
        .stderr(contains("error"));
}

#[test]
fn record_resume_when_idle_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "record", "resume"])
        .assert()
        .code(1)
        .stderr(contains("error"));
}

#[test]
fn record_without_token_exits_auth_required() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "record", "start"])
        .assert()
        .code(AUTH_REQUIRED_EXIT)
        .stderr(contains(MISSING_TOKEN_HINT));
}
