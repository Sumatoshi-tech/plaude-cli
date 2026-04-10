//! End-to-end tests for `plaude template` command group.

#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;

const BIN_NAME: &str = "plaude";

fn cmd() -> Command {
    Command::cargo_bin(BIN_NAME).expect("binary is built by cargo test")
}

#[test]
fn template_help_exits_zero() {
    cmd()
        .args(["template", "--help"])
        .assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("show"))
        .stdout(contains("add"))
        .stdout(contains("edit"))
        .stdout(contains("delete"));
}

#[test]
fn template_list_shows_builtins() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "list"])
        .assert()
        .success()
        .stdout(contains("default"))
        .stdout(contains("brief"))
        .stdout(contains("built-in"))
        .stdout(contains("5 template(s) available"));
}

#[test]
fn template_show_builtin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "show", "default"])
        .assert()
        .success()
        .stdout(contains("summarization assistant"));
}

#[test]
fn template_show_unknown_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "show", "nope"])
        .assert()
        .failure()
        .stderr(contains("template 'nope' not found"));
}

#[test]
fn template_add_creates_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "add", "my-standup"])
        .assert()
        .success()
        .stderr(contains("Created template 'my-standup'"));

    assert!(tmp.path().join("templates/my-standup.md").exists());
}

#[test]
fn template_add_from_builtin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "template",
            "add",
            "my-brief",
            "--from",
            "brief",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path().join("templates/my-brief.md")).expect("read");
    assert!(content.contains("executive summary"));
}

#[test]
fn template_add_duplicate_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl_dir = tmp.path().join("templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir");
    std::fs::write(tpl_dir.join("existing.md"), "already here").expect("write");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "add", "existing"])
        .assert()
        .failure()
        .stderr(contains("already exists"));
}

#[test]
fn template_add_from_unknown_builtin_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args([
            "--config-dir",
            tmp.path().to_str().unwrap(),
            "template",
            "add",
            "foo",
            "--from",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(contains("unknown built-in template"));
}

#[test]
fn template_delete_removes_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl_dir = tmp.path().join("templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir");
    std::fs::write(tpl_dir.join("to-remove.md"), "temp").expect("write");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "delete", "to-remove"])
        .assert()
        .success()
        .stderr(contains("Deleted template 'to-remove'"));

    assert!(!tpl_dir.join("to-remove.md").exists());
}

#[test]
fn template_delete_builtin_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "delete", "default"])
        .assert()
        .failure()
        .stderr(contains("built-in template and cannot be deleted"));
}

#[test]
fn template_delete_unknown_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "delete", "nope"])
        .assert()
        .failure()
        .stderr(contains("not found"));
}

#[test]
fn template_list_shows_user_template() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl_dir = tmp.path().join("templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir");
    std::fs::write(tpl_dir.join("custom.md"), "My custom prompt").expect("write");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "list"])
        .assert()
        .success()
        .stdout(contains("custom"))
        .stdout(contains("user"))
        .stdout(contains("6 template(s) available"));
}

#[test]
fn template_rm_alias_works() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl_dir = tmp.path().join("templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir");
    std::fs::write(tpl_dir.join("alias-test.md"), "temp").expect("write");

    cmd()
        .args(["--config-dir", tmp.path().to_str().unwrap(), "template", "rm", "alias-test"])
        .assert()
        .success()
        .stderr(contains("Deleted"));
}
