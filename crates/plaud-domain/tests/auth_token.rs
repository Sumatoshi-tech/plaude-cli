//! Tests for [`plaud_domain::AuthToken`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{AuthToken, AuthTokenError};

const TOKEN_LEN_LONG: usize = 32;
const TOKEN_LEN_SHORT: usize = 16;

// Placeholders — never real user tokens. Hex digits only so they
// satisfy the character-class check; length set per variant.
const FAKE_LONG_TOKEN: &str = "0123456789abcdef0123456789abcdef";
const FAKE_SHORT_TOKEN: &str = "0123456789abcdef";
const FAKE_UPPERCASE_LONG_TOKEN: &str = "0123456789ABCDEF0123456789ABCDEF";

const INVALID_LENGTH_15: &str = "0123456789abcde";
const INVALID_LENGTH_33: &str = "0123456789abcdef0123456789abcdef0";
const NON_HEX_INPUT_32: &str = "g123456789abcdef0123456789abcdef";

const EXPECTED_REDACTION_MARKER: &str = "<redacted>";
const MIN_RUN_OF_HEX_THAT_WOULD_LEAK_TOKEN: usize = 8;

fn contains_long_hex_run(haystack: &str, min_run: usize) -> bool {
    let mut run = 0usize;
    for byte in haystack.as_bytes() {
        if byte.is_ascii_hexdigit() {
            run += 1;
            if run >= min_run {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

#[test]
fn new_accepts_long_form_token() {
    let token = AuthToken::new(FAKE_LONG_TOKEN).expect("valid");
    assert_eq!(token.len(), TOKEN_LEN_LONG);
    assert_eq!(token.as_str(), FAKE_LONG_TOKEN);
}

#[test]
fn new_accepts_short_form_token() {
    let token = AuthToken::new(FAKE_SHORT_TOKEN).expect("valid");
    assert_eq!(token.len(), TOKEN_LEN_SHORT);
}

#[test]
fn new_accepts_uppercase_hex() {
    assert!(AuthToken::new(FAKE_UPPERCASE_LONG_TOKEN).is_ok());
}

#[test]
fn new_rejects_empty() {
    let err = AuthToken::new("").unwrap_err();
    assert!(matches!(err, AuthTokenError::InvalidLength { .. }));
}

#[test]
fn new_rejects_intermediate_length() {
    let err = AuthToken::new(INVALID_LENGTH_15).unwrap_err();
    assert!(matches!(err, AuthTokenError::InvalidLength { got, .. } if got == INVALID_LENGTH_15.len()));
}

#[test]
fn new_rejects_oversize_length() {
    let err = AuthToken::new(INVALID_LENGTH_33).unwrap_err();
    assert!(matches!(err, AuthTokenError::InvalidLength { .. }));
}

#[test]
fn new_rejects_non_hex_character() {
    let err = AuthToken::new(NON_HEX_INPUT_32).unwrap_err();
    assert_eq!(err, AuthTokenError::NonHex);
}

#[test]
fn is_empty_is_false_for_valid_token() {
    let token = AuthToken::new(FAKE_SHORT_TOKEN).expect("valid");
    assert!(!token.is_empty());
}

#[test]
fn debug_does_not_contain_any_long_hex_run() {
    let token = AuthToken::new(FAKE_LONG_TOKEN).expect("valid");
    let debug = format!("{token:?}");
    assert!(
        !contains_long_hex_run(&debug, MIN_RUN_OF_HEX_THAT_WOULD_LEAK_TOKEN),
        "Debug output leaks token: {debug}"
    );
    assert!(debug.contains(EXPECTED_REDACTION_MARKER));
    assert!(debug.contains("AuthToken"));
}

#[test]
fn clone_preserves_token_value() {
    let a = AuthToken::new(FAKE_LONG_TOKEN).expect("valid");
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(a.as_str(), b.as_str());
}
