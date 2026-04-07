//! Pure domain types for plaude-cli.
//!
//! This crate defines the vocabulary the CLI speaks in: [`Recording`],
//! [`DeviceInfo`], [`BatteryLevel`], [`CommonSettingKey`], and friends.
//! It performs no I/O and has no dependencies on any transport crate.
//!
//! Every type carries an evidence citation in its doc comment back to
//! the reverse-engineered protocol spec under `docs/protocol/` and the
//! captures under `specs/re/captures/`. See
//! [`specs/plaude-cli-v1/journeys/M01-domain-traits.md`](../../../specs/plaude-cli-v1/journeys/M01-domain-traits.md)
//! for the milestone context.
//!
//! ## Module layout
//!
//! * [`recording`] — [`RecordingId`], [`RecordingKind`], [`Recording`].
//! * [`device`] — [`DeviceSerial`] (redacting `Debug`), [`DeviceModel`],
//!   [`FirmwareVersion`], [`DeviceInfo`].
//! * [`battery`] — [`BatteryLevel`].
//! * [`storage`] — [`StorageStats`].
//! * [`setting`] — [`CommonSettingKey`], [`SettingValue`], [`Setting`].
//! * [`discovery`] — [`TransportHint`], [`DeviceCandidate`].
//! * [`auth`] — [`AuthToken`].
//!
//! ## Privacy
//!
//! Two types in this crate have **redacting `Debug` impls** by design:
//! [`DeviceSerial`] (to avoid leaking the forensic serial Plaud writes
//! into every `.WAV` file) and [`AuthToken`] (to avoid leaking the
//! BLE vendor pre-shared key through logs). Code that genuinely needs
//! the raw value must call [`DeviceSerial::reveal`] or
//! [`AuthToken::as_str`], both of which are grep-visible at review time.

pub mod auth;
pub mod battery;
pub mod device;
pub mod discovery;
pub mod recording;
pub mod setting;
pub mod storage;

pub use auth::{AuthToken, AuthTokenError};
pub use battery::{BatteryLevel, BatteryLevelError};
pub use device::{DeviceInfo, DeviceModel, DeviceSerial, DeviceSerialError, FirmwareVersion, FirmwareVersionError};
pub use discovery::{DeviceCandidate, TransportHint};
pub use recording::{Recording, RecordingId, RecordingIdError, RecordingKind};
pub use setting::{CommonSettingKey, Setting, SettingValue, SettingValueParseError, UnknownSettingCode, UnknownSettingName};
pub use storage::{StorageStats, StorageStatsError};
