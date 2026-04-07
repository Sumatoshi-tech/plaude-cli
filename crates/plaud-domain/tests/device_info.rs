//! Composition tests for [`plaud_domain::DeviceInfo`].
//!
//! The critical invariant here is that `DeviceInfo` does not leak
//! the device serial through its own derived `Debug` impl — it
//! inherits `DeviceSerial`'s redacting `Debug` because the serial
//! field's type is `DeviceSerial`, not `String`. A test asserts
//! this holds by checking no long digit run appears in the output.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_domain::{DeviceInfo, DeviceModel, DeviceSerial, FirmwareVersion};

const LOCAL_NAME: &str = "PLAUD_NOTE";
const PLACEHOLDER_SERIAL: &str = "111222333444555666"; // 18 digits, fake
const MODEL_TXT: &str = "PLAUD NOTE V0095@00:47:14 Feb 28 2024";
const MIN_RUN_OF_DIGITS_THAT_WOULD_LEAK_SERIAL: usize = 8;

fn contains_long_digit_run(s: &str, min_run: usize) -> bool {
    let mut run = 0usize;
    for byte in s.as_bytes() {
        if byte.is_ascii_digit() {
            run += 1;
            if run >= min_run {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn sample_info() -> DeviceInfo {
    DeviceInfo {
        local_name: LOCAL_NAME.to_owned(),
        model: DeviceModel::Note,
        firmware: FirmwareVersion::parse_model_txt_line(MODEL_TXT).expect("parses"),
        serial: DeviceSerial::new(PLACEHOLDER_SERIAL).expect("valid"),
    }
}

#[test]
fn debug_of_device_info_does_not_leak_the_serial() {
    let info = sample_info();
    let debug = format!("{info:?}");
    assert!(
        !contains_long_digit_run(&debug, MIN_RUN_OF_DIGITS_THAT_WOULD_LEAK_SERIAL),
        "DeviceInfo Debug output leaked serial: {debug}"
    );
}

#[test]
fn device_info_carries_every_field() {
    let info = sample_info();
    assert_eq!(info.local_name, LOCAL_NAME);
    assert_eq!(info.model, DeviceModel::Note);
    assert_eq!(info.firmware.build(), "0095");
    assert_eq!(info.serial.reveal(), PLACEHOLDER_SERIAL);
}
