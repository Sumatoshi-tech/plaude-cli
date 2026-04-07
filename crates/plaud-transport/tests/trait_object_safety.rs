//! Compile-only tests asserting every boundary trait in
//! `plaud-transport` is `dyn`-compatible.
//!
//! These tests do not *run* anything — the linker never even sees a
//! function body that calls them. What matters is that each `fn`
//! compiles, which requires the trait to be object-safe. If a future
//! change accidentally breaks `dyn Transport` (e.g. by returning
//! `impl Trait` in a trait method, or by adding a generic method),
//! this file will fail to compile and the CI will catch it.
//!
//! Journey: specs/plaude-cli-v1/journeys/M01-domain-traits.md

use plaud_transport::{AuthStore, DeviceDiscovery, Transport};

#[test]
fn transport_is_dyn_compatible() {
    fn _accepts_boxed_transport(_t: Box<dyn Transport>) {}
}

#[test]
fn device_discovery_is_dyn_compatible() {
    fn _accepts_boxed_discovery(_d: Box<dyn DeviceDiscovery>) {}
}

#[test]
fn auth_store_is_dyn_compatible() {
    fn _accepts_boxed_auth_store(_a: Box<dyn AuthStore>) {}
}
