use hyprs_sdk::config::*;
use hyprs_sdk::types::common::OutputTransform;

#[test]
fn monitor_rule_defaults() {
    let rule = MonitorRule::default();
    assert!(rule.name.is_empty());
    assert_eq!(rule.auto_dir, AutoDirection::None);
    assert!((rule.resolution_x - 1280.0).abs() < f64::EPSILON);
    assert!((rule.resolution_y - 720.0).abs() < f64::EPSILON);
    assert!((rule.scale - 1.0).abs() < f32::EPSILON);
    assert!((rule.refresh_rate - 60.0).abs() < f32::EPSILON);
    assert!(!rule.disabled);
    assert_eq!(rule.transform, OutputTransform::Normal);
    assert!(rule.mirror_of.is_empty());
    assert!(!rule.enable_10bit);
    assert_eq!(rule.cm_type, ColorManagementType::Srgb);
    assert_eq!(rule.vrr, None);
}

#[test]
fn monitor_rule_custom() {
    let rule = MonitorRule {
        name: "DP-1".to_string(),
        resolution_x: 2560.0,
        resolution_y: 1440.0,
        scale: 1.5,
        refresh_rate: 165.0,
        enable_10bit: true,
        cm_type: ColorManagementType::Wide,
        vrr: Some(1),
        ..Default::default()
    };
    assert_eq!(rule.name, "DP-1");
    assert!((rule.resolution_x - 2560.0).abs() < f64::EPSILON);
    assert!(rule.enable_10bit);
    assert_eq!(rule.vrr, Some(1));
}

#[test]
fn auto_direction_variants() {
    assert_eq!(AutoDirection::None as u8, 0);
    assert_eq!(AutoDirection::Right as u8, 4);
    assert_eq!(AutoDirection::CenterRight as u8, 8);
}

#[test]
fn monitor_rule_hdr_settings() {
    let rule = MonitorRule {
        name: "HDMI-A-1".to_string(),
        cm_type: ColorManagementType::Hdr,
        supports_hdr: 1,
        supports_wide_color: 1,
        sdr_brightness: 1.5,
        sdr_saturation: 0.8,
        min_luminance: 0.1,
        max_luminance: 1000,
        ..Default::default()
    };
    assert_eq!(rule.cm_type, ColorManagementType::Hdr);
    assert_eq!(rule.supports_hdr, 1);
    assert_eq!(rule.max_luminance, 1000);
}
