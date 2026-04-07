//! USB Mass Storage fallback transport for Plaud devices.
//!
//! When the Plaud Note's "Access via USB" toggle is enabled in the
//! phone app, the device enumerates as a VFAT mass-storage volume
//! labelled `PLAUD_NOTE` with `NOTES/`, `CALLS/`, and `MODEL.txt`.
//! This crate implements the read-only subset of
//! `plaud_transport::Transport` against that volume and offers a
//! [`WavSanitiser`] helper that zeros the device-serial bytes inside
//! each WAV's custom `pad ` RIFF chunk.
//!
//! **Deprecated by vendor.** An in-app warning string confirms Plaud
//! will disable USB file access in a firmware update — see
//! `specs/re/apk-notes/3.14.0-620/auth-token.md`. Treat this transport
//! as a convenience fallback for pre-deprecation firmware only.
//!
//! See the M10 journey at
//! `specs/plaude-cli-v1/journeys/M10-transport-usb.md` for scope,
//! deferrals, and test plan.

pub mod constants;
pub mod listing;
pub mod model_txt;
pub mod transport;
pub mod wav;

pub use constants::{CAP_USB_UNSUPPORTED, USB_DEPRECATION_NOTICE};
pub use listing::{ListingError, RecordingLocation, list_recordings};
pub use model_txt::{ModelTxtError, parse as parse_model_txt};
pub use transport::UsbTransport;
pub use wav::{WavSanitiseError, WavSanitiser};
