//! End-to-end test: `plaude-cli --help` prints usage and exits cleanly.
//!
//! Journey: `specs/plaude-cli-v1/journeys/M00-scaffold.md`

use assert_cmd::Command;
use predicates::str::contains;

/// The compiled binary name; matches `[[bin]] name` in `Cargo.toml`
/// and the `--bin` target the `Makefile` passes to Cargo.
const BIN_NAME: &str = "plaude-cli";

/// Token that must appear in a clap-generated help page.
const HELP_USAGE_HEADER: &str = "Usage:";

#[test]
fn help_exits_zero_and_mentions_binary_name() {
    Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains(BIN_NAME))
        .stdout(contains(HELP_USAGE_HEADER));
}

#[test]
fn short_help_flag_exits_zero_and_prints_usage() {
    // `-h` and `--help` both exit zero and both print the Usage header;
    // their exact rendering is clap's concern (short-help is abbreviated
    // once a flag has a multi-paragraph doc comment, so we intentionally
    // do NOT assert byte equality).
    Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .arg("-h")
        .assert()
        .success()
        .stdout(contains(HELP_USAGE_HEADER))
        .stdout(contains(BIN_NAME));
}
