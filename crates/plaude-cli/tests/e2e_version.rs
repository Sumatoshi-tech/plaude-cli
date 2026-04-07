//! End-to-end test: `plaude --version` matches `CARGO_PKG_VERSION`.
//!
//! Journey: `specs/plaude-v1/journeys/M00-scaffold.md`

use assert_cmd::Command;
use predicates::str::contains;

/// The compiled binary name; matches `[[bin]] name` in `Cargo.toml`.
const BIN_NAME: &str = "plaude";

/// Package version injected by Cargo at build time. We reference it
/// via `env!` so the test stays in lock-step with `Cargo.toml` and
/// fails loudly on any version mismatch.
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn version_exits_zero_and_prints_matching_semver() {
    Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(PKG_VERSION));
}

#[test]
fn short_version_flag_matches_long_version_flag() {
    let long = Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .arg("--version")
        .assert()
        .success();
    let short = Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .arg("-V")
        .assert()
        .success();
    assert_eq!(
        long.get_output().stdout,
        short.get_output().stdout,
        "-V and --version must render identical version output"
    );
}
