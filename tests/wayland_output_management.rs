#![cfg(feature = "wayland")]
use hypr_sdk::protocols::output_management::*;

#[test]
fn output_mode_refresh_hz() {
    let mode = OutputMode {
        width: 2560,
        height: 1440,
        refresh: 165000,
        preferred: true,
    };
    assert!((mode.refresh_hz() - 165.0).abs() < f64::EPSILON);
}

#[test]
fn output_head_defaults() {
    let head = OutputHead::default();
    assert!(head.name.is_empty());
    assert!(head.modes.is_empty());
    assert!(!head.enabled);
}

#[test]
fn output_config_entry() {
    let entry = OutputConfigEntry {
        name: "DP-1".to_string(),
        enabled: true,
        mode: Some(OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
            preferred: false,
        }),
        position_x: Some(0),
        position_y: Some(0),
        scale: Some(1.0),
        transform: Some(0),
    };
    assert!(entry.enabled);
}
