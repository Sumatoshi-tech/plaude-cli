//! Tests for `SimTransport::battery` — the only Transport method
//! that succeeds without an authenticated session.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::BatteryLevel;
use plaud_sim::SimDevice;

const MID_BATTERY_PERCENT: u8 = 42;

fn mid_battery() -> BatteryLevel {
    BatteryLevel::new(MID_BATTERY_PERCENT).expect("hand-validated battery percent")
}

#[tokio::test]
async fn battery_returns_the_configured_level_when_authenticated() {
    let sim = SimDevice::builder().with_battery(mid_battery()).build();
    let transport = sim.authenticated_transport();
    let level = transport.battery().await.expect("battery read succeeds");
    assert_eq!(level.percent(), MID_BATTERY_PERCENT);
}

#[tokio::test]
async fn battery_still_works_when_session_is_unauthenticated() {
    let sim = SimDevice::builder().with_battery(mid_battery()).build();
    let transport = sim.unauthenticated_transport();
    let level = transport.battery().await.expect("SIG battery ignores auth");
    assert_eq!(level.percent(), MID_BATTERY_PERCENT);
}
