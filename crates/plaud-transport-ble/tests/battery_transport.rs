//! Tests for `BleTransport::battery`.
//!
//! Verifies the critical invariant that battery reads bypass the
//! session's auth state entirely (matching Test 2b live evidence).
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use std::sync::Arc;

use plaud_domain::BatteryLevel;
use plaud_transport::Transport;
use plaud_transport_ble::{BleChannel, BleSession, BleTransport, FixedBatteryReader};
use tokio::sync::Mutex;

const BATTERY_PERCENT: u8 = 87;

#[tokio::test]
async fn battery_returns_configured_level_without_authentication() {
    let (channel, _peer) = BleChannel::loopback_pair();
    let session = Arc::new(Mutex::new(BleSession::new(channel)));
    let battery = Arc::new(FixedBatteryReader::new(BatteryLevel::new(BATTERY_PERCENT).expect("valid")));
    let transport = BleTransport::from_parts(session, battery);
    let level = transport.battery().await.expect("battery read");
    assert_eq!(level.percent(), BATTERY_PERCENT);
}
