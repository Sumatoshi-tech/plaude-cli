//! End-to-end tests for `plaude summarize`.
//!
//! Journey: `specs/journeys/JOURNEY-L2-template-system.md` (template management)
//! Journey: `specs/journeys/JOURNEY-L4-summarization-pipeline.md` (summarization)
//! Journey: `specs/journeys/JOURNEY-L7-batch-summarization.md` (batch)

#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;

/// The compiled binary name.
const BIN_NAME: &str = "plaude";

fn cmd() -> Command {
    Command::cargo_bin(BIN_NAME).expect("binary is built by cargo test")
}

// ── Template management (L2) ────────────────────────────────────────

#[test]
fn summarize_list_templates_shows_five_builtins() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "summarize", "--list-templates"])
        .assert()
        .success()
        .stdout(contains("default"))
        .stdout(contains("meeting-notes"))
        .stdout(contains("action-items"))
        .stdout(contains("key-decisions"))
        .stdout(contains("brief"))
        .stdout(contains("5 template(s) available"));
}

#[test]
fn summarize_list_templates_includes_user_template() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl_dir = tmp.path().join("templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir");
    std::fs::write(tpl_dir.join("my-standup.md"), "Standup meeting prompt").expect("write");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "summarize", "--list-templates"])
        .assert()
        .success()
        .stdout(contains("my-standup"))
        .stdout(contains("user"))
        .stdout(contains("6 template(s) available"));
}

#[test]
fn summarize_export_template_default() {
    cmd()
        .args(["summarize", "--export-template", "default"])
        .assert()
        .success()
        .stdout(contains("summarization assistant"))
        .stdout(contains("Key Points"));
}

#[test]
fn summarize_export_template_brief() {
    cmd()
        .args(["summarize", "--export-template", "brief"])
        .assert()
        .success()
        .stdout(contains("executive summary"));
}

#[test]
fn summarize_export_unknown_template_fails() {
    cmd()
        .args(["summarize", "--export-template", "nonexistent"])
        .assert()
        .failure()
        .stderr(contains("unknown template 'nonexistent'"))
        .stderr(contains("default"));
}

#[test]
fn summarize_help_mentions_all_flags() {
    cmd()
        .args(["summarize", "--help"])
        .assert()
        .success()
        .stdout(contains("--template"))
        .stdout(contains("--model"))
        .stdout(contains("--no-stream"))
        .stdout(contains("--json"))
        .stdout(contains("--force"))
        .stdout(contains("--dry-run"));
}

#[test]
fn summarize_no_args_shows_usage_error() {
    cmd()
        .args(["summarize"])
        .assert()
        .failure()
        .stderr(contains("no recording path supplied"));
}

// ── Summarization pipeline (L4) ─────────────────────────────────────

#[test]
fn summarize_nonexistent_path_shows_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            "/nonexistent/path/to/recording",
        ])
        .assert()
        .failure()
        .stderr(contains("no transcript found"));
}

#[test]
fn summarize_unknown_template_shows_available() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("recording.txt"), "Some transcript content").expect("write");

    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            "--template",
            "nonexistent",
            tmp.path().join("recording.txt").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("unknown template 'nonexistent'"))
        .stderr(contains("default"));
}

// ── Batch summarization (L7) ────────────────────────────────────────

#[test]
fn summarize_batch_empty_dir_no_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(contains("No transcripts found"));
}

#[test]
fn summarize_batch_dry_run_lists_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("rec1.txt"), "Transcript one").expect("write");
    std::fs::write(tmp.path().join("rec2.txt"), "Transcript two").expect("write");

    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            "--dry-run",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("rec1.txt"))
        .stdout(contains("rec2.txt"))
        .stderr(contains("Would summarize 2 recording(s)"));
}

#[test]
fn summarize_batch_dry_run_skips_already_summarized() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("rec1.txt"), "Transcript one").expect("write");
    std::fs::write(tmp.path().join("rec2.txt"), "Transcript two").expect("write");
    // rec1 already has a summary.
    std::fs::write(tmp.path().join("rec1.summary.default.md"), "---\nmodel: x\n---\n\nSummary").expect("write");

    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            "--dry-run",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("rec2.txt"))
        .stderr(contains("Would summarize 1 recording(s), skip 1"));
}

#[test]
fn summarize_batch_dry_run_force_includes_all() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("rec1.txt"), "Transcript one").expect("write");
    std::fs::write(tmp.path().join("rec1.summary.default.md"), "---\n---\n\nOld summary").expect("write");

    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "summarize",
            "--dry-run",
            "--force",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("rec1.txt"))
        .stderr(contains("Would summarize 1 recording(s), skip 0"));
}
