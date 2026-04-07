//! [`Transport`] implementation backed by a mounted `PLAUD_NOTE`
//! VFAT volume.
//!
//! Methods that have no USB analogue (battery, record control,
//! settings) return `Error::Unsupported` with the
//! [`crate::constants::CAP_USB_UNSUPPORTED`] capability string so
//! callers can special-case the USB backend if they care. See the
//! [M10 journey](../../../../specs/plaude-cli-v1/journeys/M10-transport-usb.md)
//! for scope and rationale.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use plaud_domain::{BatteryLevel, CommonSettingKey, DeviceInfo, Recording, RecordingId, SettingValue, StorageStats};
use plaud_transport::{Error, Result, Transport};

use crate::{
    constants::{CAP_USB_UNSUPPORTED, MODEL_TXT_FILENAME},
    listing::{ListingError, list_recordings},
    model_txt,
};

/// `Transport` implementation over a mounted Plaud Note VFAT volume.
#[derive(Debug, Clone)]
pub struct UsbTransport {
    root: PathBuf,
}

impl UsbTransport {
    /// Build a transport rooted at `root`, typically the mount point
    /// of the `PLAUD_NOTE` VFAT volume (e.g. `/run/media/alice/PLAUD_NOTE`).
    ///
    /// The root is **not** validated at construction time: the first
    /// `Transport` call that needs it is where missing-mount errors
    /// surface, so callers can defer mount discovery to sync time.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Borrow the mount root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn unsupported<T>() -> Result<T> {
        Err(Error::Unsupported {
            capability: CAP_USB_UNSUPPORTED,
        })
    }
}

#[async_trait]
impl Transport for UsbTransport {
    async fn device_info(&self) -> Result<DeviceInfo> {
        let path = self.root.join(MODEL_TXT_FILENAME);
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| Error::Transport(format!("failed to read {}: {e}", path.display())))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| Error::Protocol(format!("MODEL.txt is not UTF-8: {e}")))?;
        model_txt::parse(text).map_err(|e| Error::Protocol(format!("MODEL.txt parse error: {e}")))
    }

    async fn battery(&self) -> Result<BatteryLevel> {
        Self::unsupported()
    }

    async fn storage(&self) -> Result<StorageStats> {
        Self::unsupported()
    }

    async fn list_recordings(&self) -> Result<Vec<Recording>> {
        let root = self.root.clone();
        let located = tokio::task::spawn_blocking(move || list_recordings(&root))
            .await
            .map_err(|e| Error::Transport(format!("usb listing task join failed: {e}")))?
            .map_err(map_listing_error)?;
        Ok(located.into_iter().map(|loc| loc.meta).collect())
    }

    async fn read_recording(&self, id: &RecordingId) -> Result<Vec<u8>> {
        let located = self.locate(id).await?;
        tokio::fs::read(&located.wav_path)
            .await
            .map_err(|e| Error::Transport(format!("failed to read {}: {e}", located.wav_path.display())))
    }

    async fn read_recording_asr(&self, id: &RecordingId) -> Result<Vec<u8>> {
        let located = self.locate(id).await?;
        tokio::fs::read(&located.asr_path)
            .await
            .map_err(|e| Error::Transport(format!("failed to read {}: {e}", located.asr_path.display())))
    }

    async fn delete_recording(&self, _id: &RecordingId) -> Result<()> {
        Self::unsupported()
    }

    async fn read_setting(&self, _key: CommonSettingKey) -> Result<SettingValue> {
        Self::unsupported()
    }

    async fn write_setting(&self, _key: CommonSettingKey, _value: SettingValue) -> Result<()> {
        Self::unsupported()
    }

    async fn start_recording(&self) -> Result<()> {
        Self::unsupported()
    }

    async fn stop_recording(&self) -> Result<()> {
        Self::unsupported()
    }

    async fn pause_recording(&self) -> Result<()> {
        Self::unsupported()
    }

    async fn resume_recording(&self) -> Result<()> {
        Self::unsupported()
    }

    async fn set_privacy(&self, _on: bool) -> Result<()> {
        Self::unsupported()
    }
}

impl UsbTransport {
    async fn locate(&self, id: &RecordingId) -> Result<crate::listing::RecordingLocation> {
        let root = self.root.clone();
        let located = tokio::task::spawn_blocking(move || list_recordings(&root))
            .await
            .map_err(|e| Error::Transport(format!("usb listing task join failed: {e}")))?
            .map_err(map_listing_error)?;
        located
            .into_iter()
            .find(|loc| loc.meta.id() == id)
            .ok_or_else(|| Error::NotFound(id.as_str().to_owned()))
    }
}

fn map_listing_error(err: ListingError) -> Error {
    match err {
        ListingError::Io { path, source } => Error::Transport(format!("usb listing io at {}: {source}", path.display())),
        ListingError::InvalidRecordingId(e) => Error::Protocol(format!("usb listing invalid id: {e}")),
    }
}
