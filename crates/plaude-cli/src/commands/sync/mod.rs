//! `plaude sync <dir>` — mirror every recording on the device
//! into a local directory with a JSON state file for idempotence.

pub(crate) mod state;

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Args;
use plaud_auth::DEFAULT_DEVICE_ID;
use plaud_domain::{AuthToken, Recording, RecordingId};
use plaud_transport::Transport;
use plaud_transport_usb::WavSanitiser;

use crate::{
    DispatchError,
    commands::{
        auth::build_store,
        backend::TransportProvider,
        sync::state::{RecordingEntry, SyncState, inventory_hash},
    },
};

/// Extension for the stereo PCM file. Kept in sync with the
/// constant in `commands/files.rs` — they share the same filesystem
/// layout contract.
const WAV_EXTENSION: &str = "wav";
/// Extension for the mono Opus sidecar.
const ASR_EXTENSION: &str = "asr";
/// Default value for `--concurrency`. BLE is serial, so higher
/// values are accepted but have no effect in M9.
const DEFAULT_CONCURRENCY: u32 = 1;
/// Message printed when the sync plan is empty.
const NOTHING_TO_DO_MSG: &str = "nothing to do";
/// Prefix for `--dry-run` plan lines.
const DRY_RUN_LINE_PREFIX: &str = "would pull:";
/// Prefix for the deleted-on-device warning printed to stderr.
const DELETED_ON_DEVICE_PREFIX: &str = "deleted on device (still on disk):";
/// Prefix for successfully-pulled progress lines.
const PULLED_LINE_PREFIX: &str = "pulled";

/// Arguments for `plaude sync`.
#[derive(Debug, Args)]
pub(crate) struct SyncArgs {
    /// Destination directory. Created if missing.
    dir: PathBuf,
    /// Print the plan without pulling anything.
    #[arg(long)]
    dry_run: bool,
    /// Accepted for future use; BLE is serial in M9 so the value is
    /// not currently consulted. Documented behaviour: the default
    /// `1` is exact; other values are accepted silently so scripts
    /// written for later milestones keep working.
    #[arg(long, default_value_t = DEFAULT_CONCURRENCY)]
    concurrency: u32,
    /// Zero the `SN:<serial>` region of every pulled WAV before
    /// writing it to disk. Prevents the on-device forensic serial
    /// watermark from leaving the host. Only meaningful on the USB
    /// backend; a no-op on sources whose WAVs do not carry the
    /// `pad ` chunk.
    #[arg(long)]
    sanitise: bool,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) async fn run(args: SyncArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let _ = args.concurrency; // reserved; see struct doc
    tokio::fs::create_dir_all(&args.dir)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to create sync dir {}: {e}", args.dir.display())))?;
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    let recordings = transport
        .list_recordings()
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    let mut state = SyncState::load(&args.dir).await?;
    let plan = Plan::compute(&recordings, &state, &args.dir).await;
    report_deleted(&plan.deleted_on_device);
    if args.dry_run {
        print_dry_run(&plan);
        return Ok(());
    }
    if plan.is_noop(&recordings, &state) {
        println!("{NOTHING_TO_DO_MSG}");
        return Ok(());
    }
    for id in &plan.to_pull {
        pull_recording_into(transport.as_ref(), &recordings, id, &args.dir, args.sanitise, &mut state).await?;
        state.save(&args.dir).await?;
    }
    prune_deleted_from_state(&mut state, &plan.deleted_on_device);
    state.inventory_hash = inventory_hash(&recordings);
    state.save(&args.dir).await?;
    Ok(())
}

#[derive(Debug)]
struct Plan {
    to_pull: Vec<RecordingId>,
    deleted_on_device: Vec<String>,
}

impl Plan {
    async fn compute(recordings: &[Recording], state: &SyncState, dir: &Path) -> Self {
        let mut to_pull: Vec<RecordingId> = Vec::new();
        for r in recordings {
            let id_str = r.id().as_str();
            let state_entry = state.recordings.get(id_str);
            let wav_present = file_has_expected_size(&dir.join(format!("{id_str}.{WAV_EXTENSION}")), r.wav_size()).await;
            let asr_present = file_has_expected_size(&dir.join(format!("{id_str}.{ASR_EXTENSION}")), r.asr_size()).await;
            let state_entry_matches = state_entry
                .map(|e| e.wav_size == r.wav_size() && e.asr_size == r.asr_size())
                .unwrap_or(false);
            if !(state_entry_matches && wav_present && asr_present) {
                to_pull.push(r.id().clone());
            }
        }
        let current_ids: std::collections::HashSet<&str> = recordings.iter().map(|r| r.id().as_str()).collect();
        let deleted_on_device: Vec<String> = state
            .recordings
            .keys()
            .filter(|id| !current_ids.contains(id.as_str()))
            .cloned()
            .collect();
        Self {
            to_pull,
            deleted_on_device,
        }
    }

    fn is_noop(&self, recordings: &[Recording], state: &SyncState) -> bool {
        self.to_pull.is_empty() && self.deleted_on_device.is_empty() && state.inventory_hash == inventory_hash(recordings)
    }
}

async fn file_has_expected_size(path: &Path, expected: u64) -> bool {
    tokio::fs::metadata(path).await.map(|m| m.len() == expected).unwrap_or(false)
}

async fn pull_recording_into(
    transport: &dyn Transport,
    listing: &[Recording],
    id: &RecordingId,
    dir: &Path,
    sanitise: bool,
    state: &mut SyncState,
) -> Result<(), DispatchError> {
    let meta = listing
        .iter()
        .find(|r| r.id() == id)
        .ok_or_else(|| DispatchError::Runtime(format!("recording {} disappeared mid-sync", id.as_str())))?;
    let wav_path = dir.join(format!("{}.{WAV_EXTENSION}", id.as_str()));
    let asr_path = dir.join(format!("{}.{ASR_EXTENSION}", id.as_str()));
    let mut wav_bytes = transport
        .read_recording(id)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    if sanitise {
        let _ = WavSanitiser::new().sanitise(&mut wav_bytes);
    }
    tokio::fs::write(&wav_path, &wav_bytes)
        .await
        .map_err(|e| DispatchError::Runtime(format!("write {}: {e}", wav_path.display())))?;
    let asr_bytes = transport
        .read_recording_asr(id)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    tokio::fs::write(&asr_path, &asr_bytes)
        .await
        .map_err(|e| DispatchError::Runtime(format!("write {}: {e}", asr_path.display())))?;
    println!("{PULLED_LINE_PREFIX} {}", id.as_str());
    state.recordings.insert(
        id.as_str().to_owned(),
        RecordingEntry {
            wav_size: meta.wav_size(),
            asr_size: meta.asr_size(),
            pulled_at_unix_seconds: now_unix_seconds(),
        },
    );
    Ok(())
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn report_deleted(deleted: &[String]) {
    for id in deleted {
        eprintln!("{DELETED_ON_DEVICE_PREFIX} {id}");
    }
}

fn print_dry_run(plan: &Plan) {
    for id in &plan.to_pull {
        println!("{DRY_RUN_LINE_PREFIX} {}", id.as_str());
    }
    if plan.to_pull.is_empty() {
        println!("{NOTHING_TO_DO_MSG}");
    }
}

fn prune_deleted_from_state(state: &mut SyncState, deleted: &[String]) {
    for id in deleted {
        state.recordings.remove(id);
    }
}

async fn load_token(config_dir: Option<&Path>) -> Result<AuthToken, DispatchError> {
    let store = build_store(config_dir)?;
    let stored = store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read token store: {e}")))?;
    stored.ok_or(DispatchError::AuthRequired)
}

#[cfg(test)]
mod tests {
    use super::{DRY_RUN_LINE_PREFIX, NOTHING_TO_DO_MSG, PULLED_LINE_PREFIX, WAV_EXTENSION};

    #[test]
    fn text_constants_are_stable() {
        // Mutation-kill: a rename of these breaks downstream scripts
        // that grep sync output.
        assert_eq!(WAV_EXTENSION, "wav");
        assert!(DRY_RUN_LINE_PREFIX.starts_with("would pull"));
        assert!(NOTHING_TO_DO_MSG.contains("nothing"));
        assert!(PULLED_LINE_PREFIX.starts_with("pulled"));
    }
}
