#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::session_lock::*;

#[test]
fn lock_state_variants() {
    assert_ne!(LockState::Locked, LockState::Finished);
}

#[test]
fn lock_surface_config() {
    let config = LockSurfaceConfig::new(1920, 1080);
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}
