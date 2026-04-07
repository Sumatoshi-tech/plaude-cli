//! Recording enumeration over a mounted `PLAUD_NOTE` VFAT volume.
//!
//! Walks `{NOTES,CALLS}/<YYYYMMDD>/<unix>.{WAV,ASR}` and pairs each
//! WAV with its co-located ASR sidecar. A recording is only
//! surfaced when **both** files exist, matching the
//! `docs/protocol/file-formats.md` "atomic pair" contract.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use plaud_domain::{Recording, RecordingId, RecordingIdError, RecordingKind};
use thiserror::Error;

use crate::constants::{ASR_EXTENSION, CALLS_DIR, NOTES_DIR, WAV_EXTENSION};

/// Errors produced by [`list_recordings`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ListingError {
    /// Filesystem I/O error while walking the volume.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path that triggered the error, so callers can surface it.
        path: PathBuf,
        /// Wrapped std I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A filename basename (the part before the extension) was
    /// rejected by [`RecordingId::new`].
    #[error("invalid recording id in filename: {0}")]
    InvalidRecordingId(#[from] RecordingIdError),
}

/// Walk every recording directory under `root` and return the
/// discovered `(Recording, wav_path, asr_path)` triples sorted by id.
pub fn list_recordings(root: &Path) -> Result<Vec<RecordingLocation>, ListingError> {
    let mut found: BTreeMap<String, PartialEntry> = BTreeMap::new();
    for (subdir, kind) in [(NOTES_DIR, RecordingKind::Note), (CALLS_DIR, RecordingKind::Call)] {
        let root_dir = root.join(subdir);
        if !root_dir.exists() {
            continue;
        }
        walk_kind(&root_dir, kind, &mut found)?;
    }
    let mut out = Vec::new();
    for (id_str, partial) in found {
        let (Some(wav_path), Some(asr_path)) = (partial.wav_path, partial.asr_path) else {
            // Skip unpaired entries per the atomic-pair contract.
            continue;
        };
        let wav_size = std::fs::metadata(&wav_path)
            .map_err(|e| ListingError::Io {
                path: wav_path.clone(),
                source: e,
            })?
            .len();
        let asr_size = std::fs::metadata(&asr_path)
            .map_err(|e| ListingError::Io {
                path: asr_path.clone(),
                source: e,
            })?
            .len();
        let id = RecordingId::new(id_str)?;
        out.push(RecordingLocation {
            meta: Recording::new(id, partial.kind, wav_size, asr_size),
            wav_path,
            asr_path,
        });
    }
    Ok(out)
}

/// A discovered recording plus the absolute paths of its paired
/// files on the host filesystem.
#[derive(Debug, Clone)]
pub struct RecordingLocation {
    /// The `Recording` domain value the caller will return to
    /// `Transport::list_recordings` consumers.
    pub meta: Recording,
    /// Absolute filesystem path of the `.WAV` file.
    pub wav_path: PathBuf,
    /// Absolute filesystem path of the `.ASR` sidecar.
    pub asr_path: PathBuf,
}

struct PartialEntry {
    kind: RecordingKind,
    wav_path: Option<PathBuf>,
    asr_path: Option<PathBuf>,
}

fn walk_kind(root: &Path, kind: RecordingKind, acc: &mut BTreeMap<String, PartialEntry>) -> Result<(), ListingError> {
    let day_iter = std::fs::read_dir(root).map_err(|e| ListingError::Io {
        path: root.to_path_buf(),
        source: e,
    })?;
    for day_entry in day_iter {
        let day_entry = day_entry.map_err(|e| ListingError::Io {
            path: root.to_path_buf(),
            source: e,
        })?;
        let day_path = day_entry.path();
        if !day_path.is_dir() {
            continue;
        }
        walk_day(&day_path, kind, acc)?;
    }
    Ok(())
}

fn walk_day(day_path: &Path, kind: RecordingKind, acc: &mut BTreeMap<String, PartialEntry>) -> Result<(), ListingError> {
    let file_iter = std::fs::read_dir(day_path).map_err(|e| ListingError::Io {
        path: day_path.to_path_buf(),
        source: e,
    })?;
    for file_entry in file_iter {
        let file_entry = file_entry.map_err(|e| ListingError::Io {
            path: day_path.to_path_buf(),
            source: e,
        })?;
        let path = file_entry.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let entry = acc.entry(stem.to_owned()).or_insert(PartialEntry {
            kind,
            wav_path: None,
            asr_path: None,
        });
        if ext.eq_ignore_ascii_case(WAV_EXTENSION) {
            entry.wav_path = Some(path);
        } else if ext.eq_ignore_ascii_case(ASR_EXTENSION) {
            entry.asr_path = Some(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::list_recordings;
    use crate::constants::{ASR_EXTENSION, CALLS_DIR, NOTES_DIR, WAV_EXTENSION};

    const BASENAME: &str = "1775393534";
    const DATE_DIR: &str = "20260405";
    const WAV_BYTES: &[u8] = b"WAV-BODY";
    const ASR_BYTES: &[u8] = b"ASR-BODY";

    fn write_pair(root: &std::path::Path, kind_dir: &str, basename: &str) {
        let dir = root.join(kind_dir).join(DATE_DIR);
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join(format!("{basename}.{WAV_EXTENSION}")), WAV_BYTES).expect("wav");
        fs::write(dir.join(format!("{basename}.{ASR_EXTENSION}")), ASR_BYTES).expect("asr");
    }

    #[test]
    fn list_recordings_pairs_wav_and_asr_in_notes() {
        let tmp = TempDir::new().expect("tmp");
        write_pair(tmp.path(), NOTES_DIR, BASENAME);
        let list = list_recordings(tmp.path()).expect("walk");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].meta.id().as_str(), BASENAME);
        assert_eq!(list[0].meta.wav_size(), WAV_BYTES.len() as u64);
        assert_eq!(list[0].meta.asr_size(), ASR_BYTES.len() as u64);
    }

    #[test]
    fn list_recordings_walks_both_notes_and_calls() {
        let tmp = TempDir::new().expect("tmp");
        write_pair(tmp.path(), NOTES_DIR, "1775393534");
        write_pair(tmp.path(), CALLS_DIR, "1775393540");
        let list = list_recordings(tmp.path()).expect("walk");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn list_recordings_skips_wav_without_asr_sidecar() {
        let tmp = TempDir::new().expect("tmp");
        let dir = tmp.path().join(NOTES_DIR).join(DATE_DIR);
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join(format!("{BASENAME}.{WAV_EXTENSION}")), WAV_BYTES).expect("wav");
        // No ASR sidecar.
        let list = list_recordings(tmp.path()).expect("walk");
        assert!(list.is_empty());
    }

    #[test]
    fn list_recordings_on_empty_root_returns_empty_vec() {
        let tmp = TempDir::new().expect("tmp");
        let list = list_recordings(tmp.path()).expect("walk");
        assert!(list.is_empty());
    }
}
