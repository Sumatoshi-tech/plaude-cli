//! End-to-end tests for `plaude-cli files pull-one`.
//!
//! Runs against `--backend sim`. The sim backend pre-loads exactly
//! one deterministic recording whose byte contents we can assert
//! against in full.
//!
//! Journey: specs/plaude-cli-v1/journeys/M07-files-list-pull.md

use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

const BIN_NAME: &str = "plaude-cli";
const BACKEND_FLAG: &str = "--backend";
const BACKEND_SIM: &str = "sim";
const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const SIM_BASENAME: &str = "1775393534";
const SIM_WAV_BYTES: &[u8] = b"WAV-BYTES-FROM-SIM";
const SIM_ASR_BYTES: &[u8] = b"ASR-BYTES-FROM-SIM";
const WAV_EXT: &str = "wav";
const ASR_EXT: &str = "asr";
const OUTPUT_DIR_FLAG: &str = "-o";
const RESUME_FLAG: &str = "--resume";
const UNKNOWN_BASENAME: &str = "2000000000";
const RUNTIME_EXIT: i32 = 1;

fn cmd(tmp: &TempDir) -> Command {
    let mut c = Command::cargo_bin(BIN_NAME).expect("built binary");
    c.arg("--config-dir").arg(tmp.path());
    c
}

fn seed_token(tmp: &TempDir) {
    cmd(tmp).args(["auth", "set-token", SAMPLE_TOKEN]).assert().success();
}

fn wav_path(dir: &std::path::Path) -> PathBuf {
    dir.join(format!("{SIM_BASENAME}.{WAV_EXT}"))
}

fn asr_path(dir: &std::path::Path) -> PathBuf {
    dir.join(format!("{SIM_BASENAME}.{ASR_EXT}"))
}

#[test]
fn pull_one_happy_path_writes_both_files_with_exact_bytes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            SIM_BASENAME,
            OUTPUT_DIR_FLAG,
            out.path().to_str().expect("utf-8"),
        ])
        .assert()
        .success();
    let wav = std::fs::read(wav_path(out.path())).expect("wav exists");
    let asr = std::fs::read(asr_path(out.path())).expect("asr exists");
    assert_eq!(wav, SIM_WAV_BYTES);
    assert_eq!(asr, SIM_ASR_BYTES);
}

#[test]
fn pull_one_creates_missing_output_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out_root = tempfile::tempdir().expect("out root");
    let nested = out_root.path().join("sub").join("dir");
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            SIM_BASENAME,
            OUTPUT_DIR_FLAG,
            nested.to_str().expect("utf-8"),
        ])
        .assert()
        .success();
    assert!(wav_path(&nested).exists());
    assert!(asr_path(&nested).exists());
}

#[test]
fn pull_one_with_resume_skips_already_complete_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    // First pull — populates both files.
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            SIM_BASENAME,
            OUTPUT_DIR_FLAG,
            out.path().to_str().expect("utf-8"),
        ])
        .assert()
        .success();
    let wav_mtime = std::fs::metadata(wav_path(out.path())).expect("stat").modified().expect("mtime");
    // Second pull with --resume should be a no-op and not touch the file.
    std::thread::sleep(std::time::Duration::from_millis(10));
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            SIM_BASENAME,
            OUTPUT_DIR_FLAG,
            out.path().to_str().expect("utf-8"),
            RESUME_FLAG,
        ])
        .assert()
        .success();
    let wav_mtime_after = std::fs::metadata(wav_path(out.path())).expect("stat").modified().expect("mtime");
    assert_eq!(wav_mtime, wav_mtime_after, "resume must not rewrite a complete file");
}

#[test]
fn pull_one_with_resume_rewrites_a_partial_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    // Pre-seed a truncated wav file at the target path.
    std::fs::write(wav_path(out.path()), b"BOGUS").expect("pre-seed");
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            SIM_BASENAME,
            OUTPUT_DIR_FLAG,
            out.path().to_str().expect("utf-8"),
            RESUME_FLAG,
        ])
        .assert()
        .success();
    let wav = std::fs::read(wav_path(out.path())).expect("wav exists");
    assert_eq!(wav, SIM_WAV_BYTES);
}

#[test]
fn pull_one_with_unknown_id_exits_runtime_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_token(&tmp);
    let out = tempfile::tempdir().expect("out dir");
    cmd(&tmp)
        .args([
            BACKEND_FLAG,
            BACKEND_SIM,
            "files",
            "pull-one",
            UNKNOWN_BASENAME,
            OUTPUT_DIR_FLAG,
            out.path().to_str().expect("utf-8"),
        ])
        .assert()
        .code(RUNTIME_EXIT);
    assert!(!wav_path(out.path()).exists(), "no wav file on unknown id");
}
