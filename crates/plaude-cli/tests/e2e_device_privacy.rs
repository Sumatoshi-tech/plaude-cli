//! End-to-end tests for `plaude-cli device privacy` and `device name`.
//!
//! Every test runs against `--backend sim`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M11-settings-record-control.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const AUTH_REQUIRED_EXIT: i32 = 77;
const MISSING_TOKEN_HINT: &str = "plaude-cli auth";

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

#[test]
fn device_privacy_on_prints_confirmation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "privacy", "on"])
        .assert()
        .success()
        .stdout(contains("privacy on"));
}

#[test]
fn device_privacy_off_prints_confirmation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "privacy", "off"])
        .assert()
        .success()
        .stdout(contains("privacy off"));
}

#[test]
fn device_privacy_bad_arg_exits_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "privacy", "maybe"])
        .assert()
        .code(2);
}

#[test]
fn device_name_prints_local_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "name"])
        .assert()
        .success()
        .stdout(contains("PLAUD_NOTE"));
}

#[test]
fn device_privacy_without_token_exits_auth_required() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "privacy", "on"])
        .assert()
        .code(AUTH_REQUIRED_EXIT)
        .stderr(contains(MISSING_TOKEN_HINT));
}

#[test]
fn device_name_without_token_exits_auth_required() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "device", "name"])
        .assert()
        .code(AUTH_REQUIRED_EXIT)
        .stderr(contains(MISSING_TOKEN_HINT));
}
