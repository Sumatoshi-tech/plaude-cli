//! End-to-end tests for `plaude-cli device info`.
//!
//! Every test runs against `--backend sim`. Missing-token and
//! rejected-token failure paths are driven via the sandbox token
//! file and the `PLAUDE_SIM_REJECT` env override.
//!
//! Journey: specs/plaude-cli-v1/journeys/M06-battery-device-info.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const OUTPUT_FLAG: &str = "--output";
const OUTPUT_JSON: &str = "json";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const LOCAL_NAME_MARKER: &str = "PLAUD_NOTE";
const MODEL_MARKER: &str = "Plaud Note";
const FIRMWARE_HEADER: &str = "Firmware:";
const SERIAL_HEADER: &str = "Serial:";
const STORAGE_HEADER: &str = "Storage:";
const JSON_LOCAL_NAME_KEY: &str = "\"local_name\"";
const JSON_FIRMWARE_KEY: &str = "\"firmware\"";
const JSON_STORAGE_KEY: &str = "\"storage\"";
const AUTH_REQUIRED_EXIT: i32 = 77;
const AUTH_REJECTED_EXIT: i32 = 78;
const MISSING_TOKEN_HINT: &str = "plaude-cli auth";
const REJECTED_TOKEN_HINT: &str = "bootstrap";
const SIM_REJECT_ENV: &str = "PLAUDE_SIM_REJECT";
const SIM_REJECT_ON: &str = "1";

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

#[test]
fn device_info_text_prints_model_and_firmware_and_serial_and_storage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "info"])
        .assert()
        .success()
        .stdout(contains(LOCAL_NAME_MARKER))
        .stdout(contains(MODEL_MARKER))
        .stdout(contains(FIRMWARE_HEADER))
        .stdout(contains(SERIAL_HEADER))
        .stdout(contains(STORAGE_HEADER));
}

#[test]
fn device_info_json_emits_stable_schema() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "info", OUTPUT_FLAG, OUTPUT_JSON])
        .assert()
        .success()
        .stdout(contains(JSON_LOCAL_NAME_KEY))
        .stdout(contains(JSON_FIRMWARE_KEY))
        .stdout(contains(JSON_STORAGE_KEY));
}

#[test]
fn device_info_without_token_exits_with_auth_required_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // No token seeded.
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "info"])
        .assert()
        .code(AUTH_REQUIRED_EXIT)
        .stderr(contains(MISSING_TOKEN_HINT));
}

#[test]
fn device_info_with_rejected_token_exits_with_auth_rejected_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .env(SIM_REJECT_ENV, SIM_REJECT_ON)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "info"])
        .assert()
        .code(AUTH_REJECTED_EXIT)
        .stderr(contains(REJECTED_TOKEN_HINT));
}

#[test]
fn device_info_does_not_leak_raw_token_to_stdout_or_stderr() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let output = cmd(&tmp).args([BACKEND_FLAG, BACKEND_SIM, "device", "info"]).output().expect("run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(SAMPLE_TOKEN), "stdout leaked token: {stdout}");
    assert!(!stderr.contains(SAMPLE_TOKEN), "stderr leaked token: {stderr}");
}
