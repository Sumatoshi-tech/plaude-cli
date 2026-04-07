//! `MODEL.txt` parser.
//!
//! The file is a 2-line ASCII record at the root of the `PLAUD_NOTE`
//! VFAT volume. Schema (from `docs/protocol/file-formats.md`):
//!
//! ```text
//! <product-name> V<build>@<build-time> <build-date>\n
//! Serial No.:<serial>\n
//! ```
//!
//! The parser is intentionally fixed-field: it splits on the literal
//! `Serial No.:` prefix rather than running a regex, so a firmware
//! change that adds a new key is easy to spot.

use plaud_domain::{DeviceInfo, DeviceModel, DeviceSerial, DeviceSerialError, FirmwareVersion, FirmwareVersionError};
use thiserror::Error;

/// Prefix for line 2 of `MODEL.txt`. Everything after it is the raw
/// device serial (trailing whitespace stripped).
const SERIAL_PREFIX: &str = "Serial No.:";

/// Parse errors emitted by [`parse`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelTxtError {
    /// The file had fewer than the two expected lines.
    #[error("MODEL.txt requires at least two lines, got {got}")]
    TooShort {
        /// Number of non-empty lines actually present.
        got: usize,
    },
    /// Line 1 did not contain a parseable `V<build>` token.
    #[error("failed to parse firmware line: {0}")]
    Firmware(#[from] FirmwareVersionError),
    /// Line 2 was missing the literal `Serial No.:` prefix.
    #[error("missing `Serial No.:` prefix on line 2")]
    MissingSerialPrefix,
    /// The serial string after the prefix failed validation.
    #[error("serial validation failed: {0}")]
    Serial(#[from] DeviceSerialError),
}

/// Parse the full contents of a `MODEL.txt` file.
///
/// # Errors
///
/// Returns a [`ModelTxtError`] variant for any schema violation.
pub fn parse(contents: &str) -> Result<DeviceInfo, ModelTxtError> {
    let mut lines = contents.lines().filter(|l| !l.trim().is_empty());
    let firmware_line = lines.next().ok_or(ModelTxtError::TooShort { got: 0 })?;
    let serial_line = lines.next().ok_or(ModelTxtError::TooShort { got: 1 })?;
    let firmware = FirmwareVersion::parse_model_txt_line(firmware_line)?;
    let serial_raw = serial_line
        .trim()
        .strip_prefix(SERIAL_PREFIX)
        .ok_or(ModelTxtError::MissingSerialPrefix)?
        .trim();
    let serial = DeviceSerial::new(serial_raw)?;
    let local_name = firmware_line.split_whitespace().next().unwrap_or("PLAUD").to_owned();
    Ok(DeviceInfo {
        local_name,
        // Only `Note` is in our shipped corpus; extend when we see
        // NotePin / NotePinS / Pro hardware.
        model: DeviceModel::Note,
        firmware,
        serial,
    })
}

#[cfg(test)]
mod tests {
    use plaud_domain::DeviceModel;

    use super::{ModelTxtError, parse};

    const HAPPY_INPUT: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024\nSerial No.:123456789012345678\n";
    const HAPPY_SERIAL: &str = "123456789012345678";
    const HAPPY_BUILD: &str = "0095";
    const TOO_SHORT_INPUT: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024\n";
    const MISSING_PREFIX_INPUT: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024\n123456789012345678\n";
    const EMPTY_SERIAL_INPUT: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024\nSerial No.:\n";
    const NO_BUILD_INPUT: &str = "PLAUD NOTE\nSerial No.:123456789012345678\n";

    #[test]
    fn parse_happy_path_pulls_product_firmware_and_serial() {
        let info = parse(HAPPY_INPUT).expect("parse");
        assert_eq!(info.local_name, "PLAUD");
        assert_eq!(info.model, DeviceModel::Note);
        assert_eq!(info.firmware.build(), HAPPY_BUILD);
        assert_eq!(info.serial.reveal(), HAPPY_SERIAL);
    }

    #[test]
    fn parse_preserves_the_build_stamp() {
        let info = parse(HAPPY_INPUT).expect("parse");
        assert_eq!(info.firmware.build_stamp(), Some("00:47:14 Feb 28 2024"));
    }

    #[test]
    fn parse_rejects_single_line_input() {
        let err = parse(TOO_SHORT_INPUT).unwrap_err();
        assert!(matches!(err, ModelTxtError::TooShort { got: 1 }));
    }

    #[test]
    fn parse_rejects_empty_input() {
        let err = parse("").unwrap_err();
        assert!(matches!(err, ModelTxtError::TooShort { got: 0 }));
    }

    #[test]
    fn parse_rejects_line_two_without_prefix() {
        let err = parse(MISSING_PREFIX_INPUT).unwrap_err();
        assert!(matches!(err, ModelTxtError::MissingSerialPrefix));
    }

    #[test]
    fn parse_rejects_empty_serial_value() {
        let err = parse(EMPTY_SERIAL_INPUT).unwrap_err();
        assert!(matches!(err, ModelTxtError::Serial(_)));
    }

    #[test]
    fn parse_rejects_firmware_line_without_build_token() {
        let err = parse(NO_BUILD_INPUT).unwrap_err();
        assert!(matches!(err, ModelTxtError::Firmware(_)));
    }
}
