//! Tests for failure injection: `inject_disconnect_after` and
//! `inject_delay`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use std::time::{Duration, Instant};

use plaud_sim::SimDevice;
use plaud_transport::Error;

const DISCONNECT_THRESHOLD: u32 = 2;
const INJECTED_DELAY_MS: u64 = 50;

#[tokio::test]
async fn disconnect_after_allows_n_operations_then_errors() {
    let sim = SimDevice::builder().inject_disconnect_after(DISCONNECT_THRESHOLD).build();
    let transport = sim.authenticated_transport();
    // First two ops succeed.
    transport.battery().await.expect("op 1");
    transport.battery().await.expect("op 2");
    // The third op is past the threshold and errors as transport loss.
    let err = transport.battery().await.unwrap_err();
    assert!(matches!(err, Error::Transport(_)));
    // Op counter reflects the two successful ops plus the failed
    // attempt that was rejected before incrementing.
    assert!(sim.op_count() >= DISCONNECT_THRESHOLD);
}

#[tokio::test]
async fn delay_per_op_actually_sleeps_the_configured_duration() {
    let sim = SimDevice::builder().inject_delay(Duration::from_millis(INJECTED_DELAY_MS)).build();
    let transport = sim.authenticated_transport();
    let start = Instant::now();
    transport.battery().await.expect("delayed battery read");
    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(INJECTED_DELAY_MS),
        "expected at least {INJECTED_DELAY_MS}ms, got {elapsed:?}"
    );
}

#[tokio::test]
async fn op_count_increments_on_each_call() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    assert_eq!(sim.op_count(), 0);
    transport.battery().await.expect("op");
    assert_eq!(sim.op_count(), 1);
    transport.battery().await.expect("op");
    assert_eq!(sim.op_count(), 2);
}
