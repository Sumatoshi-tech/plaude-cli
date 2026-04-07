//! Common device settings — the key enum, the value union, and the
//! `Setting` pair the `Transport::read_setting` / `write_setting`
//! methods traffic in.
//!
//! The key enum mirrors `Constants$CommonSettings$SettingType` from
//! the tinnotech pen-BLE SDK (20 variants). Evidence:
//!
//! `specs/re/captures/apk/decompiled/3.14.0-620/sources/com/tinnotech/penblesdk/Constants$CommonSettings$SettingType.java`

use thiserror::Error;

// ---------------------------------------------------------------------
// Setting-type codes as extracted from the tinnotech SDK source.
// Each constant corresponds to one enum variant below and is the
// single source of truth consumed by both `code()` and `from_code()`.
// ---------------------------------------------------------------------

const CODE_BACK_LIGHT_TIME: u8 = 1;
const CODE_BACK_LIGHT_BRIGHTNESS: u8 = 2;
const CODE_LANGUAGE: u8 = 3;
const CODE_AUTO_DELETE_RECORD_FILE: u8 = 4;
const CODE_ENABLE_VAD: u8 = 15;
const CODE_REC_SCENE: u8 = 16;
const CODE_REC_MODE: u8 = 17;
const CODE_VAD_SENSITIVITY: u8 = 18;
const CODE_VPU_GAIN: u8 = 19;
const CODE_MIC_GAIN: u8 = 20;
const CODE_WIFI_CHANNEL: u8 = 21;
const CODE_SWITCH_HANDLER_ID: u8 = 22;
const CODE_AUTO_POWER_OFF: u8 = 23;
const CODE_SAVE_RAW_FILE: u8 = 24;
const CODE_AUTO_RECORD: u8 = 25;
const CODE_AUTO_SYNC: u8 = 26;
const CODE_FIND_MY: u8 = 27;
const CODE_VPU_CLK: u8 = 30;
const CODE_AUTO_STOP_RECORD: u8 = 31;
const CODE_BATTERY_MODE: u8 = 32;

/// A configurable device setting, identified by its numeric code on
/// the wire and by a symbolic Rust variant.
///
/// Every variant is documented with the `SettingType` enum value it
/// mirrors from the tinnotech SDK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CommonSettingKey {
    /// Screen backlight-on duration (seconds).
    BackLightTime,
    /// Screen backlight brightness (device-defined scale).
    BackLightBrightness,
    /// UI language code.
    Language,
    /// Whether the device auto-deletes old recordings on full flash.
    AutoDeleteRecordFile,
    /// Voice activity detection master switch.
    EnableVad,
    /// Recording scene profile (meeting, interview, etc.).
    RecScene,
    /// Recording mode variant.
    RecMode,
    /// VAD sensitivity level.
    VadSensitivity,
    /// Voice processing unit gain.
    VpuGain,
    /// Microphone pre-amp gain.
    MicGain,
    /// Wi-Fi channel preference for Fast Transfer / Sync-In-Idle.
    WifiChannel,
    /// Handler id for the device's physical mode switch.
    SwitchHandlerId,
    /// Auto-power-off timeout.
    AutoPowerOff,
    /// Whether the raw WAV file is retained alongside the Opus sidecar.
    SaveRawFile,
    /// Whether the device auto-starts a recording in specific conditions.
    AutoRecord,
    /// Whether the device uploads recordings over Wi-Fi when idle.
    AutoSync,
    /// "Find My" feature toggle.
    FindMy,
    /// Voice processing unit clock rate.
    VpuClk,
    /// Whether the device auto-stops a recording after an interval.
    AutoStopRecord,
    /// Battery power profile.
    BatteryMode,
}

/// Error returned by [`CommonSettingKey::from_code`] for unknown codes.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown setting code {code}")]
pub struct UnknownSettingCode {
    /// The code that was not recognised.
    pub code: u8,
}

/// Error returned by [`CommonSettingKey::from_name`] for unrecognised
/// setting names.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown setting name: {name}")]
pub struct UnknownSettingName {
    /// The name that was not recognised.
    pub name: String,
}

impl CommonSettingKey {
    /// The wire-level code carried in the `SettingType` byte of a
    /// `0x0008 CommonSettings` request.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::BackLightTime => CODE_BACK_LIGHT_TIME,
            Self::BackLightBrightness => CODE_BACK_LIGHT_BRIGHTNESS,
            Self::Language => CODE_LANGUAGE,
            Self::AutoDeleteRecordFile => CODE_AUTO_DELETE_RECORD_FILE,
            Self::EnableVad => CODE_ENABLE_VAD,
            Self::RecScene => CODE_REC_SCENE,
            Self::RecMode => CODE_REC_MODE,
            Self::VadSensitivity => CODE_VAD_SENSITIVITY,
            Self::VpuGain => CODE_VPU_GAIN,
            Self::MicGain => CODE_MIC_GAIN,
            Self::WifiChannel => CODE_WIFI_CHANNEL,
            Self::SwitchHandlerId => CODE_SWITCH_HANDLER_ID,
            Self::AutoPowerOff => CODE_AUTO_POWER_OFF,
            Self::SaveRawFile => CODE_SAVE_RAW_FILE,
            Self::AutoRecord => CODE_AUTO_RECORD,
            Self::AutoSync => CODE_AUTO_SYNC,
            Self::FindMy => CODE_FIND_MY,
            Self::VpuClk => CODE_VPU_CLK,
            Self::AutoStopRecord => CODE_AUTO_STOP_RECORD,
            Self::BatteryMode => CODE_BATTERY_MODE,
        }
    }

    /// Decode a wire-level setting code into its symbolic variant.
    ///
    /// The tinnotech SDK's own `find()` method is incomplete (several
    /// variants cannot be looked up by code); this implementation
    /// covers every declared variant to avoid reproducing that bug.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownSettingCode`] for any code outside the set
    /// enumerated by [`Self::code`].
    pub const fn from_code(code: u8) -> Result<Self, UnknownSettingCode> {
        let key = match code {
            CODE_BACK_LIGHT_TIME => Self::BackLightTime,
            CODE_BACK_LIGHT_BRIGHTNESS => Self::BackLightBrightness,
            CODE_LANGUAGE => Self::Language,
            CODE_AUTO_DELETE_RECORD_FILE => Self::AutoDeleteRecordFile,
            CODE_ENABLE_VAD => Self::EnableVad,
            CODE_REC_SCENE => Self::RecScene,
            CODE_REC_MODE => Self::RecMode,
            CODE_VAD_SENSITIVITY => Self::VadSensitivity,
            CODE_VPU_GAIN => Self::VpuGain,
            CODE_MIC_GAIN => Self::MicGain,
            CODE_WIFI_CHANNEL => Self::WifiChannel,
            CODE_SWITCH_HANDLER_ID => Self::SwitchHandlerId,
            CODE_AUTO_POWER_OFF => Self::AutoPowerOff,
            CODE_SAVE_RAW_FILE => Self::SaveRawFile,
            CODE_AUTO_RECORD => Self::AutoRecord,
            CODE_AUTO_SYNC => Self::AutoSync,
            CODE_FIND_MY => Self::FindMy,
            CODE_VPU_CLK => Self::VpuClk,
            CODE_AUTO_STOP_RECORD => Self::AutoStopRecord,
            CODE_BATTERY_MODE => Self::BatteryMode,
            _ => return Err(UnknownSettingCode { code }),
        };
        Ok(key)
    }

    /// Stable human-readable name used by the CLI's textual output
    /// and by `plaude settings` subcommand argument parsing.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BackLightTime => "back-light-time",
            Self::BackLightBrightness => "back-light-brightness",
            Self::Language => "language",
            Self::AutoDeleteRecordFile => "auto-delete-record-file",
            Self::EnableVad => "enable-vad",
            Self::RecScene => "rec-scene",
            Self::RecMode => "rec-mode",
            Self::VadSensitivity => "vad-sensitivity",
            Self::VpuGain => "vpu-gain",
            Self::MicGain => "mic-gain",
            Self::WifiChannel => "wifi-channel",
            Self::SwitchHandlerId => "switch-handler-id",
            Self::AutoPowerOff => "auto-power-off",
            Self::SaveRawFile => "save-raw-file",
            Self::AutoRecord => "auto-record",
            Self::AutoSync => "auto-sync",
            Self::FindMy => "find-my",
            Self::VpuClk => "vpu-clk",
            Self::AutoStopRecord => "auto-stop-record",
            Self::BatteryMode => "battery-mode",
        }
    }

    /// Parse a human-readable setting name (as produced by [`Self::name`])
    /// back into the corresponding variant.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownSettingName`] if the name does not match any
    /// variant.
    pub fn from_name(name: &str) -> Result<Self, UnknownSettingName> {
        Self::all()
            .iter()
            .find(|k| k.name() == name)
            .copied()
            .ok_or_else(|| UnknownSettingName { name: name.to_owned() })
    }

    /// Every variant, in declaration order. Useful for iterating
    /// settings during a full `list` sweep.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::BackLightTime,
            Self::BackLightBrightness,
            Self::Language,
            Self::AutoDeleteRecordFile,
            Self::EnableVad,
            Self::RecScene,
            Self::RecMode,
            Self::VadSensitivity,
            Self::VpuGain,
            Self::MicGain,
            Self::WifiChannel,
            Self::SwitchHandlerId,
            Self::AutoPowerOff,
            Self::SaveRawFile,
            Self::AutoRecord,
            Self::AutoSync,
            Self::FindMy,
            Self::VpuClk,
            Self::AutoStopRecord,
            Self::BatteryMode,
        ]
    }
}

/// The value a setting can carry. The union is narrow on purpose: the
/// CLI does not want its settings subsystem to grow into a generic
/// key-value store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SettingValue {
    /// Boolean setting, e.g. `EnableVad`.
    Bool(bool),
    /// Small unsigned integer, e.g. `MicGain`, `BatteryMode`.
    U8(u8),
    /// Wider unsigned integer, e.g. `AutoPowerOff` timeout in seconds.
    U32(u32),
}

impl SettingValue {
    /// Parse a CLI-supplied string into a [`SettingValue`].
    ///
    /// Tries boolean first (`true`/`false`), then `u8`, then `u32`.
    /// Returns the narrowest type that fits.
    ///
    /// # Errors
    ///
    /// Returns [`SettingValueParseError`] if the string cannot be
    /// interpreted as any supported type.
    pub fn parse(s: &str) -> Result<Self, SettingValueParseError> {
        match s {
            "true" => return Ok(Self::Bool(true)),
            "false" => return Ok(Self::Bool(false)),
            _ => {}
        }
        if let Ok(v) = s.parse::<u8>() {
            return Ok(Self::U8(v));
        }
        if let Ok(v) = s.parse::<u32>() {
            return Ok(Self::U32(v));
        }
        Err(SettingValueParseError { input: s.to_owned() })
    }
}

/// Error returned by [`SettingValue::parse`] for unrecognised input.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("cannot parse setting value: {input}")]
pub struct SettingValueParseError {
    /// The input that could not be parsed.
    pub input: String,
}

impl std::fmt::Display for SettingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v}"),
            Self::U8(v) => write!(f, "{v}"),
            Self::U32(v) => write!(f, "{v}"),
        }
    }
}

/// A key + value pair as produced by `Transport::read_setting` and
/// consumed by `Transport::write_setting`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Setting {
    /// Which setting the value belongs to.
    pub key: CommonSettingKey,
    /// The current value, already typed for ergonomic consumption.
    pub value: SettingValue,
}

impl Setting {
    /// Construct a setting pair.
    #[must_use]
    pub const fn new(key: CommonSettingKey, value: SettingValue) -> Self {
        Self { key, value }
    }
}
