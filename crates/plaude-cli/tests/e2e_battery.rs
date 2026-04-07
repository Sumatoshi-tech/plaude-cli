//! End-to-end tests for `plaude-cli battery`.
//!
//! Every test runs against `--backend sim`; the real BLE backend is
//! stubbed until the btleplug wire-up milestone. No hardware required.
//!
//! Journey: specs/plaude-cli-v1/journeys/M06-battery-device-info.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const BACKEND_BLE: &str = "ble";
const BATTERY_LINE_PREFIX: &str = "Battery:";
const OUTPUT_FLAG: &str = "--output";
const OUTPUT_JSON: &str = "json";
const JSON_PERCENT_KEY: &str = "\"percent\"";
const DEFAULT_SIM_BATTERY_PERCENT: u8 = 100;
const UNSUPPORTED_EXIT_CODE: i32 = 69;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
fn battery_text_prints_percent_against_sim() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "battery"])
        .assert()
        .success()
        .stdout(contains(BATTERY_LINE_PREFIX))
        .stdout(contains(format!("{DEFAULT_SIM_BATTERY_PERCENT}%")));
}

#[test]
fn battery_works_without_any_stored_token() {
    // Battery is the SIG analogue — no auth token needed. This pins
    // the Test 2b invariant at the CLI level.
    let tmp = tempfile::tempdir().expect("tempdir");
    // Intentionally do NOT set a token first.
    cmd(&tmp).args([BACKEND_FLAG, BACKEND_SIM, "battery"]).assert().success();
}

#[test]
fn battery_json_emits_percent_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "battery", OUTPUT_FLAG, OUTPUT_JSON])
        .assert()
        .success()
        .stdout(contains(JSON_PERCENT_KEY));
}

#[test]
fn battery_ble_backend_succeeds_or_reports_unavailable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Real btleplug backend: succeeds if a Plaud device is nearby,
    // or exits 69 (EX_UNAVAILABLE) if no adapter/device is available.
    let assert = cmd(&tmp).args([BACKEND_FLAG, BACKEND_BLE, "battery"]).assert();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert!(
        code == 0 || code == UNSUPPORTED_EXIT_CODE,
        "expected exit 0 or {UNSUPPORTED_EXIT_CODE}, got {code}"
    );
}
