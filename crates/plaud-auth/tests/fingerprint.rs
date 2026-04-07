//! Tests for [`plaud_auth::fingerprint::token_fingerprint`].
//!
//! Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md

use plaud_auth::token_fingerprint;
use plaud_domain::AuthToken;

const SAMPLE_TOKEN_A: &str = "0123456789abcdef0123456789abcdef";
const SAMPLE_TOKEN_B: &str = "fedcba9876543210fedcba9876543210";
const FINGERPRINT_LEN: usize = 16;

fn token(raw: &str) -> AuthToken {
    AuthToken::new(raw).expect("test token is hand-validated")
}

#[test]
fn fingerprint_has_exactly_sixteen_hex_characters() {
    let fp = token_fingerprint(&token(SAMPLE_TOKEN_A));
    assert_eq!(fp.len(), FINGERPRINT_LEN);
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn fingerprint_is_deterministic_for_the_same_token() {
    let a = token_fingerprint(&token(SAMPLE_TOKEN_A));
    let b = token_fingerprint(&token(SAMPLE_TOKEN_A));
    assert_eq!(a, b);
}

#[test]
fn different_tokens_produce_different_fingerprints() {
    let a = token_fingerprint(&token(SAMPLE_TOKEN_A));
    let b = token_fingerprint(&token(SAMPLE_TOKEN_B));
    assert_ne!(a, b);
}

#[test]
fn fingerprint_does_not_contain_any_substring_of_the_raw_token() {
    // SAMPLE_TOKEN_A starts with `0123456789abcdef`; assert the
    // fingerprint does not contain a 10+ hex-digit run that matches
    // that prefix. This kills the mutant that accidentally prints
    // part of the token alongside the hash.
    let fp = token_fingerprint(&token(SAMPLE_TOKEN_A));
    assert!(!fp.contains("0123456789"));
}
