//! Device discovery types: transport hint and candidate descriptors.
//!
//! Discovery surfaces *candidates* — summaries of what a scanner saw
//! over the air / on the bus — without yet establishing a working
//! session. The CLI then picks a candidate and hands it to a
//! transport-specific connector.

/// Which transport a discovery candidate was observed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TransportHint {
    /// Bluetooth Low Energy central observed this candidate.
    Ble,
    /// USB Mass Storage enumeration observed this candidate.
    Usb,
    /// Wi-Fi Fast Transfer hotspot observed this candidate.
    Wifi,
}

impl TransportHint {
    /// Short, stable, human-readable name used by the CLI's textual
    /// output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ble => "ble",
            Self::Usb => "usb",
            Self::Wifi => "wifi",
        }
    }
}

/// A device observed by a discovery scan but not yet connected to.
///
/// The fields are intentionally minimal: everything downstream
/// milestones need to identify, filter, rank, and connect to the
/// device — nothing more, so the type is stable as new transports are
/// added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCandidate {
    /// Advertised local name (e.g. `"PLAUD_NOTE"`).
    pub local_name: String,
    /// BLE manufacturer identifier (`0x0059` for Nordic on Plaud Note;
    /// `0` for non-BLE transports that have no equivalent).
    pub manufacturer_id: u16,
    /// RSSI in dBm if available from the scanner (BLE only; `None`
    /// for USB / Wi-Fi).
    pub rssi_dbm: Option<i16>,
    /// Which transport produced this candidate.
    pub transport_hint: TransportHint,
}

impl DeviceCandidate {
    /// Construct a candidate.
    #[must_use]
    pub const fn new(local_name: String, manufacturer_id: u16, rssi_dbm: Option<i16>, transport_hint: TransportHint) -> Self {
        Self {
            local_name,
            manufacturer_id,
            rssi_dbm,
            transport_hint,
        }
    }
}
