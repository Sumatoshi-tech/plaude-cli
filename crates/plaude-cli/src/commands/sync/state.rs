//! State file for `plaude-cli sync`.
//!
//! Lives at `<dir>/.plaude-sync.json`. Records which recordings have
//! landed on disk alongside the SHA-256 hash of the last-observed
//! device inventory so an idempotent re-run can early-exit without
//! re-pulling anything.

use std::{collections::BTreeMap, path::Path};

use plaud_domain::Recording;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::DispatchError;

/// Current state-file schema version. Bumped if a breaking change
/// ships; older files are rejected with a clear error pointing the
/// user at a manual migration.
pub(crate) const STATE_FILE_VERSION: u32 = 1;

/// Filename (under the sync destination directory) for the
/// persisted state.
pub(crate) const STATE_FILE_NAME: &str = ".plaude-sync.json";

/// On-disk sync state. Serialised as JSON via `serde_json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SyncState {
    pub(crate) version: u32,
    pub(crate) inventory_hash: String,
    pub(crate) recordings: BTreeMap<String, RecordingEntry>,
}

/// One row in [`SyncState::recordings`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordingEntry {
    pub(crate) wav_size: u64,
    pub(crate) asr_size: u64,
    pub(crate) pulled_at_unix_seconds: i64,
}

impl SyncState {
    pub(crate) fn empty() -> Self {
        Self {
            version: STATE_FILE_VERSION,
            inventory_hash: String::new(),
            recordings: BTreeMap::new(),
        }
    }

    /// Load the state file from `dir`, returning an empty state if
    /// the file does not exist yet. Schema-version mismatches are
    /// surfaced as a usage error.
    pub(crate) async fn load(dir: &Path) -> Result<Self, DispatchError> {
        let path = dir.join(STATE_FILE_NAME);
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let parsed: Self =
                    serde_json::from_slice(&bytes).map_err(|e| DispatchError::Runtime(format!("failed to parse state file: {e}")))?;
                if parsed.version != STATE_FILE_VERSION {
                    return Err(DispatchError::Usage(format!(
                        "state file version {} is not understood by this CLI (expected {STATE_FILE_VERSION}); remove {} to start fresh",
                        parsed.version,
                        path.display()
                    )));
                }
                Ok(parsed)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(e) => Err(DispatchError::Runtime(format!("failed to read state file: {e}"))),
        }
    }

    /// Atomically persist the state file under `dir`. Writes to a
    /// sibling `.tmp` file then renames, so a crash mid-write cannot
    /// leave the state truncated.
    pub(crate) async fn save(&self, dir: &Path) -> Result<(), DispatchError> {
        let path = dir.join(STATE_FILE_NAME);
        let mut tmp = path.clone();
        let original_file_name = path
            .file_name()
            .ok_or_else(|| DispatchError::Runtime("state file path has no filename".to_owned()))?
            .to_string_lossy()
            .into_owned();
        tmp.set_file_name(format!("{original_file_name}.tmp"));
        let rendered = serde_json::to_vec_pretty(self).map_err(|e| DispatchError::Runtime(format!("serialise state: {e}")))?;
        tokio::fs::write(&tmp, &rendered)
            .await
            .map_err(|e| DispatchError::Runtime(format!("write state tmp {}: {e}", tmp.display())))?;
        tokio::fs::rename(&tmp, &path)
            .await
            .map_err(|e| DispatchError::Runtime(format!("rename state {} → {}: {e}", tmp.display(), path.display())))?;
        Ok(())
    }
}

/// SHA-256 over `(id, wav_size, asr_size)` triples sorted by id.
/// Stable across reorderings, changes on any content delta. Used
/// to short-circuit a no-op re-run in one hash comparison instead
/// of walking every entry.
pub(crate) fn inventory_hash(recordings: &[Recording]) -> String {
    let mut sorted: Vec<&Recording> = recordings.iter().collect();
    sorted.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    let mut hasher = Sha256::new();
    for r in sorted {
        hasher.update(r.id().as_str().as_bytes());
        hasher.update(r.wav_size().to_le_bytes());
        hasher.update(r.asr_size().to_le_bytes());
        hasher.update([0x00]);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use plaud_domain::{Recording, RecordingId, RecordingKind};

    use super::{RecordingEntry, STATE_FILE_VERSION, SyncState, inventory_hash};

    const BASENAME_A: &str = "1775393534";
    const BASENAME_B: &str = "1775393540";

    fn make_recording(basename: &str, wav: u64, asr: u64) -> Recording {
        Recording::new(RecordingId::new(basename).expect("valid"), RecordingKind::Note, wav, asr)
    }

    #[test]
    fn inventory_hash_is_stable_under_reordering() {
        let h1 = inventory_hash(&[make_recording(BASENAME_A, 10, 20), make_recording(BASENAME_B, 30, 40)]);
        let h2 = inventory_hash(&[make_recording(BASENAME_B, 30, 40), make_recording(BASENAME_A, 10, 20)]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn inventory_hash_changes_when_a_size_changes() {
        let h1 = inventory_hash(&[make_recording(BASENAME_A, 10, 20)]);
        let h2 = inventory_hash(&[make_recording(BASENAME_A, 11, 20)]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn inventory_hash_changes_when_a_new_recording_is_added() {
        let h1 = inventory_hash(&[make_recording(BASENAME_A, 10, 20)]);
        let h2 = inventory_hash(&[make_recording(BASENAME_A, 10, 20), make_recording(BASENAME_B, 30, 40)]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn sync_state_empty_has_current_schema_version() {
        let state = SyncState::empty();
        assert_eq!(state.version, STATE_FILE_VERSION);
        assert!(state.recordings.is_empty());
        assert!(state.inventory_hash.is_empty());
    }

    #[test]
    fn sync_state_json_round_trip_preserves_every_field() {
        let mut entries = BTreeMap::new();
        entries.insert(
            BASENAME_A.to_owned(),
            RecordingEntry {
                wav_size: 10,
                asr_size: 20,
                pulled_at_unix_seconds: 1775393534,
            },
        );
        let original = SyncState {
            version: STATE_FILE_VERSION,
            inventory_hash: "abcdef0123456789".to_owned(),
            recordings: entries,
        };
        let rendered = serde_json::to_string(&original).expect("ser");
        let parsed: SyncState = serde_json::from_str(&rendered).expect("de");
        assert_eq!(parsed, original);
    }
}
