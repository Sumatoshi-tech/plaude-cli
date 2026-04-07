//! End-to-end tests for `plaude sync`.
//!
//! Uses the `PLAUDE_SIM_RECORDINGS` env var exposed by the sim
//! backend to vary the device fixture between runs inside a single
//! test, covering the empty/one-file/incremental/deleted/no-op
//! matrix without needing a second process.
//!
//! Journey: specs/plaude-v1/journeys/M09-sync.md

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const SIM_RECORDINGS_ENV: &str = "PLAUDE_SIM_RECORDINGS";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const BASENAME_A: &str = "1775393534";
const BASENAME_B: &str = "1775393540";
const WAV_EXT: &str = "wav";
const ASR_EXT: &str = "asr";
const STATE_FILE_NAME: &str = ".plaude-sync.json";
const NOTHING_TO_DO_MSG: &str = "nothing to do";
const WOULD_PULL_PREFIX: &str = "would pull:";
const DELETED_PREFIX: &str = "deleted on device";
const AUTH_REQUIRED_EXIT: i32 = 77;
const DRY_RUN_FLAG: &str = "--dry-run";

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

fn wav_path(dir: &Path, basename: &str) -> PathBuf {
    dir.join(format!("{basename}.{WAV_EXT}"))
}

fn asr_path(dir: &Path, basename: &str) -> PathBuf {
    dir.join(format!("{basename}.{ASR_EXT}"))
}

#[test]
fn sync_on_empty_device_writes_only_state_file_with_no_recordings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, "")
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success();
    assert!(out.path().join(STATE_FILE_NAME).exists());
    // Nothing else on disk.
    assert!(!wav_path(out.path(), BASENAME_A).exists());
}

#[test]
fn sync_one_file_pulls_both_wav_and_asr_and_writes_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, BASENAME_A)
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success();
    assert!(wav_path(out.path(), BASENAME_A).exists());
    assert!(asr_path(out.path(), BASENAME_A).exists());
    assert!(out.path().join(STATE_FILE_NAME).exists());
}

#[test]
fn sync_second_run_with_same_device_is_a_noop() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    let run = |extra: bool| {
        let mut c = cmd(&tmp);
        c.env(SIM_RECORDINGS_ENV, BASENAME_A)
            .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")]);
        if extra {
            c.assert().success().stdout(contains(NOTHING_TO_DO_MSG));
        } else {
            c.assert().success();
        }
    };
    run(false);
    run(true);
}

#[test]
fn sync_incremental_only_pulls_the_newly_added_recording() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    // First run: one recording.
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, BASENAME_A)
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success();
    // Second run: two recordings. Only B should be new.
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, format!("{BASENAME_A},{BASENAME_B}"))
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success();
    assert!(wav_path(out.path(), BASENAME_A).exists());
    assert!(wav_path(out.path(), BASENAME_B).exists());
}

#[test]
fn sync_dry_run_prints_plan_without_writing_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, BASENAME_A)
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8"), DRY_RUN_FLAG])
        .assert()
        .success()
        .stdout(contains(WOULD_PULL_PREFIX))
        .stdout(contains(BASENAME_A));
    assert!(!wav_path(out.path(), BASENAME_A).exists());
    assert!(!out.path().join(STATE_FILE_NAME).exists());
}

#[test]
fn sync_reports_deleted_on_device_without_removing_local_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    // Seed: two recordings on the sim.
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, format!("{BASENAME_A},{BASENAME_B}"))
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success();
    assert!(wav_path(out.path(), BASENAME_A).exists());
    assert!(wav_path(out.path(), BASENAME_B).exists());
    // Second run: only A on sim. B is deleted on device.
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, BASENAME_A)
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .success()
        .stderr(contains(DELETED_PREFIX));
    // B's local files are untouched.
    assert!(wav_path(out.path(), BASENAME_B).exists());
}

#[test]
fn sync_without_token_exits_with_auth_required_code() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .env(SIM_RECORDINGS_ENV, BASENAME_A)
        .args([BACKEND_FLAG, BACKEND_SIM, "sync", out.path().to_str().expect("utf-8")])
        .assert()
        .code(AUTH_REQUIRED_EXIT);
}
