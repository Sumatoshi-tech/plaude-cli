//! End-to-end tests for `plaude llm` commands.
//!
//! Journey: `specs/journeys/JOURNEY-L1-llm-config-provider.md`

#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;

/// The compiled binary name.
const BIN_NAME: &str = "plaude";

fn cmd() -> Command {
    Command::cargo_bin(BIN_NAME).expect("binary is built by cargo test")
}

#[test]
fn llm_help_exits_zero() {
    cmd().args(["llm", "--help"]).assert().success().stdout(contains("check"));
}

#[test]
fn llm_check_no_config_prints_default_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "llm", "check"])
        .assert()
        .success()
        .stdout(contains("llama3.2:3b"))
        .stdout(contains("auto-detect"));
}

#[test]
fn llm_check_with_config_file_shows_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("llm.toml"), "model = \"gpt-4o-mini\"\n").expect("write config");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "llm", "check"])
        .assert()
        .success()
        .stdout(contains("gpt-4o-mini"));
}

#[test]
fn llm_check_with_custom_provider_shows_details() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        tmp.path().join("llm.toml"),
        r#"
model = "local-model"

[provider]
kind = "openai"
base_url = "http://localhost:1234/v1"
api_key_env = "TEST_LLM_KEY"
"#,
    )
    .expect("write config");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "llm", "check"])
        .assert()
        .success()
        .stdout(contains("local-model"))
        .stdout(contains("openai"))
        .stdout(contains("http://localhost:1234/v1"))
        .stdout(contains("$TEST_LLM_KEY"));
}

#[test]
fn llm_check_invalid_config_shows_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("llm.toml"), "{{broken}}").expect("write config");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "llm", "check"])
        .assert()
        .failure()
        .stderr(contains("invalid llm.toml"));
}
