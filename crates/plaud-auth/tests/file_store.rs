//! Tests for [`plaud_auth::FileStore`].
//!
//! All tests use a per-test `tempfile::TempDir` to avoid touching
//! the user's real `~/.config/plaude/token`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md

use plaud_auth::{DEFAULT_DEVICE_ID, FileStore};
use plaud_domain::AuthToken;
use plaud_transport::AuthStore;
use tempfile::TempDir;

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const OVERWRITE_TOKEN: &str = "deadbeefcafebabe1234567890abcdef";
const TOKEN_FILE_NAME: &str = "token";
#[cfg(unix)]
const EXPECTED_FILE_MODE: u32 = 0o600;
#[cfg(unix)]
const EXPECTED_PARENT_MODE: u32 = 0o700;

fn sandbox() -> (TempDir, FileStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("plaude").join(TOKEN_FILE_NAME);
    (tmp, FileStore::new(path))
}

fn token() -> AuthToken {
    AuthToken::new(SAMPLE_TOKEN).expect("hand-validated")
}

#[tokio::test]
async fn get_on_empty_store_returns_none() {
    let (_tmp, store) = sandbox();
    let result = store.get_token(DEFAULT_DEVICE_ID).await.expect("get");
    assert!(result.is_none());
}

#[tokio::test]
async fn put_then_get_round_trips_the_token() {
    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    let got = store.get_token(DEFAULT_DEVICE_ID).await.expect("get");
    assert_eq!(got.as_ref().map(|t| t.as_str().to_owned()), Some(SAMPLE_TOKEN.to_owned()));
}

#[tokio::test]
async fn put_creates_parent_dir_when_missing() {
    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    let parent = store.path().parent().expect("has parent");
    assert!(parent.is_dir(), "parent directory should exist: {parent:?}");
}

#[tokio::test]
async fn put_overwrites_an_existing_token_file() {
    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("first put");
    let new_token = AuthToken::new(OVERWRITE_TOKEN).expect("hand-validated");
    store.put_token(DEFAULT_DEVICE_ID, new_token).await.expect("second put");
    let got = store.get_token(DEFAULT_DEVICE_ID).await.expect("get").expect("some");
    assert_eq!(got.as_str(), OVERWRITE_TOKEN);
}

#[tokio::test]
async fn remove_on_empty_store_is_idempotent() {
    let (_tmp, store) = sandbox();
    store.remove_token(DEFAULT_DEVICE_ID).await.expect("first remove");
    store.remove_token(DEFAULT_DEVICE_ID).await.expect("second remove");
}

#[tokio::test]
async fn remove_deletes_the_token_file_after_put() {
    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    store.remove_token(DEFAULT_DEVICE_ID).await.expect("remove");
    let got = store.get_token(DEFAULT_DEVICE_ID).await.expect("get");
    assert!(got.is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn put_sets_token_file_mode_to_0600_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    let metadata = std::fs::metadata(store.path()).expect("stat");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, EXPECTED_FILE_MODE);
}

#[cfg(unix)]
#[tokio::test]
async fn put_sets_parent_dir_mode_to_0700_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let (_tmp, store) = sandbox();
    store.put_token(DEFAULT_DEVICE_ID, token()).await.expect("put");
    let parent = store.path().parent().expect("has parent");
    let metadata = std::fs::metadata(parent).expect("stat");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, EXPECTED_PARENT_MODE);
}
