//! End-to-end tests for `plaude-cli settings list|get|set`.
//!
//! Every test runs against `--backend sim`. The sim preloads three
//! settings: `enable-vad=true`, `mic-gain=20`, `auto-power-off=300`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M11-settings-record-control.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const OUTPUT_FLAG: &str = "--output";
const OUTPUT_JSON: &str = "json";
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
fn settings_list_text_prints_preloaded_settings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "list"])
        .assert()
        .success()
        .stdout(contains("enable-vad = true"))
        .stdout(contains("mic-gain = 20"))
        .stdout(contains("auto-power-off = 300"));
}

#[test]
fn settings_list_json_emits_array() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let output = cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "list", OUTPUT_FLAG, OUTPUT_JSON])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf-8");
    let parsed: serde_json::Value = serde_json::from_str(text.trim()).expect("valid json");
    assert!(parsed.is_array(), "expected JSON array, got {parsed}");
    let arr = parsed.as_array().unwrap();
    assert!(arr.len() >= 3, "expected at least 3 settings, got {}", arr.len());
}

#[test]
fn settings_get_reads_single_setting() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "get", "enable-vad"])
        .assert()
        .success()
        .stdout(contains("enable-vad = true"));
}

#[test]
fn settings_get_unknown_name_exits_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "get", "no-such-setting"])
        .assert()
        .code(2)
        .stderr(contains("unknown setting name"));
}

#[test]
fn settings_set_writes_and_confirms() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "set", "enable-vad", "false"])
        .assert()
        .success()
        .stdout(contains("enable-vad = false"));
}

#[test]
fn settings_set_bad_value_exits_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "set", "enable-vad", "not-a-value"])
        .assert()
        .code(2)
        .stderr(contains("cannot parse setting value"));
}

#[test]
fn settings_without_token_exits_auth_required() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "settings", "list"])
        .assert()
        .code(AUTH_REQUIRED_EXIT)
        .stderr(contains(MISSING_TOKEN_HINT));
}
