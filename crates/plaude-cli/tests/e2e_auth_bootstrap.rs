//! End-to-end tests for `plaude auth bootstrap`.
//!
//! Runs against `--backend sim` — the CLI spawns a hermetic loopback
//! peripheral and a fake phone that writes a deterministic auth
//! token. The real BlueZ peripheral lands in a later milestone.
//!
//! Journey: specs/plaude-v1/journeys/M08-auth-bootstrap-peripheral.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const BACKEND_BLE: &str = "ble";
const BOOTSTRAP_FINGERPRINT_LINE: &str = "Token captured. Fingerprint:";
const SHOW_FINGERPRINT_LINE: &str = "Fingerprint:";
const UNAVAILABLE_EXIT: i32 = 69;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
fn auth_bootstrap_sim_captures_deterministic_token_and_prints_fingerprint() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "auth", "bootstrap"])
        .assert()
        .success()
        .stdout(contains(BOOTSTRAP_FINGERPRINT_LINE));
}

#[test]
fn auth_bootstrap_sim_stores_token_so_auth_show_sees_it_afterwards() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp).args([BACKEND_FLAG, BACKEND_SIM, "auth", "bootstrap"]).assert().success();
    cmd(&tmp)
        .args(["auth", "show"])
        .assert()
        .success()
        .stdout(contains(SHOW_FINGERPRINT_LINE));
}

#[test]
fn auth_bootstrap_ble_backend_does_not_panic() {
    // The real BlueZ peripheral either advertises (if adapter present)
    // or fails with a runtime error. We just verify it doesn't panic.
    let tmp = tempfile::tempdir().expect("tempdir");
    let assert = cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_BLE, "auth", "bootstrap", "--timeout", "2"])
        .assert();
    let code = assert.get_output().status.code().unwrap_or(-1);
    // Accept: 0 (success), 1 (runtime: timeout/BlueZ), 69 (unavailable)
    assert!(code == 0 || code == 1 || code == UNAVAILABLE_EXIT, "unexpected exit code: {code}");
}

#[test]
fn auth_bootstrap_sim_two_runs_in_a_row_both_succeed_idempotently() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp).args([BACKEND_FLAG, BACKEND_SIM, "auth", "bootstrap"]).assert().success();
    cmd(&tmp).args([BACKEND_FLAG, BACKEND_SIM, "auth", "bootstrap"]).assert().success();
}
