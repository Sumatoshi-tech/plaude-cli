//! Device identity types: serial, model, firmware version, device info.
//!
//! `DeviceSerial` is the only type in the crate whose `Debug` impl
//! redacts by design — Plaud writes the 18-digit serial into every
//! `.WAV` file's `pad ` RIFF chunk, so leaking it through a log line
//! would undo the CLI's forensic-sanitisation guarantees. See
//! `docs/protocol/file-formats.md` for the watermark details.

use std::fmt;

use thiserror::Error;

/// Minimum accepted length of a device serial string.
const DEVICE_SERIAL_MIN_LEN: usize = 8;

/// Maximum accepted length of a device serial string.
const DEVICE_SERIAL_MAX_LEN: usize = 32;

/// `MODEL.txt` always prefixes the build number with an uppercase `V`.
/// Example line: `PLAUD NOTE V0095@00:47:14 Feb 28 2024`.
const FIRMWARE_BUILD_PREFIX: char = 'V';

/// Separator between the build number and the build-time stamp inside
/// the `MODEL.txt` version token.
const FIRMWARE_BUILD_STAMP_SEPARATOR: char = '@';

/// Validation errors produced by [`DeviceSerial::new`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceSerialError {
    /// Input was the empty string.
    #[error("device serial must not be empty")]
    Empty,
    /// Input contained a byte that is not an ASCII decimal digit.
    #[error("device serial must be ASCII digits only")]
    NonDigit,
    /// Input length was outside the valid range.
    #[error("device serial length {got} is outside the valid range [{min}..={max}]")]
    InvalidLength {
        /// Observed length.
        got: usize,
        /// Inclusive minimum.
        min: usize,
        /// Inclusive maximum.
        max: usize,
    },
}

/// An 18-ish-digit device serial. The observed format on a V0095 Plaud
/// Note is exactly 18 ASCII decimal digits, but we accept a wider range
/// to be tolerant of future device generations.
///
/// # Privacy
///
/// The [`fmt::Debug`] impl on this type intentionally **does not**
/// include the raw serial. The serial is considered forensically
/// sensitive because Plaud embeds it into every `.WAV` file's `pad `
/// RIFF subchunk (see [`docs/protocol/file-formats.md`]). Code that
/// needs the raw value must go through [`Self::reveal`] explicitly so
/// it is greppable at review time.
///
/// [`docs/protocol/file-formats.md`]: ../../../docs/protocol/file-formats.md
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeviceSerial(String);

impl DeviceSerial {
    /// Construct a `DeviceSerial`, validating length and character class.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceSerialError`] if the input is empty, contains a
    /// non-digit byte, or has a length outside the accepted range.
    pub fn new(input: impl Into<String>) -> Result<Self, DeviceSerialError> {
        let raw = input.into();
        if raw.is_empty() {
            return Err(DeviceSerialError::Empty);
        }
        let len = raw.len();
        if !(DEVICE_SERIAL_MIN_LEN..=DEVICE_SERIAL_MAX_LEN).contains(&len) {
            return Err(DeviceSerialError::InvalidLength {
                got: len,
                min: DEVICE_SERIAL_MIN_LEN,
                max: DEVICE_SERIAL_MAX_LEN,
            });
        }
        if !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(DeviceSerialError::NonDigit);
        }
        Ok(Self(raw))
    }

    /// Expose the raw serial.
    ///
    /// This is the only accessor that returns the unredacted string.
    /// Use it sparingly: anything that reaches logs or user output
    /// should stay behind the `Debug` impl.
    #[must_use]
    pub fn reveal(&self) -> &str {
        &self.0
    }

    /// Length of the underlying serial in bytes. Useful for `Debug`
    /// output that wants to show "we have a serial" without leaking
    /// its content.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the underlying serial is empty. Structurally always
    /// `false` for a constructed `DeviceSerial` (the constructor
    /// rejects empty input), provided for clippy's `len_without_is_empty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// A placeholder serial suitable for simulators and tests. Never
    /// matches a real Plaud device. Exactly 8 ASCII zeros, so the
    /// validation invariants of [`Self::new`] are trivially satisfied.
    #[must_use]
    pub fn placeholder() -> Self {
        Self(String::from("00000000"))
    }
}

impl fmt::Debug for DeviceSerial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceSerial")
            .field("len", &self.0.len())
            .field("value", &"<redacted>")
            .finish()
    }
}

/// The Plaud product line. Expanded as new models are verified against
/// the protocol spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeviceModel {
    /// Original Plaud Note (credit-card form factor).
    Note,
    /// Plaud NotePin wearable.
    NotePin,
    /// Plaud NotePin S wearable.
    NotePinS,
    /// Plaud Note Pro.
    NotePro,
    /// Any other product line the CLI has not been validated against.
    /// Carries the raw model name as advertised by the device.
    Unknown(String),
}

impl DeviceModel {
    /// Human-readable name for the CLI's textual output.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Note => "Plaud Note",
            Self::NotePin => "Plaud NotePin",
            Self::NotePinS => "Plaud NotePin S",
            Self::NotePro => "Plaud Note Pro",
            Self::Unknown(raw) => raw.as_str(),
        }
    }
}

/// Validation errors produced by [`FirmwareVersion::parse_model_txt_line`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum FirmwareVersionError {
    /// The input did not contain a `V<build>` token.
    #[error("no `V<build>` token found in model.txt line")]
    MissingBuildToken,
    /// The build token was empty after the `V` prefix.
    #[error("build identifier after `V` was empty")]
    EmptyBuild,
}

/// Firmware identification parsed from the first line of `MODEL.txt`.
///
/// Example input: `PLAUD NOTE V0095@00:47:14 Feb 28 2024`
///
/// Evidence: `specs/re/captures/usb/2026-04-05-plaud-note-v0095-baseline.md`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirmwareVersion {
    build: String,
    build_stamp: Option<String>,
}

impl FirmwareVersion {
    /// Parse the first line of a `MODEL.txt` file.
    ///
    /// The parser scans for the first whitespace-delimited token that
    /// starts with `V`, extracts the build identifier up to the `@`
    /// separator (or end of token if absent), and captures the rest of
    /// the line after `@` as the build time stamp.
    ///
    /// # Errors
    ///
    /// Returns [`FirmwareVersionError`] if no `V`-prefixed token is
    /// found, or if the build identifier is empty.
    pub fn parse_model_txt_line(line: &str) -> Result<Self, FirmwareVersionError> {
        let token = line
            .split_whitespace()
            .find(|t| t.starts_with(FIRMWARE_BUILD_PREFIX))
            .ok_or(FirmwareVersionError::MissingBuildToken)?;
        let after_prefix = token.trim_start_matches(FIRMWARE_BUILD_PREFIX);
        let build_str = match after_prefix.split_once(FIRMWARE_BUILD_STAMP_SEPARATOR) {
            Some((b, _)) => b,
            None => after_prefix,
        };
        if build_str.is_empty() {
            return Err(FirmwareVersionError::EmptyBuild);
        }
        // When the line contains `@`, the build stamp is everything
        // after the first `@` through to the end of the line. This
        // preserves multi-token stamps like `00:47:14 Feb 28 2024`.
        let build_stamp = line
            .split_once(FIRMWARE_BUILD_STAMP_SEPARATOR)
            .map(|(_, tail)| tail.trim().to_owned())
            .filter(|s| !s.is_empty());
        Ok(Self {
            build: build_str.to_owned(),
            build_stamp,
        })
    }

    /// The build identifier, e.g. `"0095"`.
    #[must_use]
    pub fn build(&self) -> &str {
        &self.build
    }

    /// The build time stamp if present, e.g. `"00:47:14 Feb 28 2024"`.
    #[must_use]
    pub fn build_stamp(&self) -> Option<&str> {
        self.build_stamp.as_deref()
    }

    /// A placeholder firmware version suitable for simulators and
    /// tests. Carries a non-empty build identifier and no stamp.
    #[must_use]
    pub fn placeholder() -> Self {
        Self {
            build: String::from("0000"),
            build_stamp: None,
        }
    }
}

impl DeviceInfo {
    /// A placeholder [`DeviceInfo`] suitable for simulators and
    /// tests. Local name is `"PLAUD_NOTE"`; model is
    /// [`DeviceModel::Note`]; firmware and serial are placeholders.
    #[must_use]
    pub fn placeholder() -> Self {
        Self {
            local_name: String::from("PLAUD_NOTE"),
            model: DeviceModel::Note,
            firmware: FirmwareVersion::placeholder(),
            serial: DeviceSerial::placeholder(),
        }
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build)?;
        if let Some(stamp) = &self.build_stamp {
            f.write_str(" (")?;
            f.write_str(stamp)?;
            f.write_str(")")?;
        }
        Ok(())
    }
}

/// Full device identity as reported over the transport layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Advertised BLE local name (e.g. `"PLAUD_NOTE"`).
    pub local_name: String,
    /// Product line.
    pub model: DeviceModel,
    /// Firmware version from `MODEL.txt` or the equivalent BLE query.
    pub firmware: FirmwareVersion,
    /// Device serial (non-leaking `Debug`).
    pub serial: DeviceSerial,
}
