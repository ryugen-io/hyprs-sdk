#![cfg(feature = "wayland")]
use hypr_sdk::protocols::layer_shell::*;

#[test]
fn layer_ordering() {
    assert_eq!(ShellLayer::Background as u32, 0);
    assert_eq!(ShellLayer::Overlay as u32, 3);
}

#[test]
fn anchor_bitmask() {
    let anchor = Anchor::TOP | Anchor::LEFT | Anchor::RIGHT;
    assert!(anchor.contains(Anchor::TOP));
    assert!(!anchor.contains(Anchor::BOTTOM));
}

#[test]
fn anchor_horizontal_bar() {
    let bar = Anchor::TOP | Anchor::LEFT | Anchor::RIGHT;
    assert!(bar.is_horizontal_bar());
    assert!(!bar.is_vertical_bar());
}

#[test]
fn anchor_vertical_bar() {
    let bar = Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM;
    assert!(bar.is_vertical_bar());
    assert!(!bar.is_horizontal_bar());
}

#[test]
fn keyboard_interactivity_variants() {
    assert_eq!(KeyboardInteractivity::None as u32, 0);
    assert_eq!(KeyboardInteractivity::Exclusive as u32, 1);
    assert_eq!(KeyboardInteractivity::OnDemand as u32, 2);
}

#[test]
fn layer_surface_config_defaults() {
    let config = LayerSurfaceConfig::default();
    assert_eq!(config.layer, ShellLayer::Top);
    assert_eq!(config.width, 0);
    assert!(config.anchor.is_empty());
}

#[test]
fn layer_surface_config_taskbar() {
    let config = LayerSurfaceConfig {
        layer: ShellLayer::Top,
        namespace: "taskbar".to_string(),
        width: 0,
        height: 40,
        anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
        exclusive_zone: 40,
        keyboard_interactivity: KeyboardInteractivity::None,
        margin_top: 0,
        margin_bottom: 0,
        margin_left: 0,
        margin_right: 0,
    };
    assert_eq!(config.exclusive_zone, 40);
    assert!(config.anchor.contains(Anchor::BOTTOM));
}
