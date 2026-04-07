//! Tests for [`plaud_auth::ChainStore`] fallback semantics. Uses two
//! `FileStore`s backed by separate tempdirs as the primary and
//! secondary so no keyring daemon is required.
//!
//! Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md

use plaud_auth::{ChainStore, DEFAULT_DEVICE_ID, FileStore};
use plaud_domain::AuthToken;
use plaud_transport::AuthStore;
use tempfile::TempDir;

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";

fn paired_chain() -> (TempDir, TempDir, ChainStore) {
    let primary_tmp = tempfile::tempdir().expect("primary tempdir");
    let secondary_tmp = tempfile::tempdir().expect("secondary tempdir");
    let primary = Box::new(FileStore::new(primary_tmp.path().join("token")));
    let secondary = Box::new(FileStore::new(secondary_tmp.path().join("token")));
    (primary_tmp, secondary_tmp, ChainStore::new(primary, secondary))
}

fn token() -> AuthToken {
    AuthToken::new(SAMPLE_TOKEN).expect("hand-validated")
}

#[tokio::test]
async fn put_writes_to_primary_so_get_reads_from_primary() {
    let (_a, _b, chain) = paired_chain();
    chain.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    let got = chain.get_token(DEFAULT_DEVICE_ID).await.expect("get").expect("some");
    assert_eq!(got.as_str(), SAMPLE_TOKEN);
}

#[tokio::test]
async fn get_falls_through_to_secondary_when_primary_has_nothing() {
    // Hand-write a token file to the secondary's path only.
    let primary_tmp = tempfile::tempdir().expect("primary tempdir");
    let secondary_tmp = tempfile::tempdir().expect("secondary tempdir");
    let primary = Box::new(FileStore::new(primary_tmp.path().join("token")));
    let secondary_store = FileStore::new(secondary_tmp.path().join("token"));
    secondary_store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("seed secondary");
    let chain = ChainStore::new(primary, Box::new(secondary_store));
    let got = chain.get_token(DEFAULT_DEVICE_ID).await.expect("get").expect("secondary hit");
    assert_eq!(got.as_str(), SAMPLE_TOKEN);
}

#[tokio::test]
async fn remove_clears_both_backends() {
    let (_a, _b, chain) = paired_chain();
    chain.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    chain.remove_token(DEFAULT_DEVICE_ID).await.expect("remove");
    let got = chain.get_token(DEFAULT_DEVICE_ID).await.expect("get");
    assert!(got.is_none());
}

#[tokio::test]
async fn remove_is_idempotent_on_empty_chain() {
    let (_a, _b, chain) = paired_chain();
    chain.remove_token(DEFAULT_DEVICE_ID).await.expect("first remove");
    chain.remove_token(DEFAULT_DEVICE_ID).await.expect("second remove");
}
