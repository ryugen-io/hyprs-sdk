#![cfg(feature = "wayland")]
use hypr_sdk::protocols::idle::*;

#[test]
fn idle_config_from_secs() {
    let config = IdleNotificationConfig::from_secs(300);
    assert_eq!(config.timeout_ms(), 300_000);
}

#[test]
fn idle_config_from_millis() {
    let config = IdleNotificationConfig::from_millis(5000);
    assert_eq!(config.timeout_ms(), 5000);
}

#[test]
fn idle_state_variants() {
    assert_ne!(IdleState::Active, IdleState::Idle);
}
