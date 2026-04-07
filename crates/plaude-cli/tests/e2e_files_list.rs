//! End-to-end tests for `plaude files list`.
//!
//! Runs against `--backend sim`. The sim backend pre-loads exactly
//! one deterministic recording whose id the tests assert against.
//!
//! Journey: specs/plaude-v1/journeys/M07-files-list-pull.md

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const OUTPUT_FLAG: &str = "--output";
const OUTPUT_JSON: &str = "json";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const SIM_BASENAME: &str = "1775393534";
const TEXT_HEADER_ID: &str = "ID";
const TEXT_HEADER_KIND: &str = "KIND";
const TEXT_HEADER_DURATION: &str = "DURATION";
const TEXT_HEADER_SIZE: &str = "SIZE";
const JSON_ID_KEY: &str = "\"id\"";
const JSON_KIND_KEY: &str = "\"kind\"";
const JSON_WAV_SIZE_KEY: &str = "\"wav_size\"";
const JSON_ASR_SIZE_KEY: &str = "\"asr_size\"";
const AUTH_REQUIRED_EXIT: i32 = 77;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

#[test]
fn files_list_text_prints_table_header_and_preloaded_recording() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "files", "list"])
        .assert()
        .success()
        .stdout(contains(TEXT_HEADER_ID))
        .stdout(contains(TEXT_HEADER_KIND))
        .stdout(contains(TEXT_HEADER_DURATION))
        .stdout(contains(TEXT_HEADER_SIZE))
        .stdout(contains(SIM_BASENAME));
}

#[test]
fn files_list_json_emits_stable_keys() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "files", "list", OUTPUT_FLAG, OUTPUT_JSON])
        .assert()
        .success()
        .stdout(contains(JSON_ID_KEY))
        .stdout(contains(JSON_KIND_KEY))
        .stdout(contains(JSON_WAV_SIZE_KEY))
        .stdout(contains(JSON_ASR_SIZE_KEY));
}

#[test]
fn files_list_without_token_exits_with_auth_required_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd(&tmp)
        .args([BACKEND_FLAG, BACKEND_SIM, "files", "list"])
        .assert()
        .code(AUTH_REQUIRED_EXIT);
}
