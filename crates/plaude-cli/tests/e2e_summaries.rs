//! End-to-end tests for `plaude summaries` command group.
//!
//! Journey: `specs/journeys/JOURNEY-L5-summary-management.md`

#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;

const BIN_NAME: &str = "plaude";

fn cmd() -> Command {
    Command::cargo_bin(BIN_NAME).expect("binary is built by cargo test")
}

/// Create a fixture summary file with front matter.
fn write_fixture_summary(dir: &std::path::Path, id: &str, template: &str) {
    let content = format!(
        "---\nmodel: llama3.2:3b\ntemplate: {template}\ncreated_at: 2026-04-08T10:00:00+00:00\ntoken_count: 42\n---\n\nThis is the summary body for {id}.\n"
    );
    let filename = format!("{id}.summary.{template}.md");
    std::fs::write(dir.join(filename), content).expect("write fixture");
}

#[test]
fn summaries_help_exits_zero() {
    cmd()
        .args(["summaries", "--help"])
        .assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("show"))
        .stdout(contains("delete"));
}

#[test]
fn summaries_list_with_fixture_shows_entry() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    cmd()
        .args(["summaries", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("default"))
        .stdout(contains("llama3.2:3b"))
        .stdout(contains("1 summary(ies) found"));
}

#[test]
fn summaries_list_multiple_templates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");
    write_fixture_summary(tmp.path(), "1712345678", "brief");

    cmd()
        .args(["summaries", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("default"))
        .stdout(contains("brief"))
        .stdout(contains("2 summary(ies) found"));
}

#[test]
fn summaries_list_empty_shows_hint() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["summaries", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("No summaries found"))
        .stderr(contains("plaude summarize"));
}

#[test]
fn summaries_list_json_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    cmd()
        .args(["summaries", "list", "--json", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("\"template\": \"default\""))
        .stdout(contains("\"model\": \"llama3.2:3b\""));
}

#[test]
fn summaries_show_prints_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    cmd()
        .args(["summaries", "show", "--template", "default", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("This is the summary body"));
}

#[test]
fn summaries_show_without_template_shows_most_recent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    cmd()
        .args(["summaries", "show", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("This is the summary body"));
}

#[test]
fn summaries_show_missing_template_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    cmd()
        .args(["summaries", "show", "--template", "nonexistent", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("No summary with template 'nonexistent'"))
        .stderr(contains("default"));
}

#[test]
fn summaries_show_no_summaries_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["summaries", "show", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("No summaries found"));
}

#[test]
fn summaries_delete_removes_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_fixture_summary(tmp.path(), "1712345678", "default");

    let summary_file = tmp.path().join("1712345678.summary.default.md");
    assert!(summary_file.exists());

    cmd()
        .args(["summaries", "delete", "--template", "default", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(contains("Deleted summary"));

    assert!(!summary_file.exists());
}

#[test]
fn summaries_delete_missing_template_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["summaries", "delete", "--template", "nonexistent", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("no summary with template 'nonexistent'"));
}
