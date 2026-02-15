use hypr_sdk::config::*;

#[test]
fn config_option_type_from_raw() {
    assert_eq!(ConfigOptionType::from_raw(0), Some(ConfigOptionType::Bool));
    assert_eq!(ConfigOptionType::from_raw(4), Some(ConfigOptionType::StringLong));
    assert_eq!(ConfigOptionType::from_raw(8), Some(ConfigOptionType::Vector));
    assert_eq!(ConfigOptionType::from_raw(99), None);
}

#[test]
fn config_option_flags_percentage() {
    let flags = ConfigOptionFlags::PERCENTAGE;
    assert!(flags.is_percentage());

    let empty = ConfigOptionFlags::default();
    assert!(!empty.is_percentage());
}

#[test]
fn css_gap_uniform() {
    let gap = CssGapData::uniform(10);
    assert_eq!(gap.top, 10);
    assert_eq!(gap.right, 10);
    assert_eq!(gap.bottom, 10);
    assert_eq!(gap.left, 10);
}

#[test]
fn css_gap_symmetric() {
    let gap = CssGapData::symmetric(5, 10);
    assert_eq!(gap.top, 5);
    assert_eq!(gap.right, 10);
    assert_eq!(gap.bottom, 5);
    assert_eq!(gap.left, 10);
}

#[test]
fn css_gap_serde_roundtrip() {
    let gap = CssGapData { top: 1, right: 2, bottom: 3, left: 4 };
    let json = serde_json::to_string(&gap).unwrap();
    let parsed: CssGapData = serde_json::from_str(&json).unwrap();
    assert_eq!(gap, parsed);
}

#[test]
fn config_option_data_variants() {
    let bool_data = ConfigOptionData::Bool { value: true };
    assert!(matches!(bool_data, ConfigOptionData::Bool { value: true }));

    let range_data = ConfigOptionData::Range { value: 5, min: 0, max: 10 };
    assert!(matches!(range_data, ConfigOptionData::Range { value: 5, min: 0, max: 10 }));

    let choice_data = ConfigOptionData::Choice {
        first_index: 0,
        choices: "one,two,three".to_string(),
    };
    if let ConfigOptionData::Choice { choices, .. } = &choice_data {
        assert_eq!(choices, "one,two,three");
    }
}

#[test]
fn color_management_type_defaults_to_auto() {
    let cm = ColorManagementType::default();
    assert_eq!(cm, ColorManagementType::Auto);
}

#[test]
fn gradient_value_construction() {
    let grad = GradientValue {
        colors: vec![0xFF0000FF, 0x00FF00FF],
        angle: std::f32::consts::FRAC_PI_4,
    };
    assert_eq!(grad.colors.len(), 2);
    assert!((grad.angle - std::f32::consts::FRAC_PI_4).abs() < f32::EPSILON);
}
