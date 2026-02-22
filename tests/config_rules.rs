use hypr_sdk::config::*;

#[test]
fn window_rule_construction() {
    let rule = WindowRule {
        effect: WindowRuleEffect::Opacity,
        value: "0.9 0.8".to_string(),
    };
    assert_eq!(rule.effect, WindowRuleEffect::Opacity);
    assert_eq!(rule.value, "0.9 0.8");
}

#[test]
fn window_rule_static_effects() {
    assert_ne!(WindowRuleEffect::Float, WindowRuleEffect::Tile);
    assert_ne!(WindowRuleEffect::Fullscreen, WindowRuleEffect::Maximize);
    assert_ne!(WindowRuleEffect::Pin, WindowRuleEffect::NoInitialFocus);
}

#[test]
fn window_rule_dynamic_effects() {
    let effects = [
        WindowRuleEffect::Rounding,
        WindowRuleEffect::Animation,
        WindowRuleEffect::BorderColor,
        WindowRuleEffect::Opacity,
        WindowRuleEffect::NoBlur,
        WindowRuleEffect::NoShadow,
        WindowRuleEffect::Xray,
    ];
    // WHY: Needed for correctness and maintainability: All variants should be distinct
    for (i, a) in effects.iter().enumerate() {
        for (j, b) in effects.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn layer_rule_construction() {
    let rule = LayerRule {
        effect: LayerRuleEffect::Blur,
        value: String::new(),
    };
    assert_eq!(rule.effect, LayerRuleEffect::Blur);
}

#[test]
fn layer_rule_all_effects() {
    let effects = [
        LayerRuleEffect::NoAnim,
        LayerRuleEffect::Blur,
        LayerRuleEffect::BlurPopups,
        LayerRuleEffect::IgnoreAlpha,
        LayerRuleEffect::DimAround,
        LayerRuleEffect::Xray,
        LayerRuleEffect::Animation,
        LayerRuleEffect::Order,
        LayerRuleEffect::AboveLock,
        LayerRuleEffect::NoScreenShare,
    ];
    assert_eq!(effects.len(), 10);
}
