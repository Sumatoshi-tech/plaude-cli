//! End-to-end test: `plaude` with no args exits with a usage error.
//!
//! CLIG (Command Line Interface Guidelines) mandates exit code 2 for
//! "command was used incorrectly" — missing a required subcommand is
//! exactly that case. We verify the exit code and that the user is
//! shown a usage hint on stderr or stdout so they can self-recover.
//!
//! Journey: `specs/plaude-v1/journeys/M00-scaffold.md`

use assert_cmd::Command;
use predicates::{Predicate, str::contains};

/// The compiled binary name.
const BIN_NAME: &str = "plaude";

/// CLIG-compliant exit code for "incorrect usage". See
/// <https://clig.dev/#the-basics> and the sysexits(3) convention.
const EXIT_USAGE: i32 = 2;

/// Token that clap emits in its help page.
const HELP_USAGE_HEADER: &str = "Usage:";

#[test]
fn no_args_exits_two_and_prints_usage_hint() {
    let assertion = Command::cargo_bin(BIN_NAME)
        .expect("binary is built by cargo test")
        .assert()
        .code(EXIT_USAGE);
    // clap prints the usage hint to stderr OR stdout depending on
    // how we trigger the help path; we tolerate either.
    let output = assertion.get_output();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        contains(HELP_USAGE_HEADER).eval(combined.as_str()) || contains(BIN_NAME).eval(combined.as_str()),
        "expected usage hint in combined output, got:\n{combined}"
    );
}
