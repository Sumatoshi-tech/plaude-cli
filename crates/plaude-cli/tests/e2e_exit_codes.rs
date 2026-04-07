//! End-to-end tests for exit-code contract.
//!
//! Verifies the sysexits(3)-aligned exit codes documented in
//! `docs/usage/exit-codes.md`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M12-hardening.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_BLE: &str = "ble";
const EX_UNAVAILABLE: i32 = 69;
const EX_NOPERM: i32 = 77;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
fn ble_backend_succeeds_or_reports_unavailable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Real btleplug backend: exit 0 if a Plaud device is nearby,
    // or exit 69 (EX_UNAVAILABLE) if no adapter/device.
    let assert = cmd(&tmp).args(["--backend", BACKEND_BLE, "battery"]).assert();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert!(
        code == 0 || code == EX_UNAVAILABLE,
        "expected exit 0 or {EX_UNAVAILABLE}, got {code}"
    );
}

#[test]
fn about_flag_prints_privacy_disclosure_and_exits_zero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .arg("--about")
        .assert()
        .success()
        .stdout(contains("PRIVACY NOTICE"))
        .stdout(contains("CLEARTEXT"))
        .stdout(contains("serial"));
}

#[test]
fn missing_token_returns_ex_noperm() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["--backend", "sim", "device", "info"])
        .assert()
        .code(EX_NOPERM)
        .stderr(contains("auth"));
}
