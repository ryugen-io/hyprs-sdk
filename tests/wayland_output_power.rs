#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::output_power::PowerMode;

#[test]
fn power_mode_variants() {
    // WHY: Needed for correctness and maintainability: Protocol defines Off=0, On=1
    assert_eq!(PowerMode::Off as u32, 0);
    assert_eq!(PowerMode::On as u32, 1);
}

#[test]
fn power_mode_from_raw() {
    assert_eq!(PowerMode::from_raw(0), Some(PowerMode::Off));
    assert_eq!(PowerMode::from_raw(1), Some(PowerMode::On));
    assert_eq!(PowerMode::from_raw(99), None);
}

#[test]
fn power_mode_display() {
    assert_eq!(PowerMode::On.to_string(), "on");
    assert_eq!(PowerMode::Off.to_string(), "off");
}
