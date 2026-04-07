//! End-to-end tests for the `plaude-cli auth` subcommand tree.
//!
//! Every test sandboxes the token file under a per-test `tempdir`
//! via the global `--config-dir` flag, so the user's real
//! `~/.config/plaude/token` is never touched.
//!
//! Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const FINGERPRINT_LINE: &str = "Fingerprint:";
const STORED_LINE: &str = "Token stored";
const REMOVED_LINE: &str = "Token removed";
const INVALID_TOKEN: &str = "not-hex";
const EXPECTED_USAGE_EXIT: i32 = 2;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built");
    c.arg("--config-dir").arg(tmp.path());
    c
}

#[test]
fn auth_set_token_stores_and_show_returns_a_fingerprint() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["auth", "set-token", SAMPLE_TOKEN])
        .assert()
        .success()
        .stdout(contains(STORED_LINE))
        .stdout(contains(FINGERPRINT_LINE));
    cmd(&tmp)
        .args(["auth", "show"])
        .assert()
        .success()
        .stdout(contains(FINGERPRINT_LINE));
}

#[test]
fn auth_set_token_does_not_print_the_raw_token() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let output = cmd(&tmp).args(["auth", "set-token", SAMPLE_TOKEN]).output().expect("run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(SAMPLE_TOKEN),
        "stdout must not contain the raw token, got: {stdout}"
    );
}

#[test]
fn auth_show_before_any_token_exits_with_usage_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["auth", "show"])
        .assert()
        .code(EXPECTED_USAGE_EXIT)
        .stderr(contains("no token stored"));
}

#[test]
fn auth_clear_is_idempotent_on_empty_sandbox() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp).args(["auth", "clear"]).assert().success().stdout(contains(REMOVED_LINE));
}

#[test]
fn auth_set_token_rejects_invalid_input_with_usage_exit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args(["auth", "set-token", INVALID_TOKEN])
        .assert()
        .code(EXPECTED_USAGE_EXIT)
        .stderr(contains("invalid token"));
}

#[test]
fn auth_set_then_clear_then_show_returns_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
    cmd(&tmp).args(["auth", "clear"]).assert().success();
    cmd(&tmp).args(["auth", "show"]).assert().code(EXPECTED_USAGE_EXIT);
}
