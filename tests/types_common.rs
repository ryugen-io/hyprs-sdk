use hyprs_sdk::types::common::{
    ContentType, FullscreenMode, Layer, MonitorId, OutputTransform, WindowAddress, WorkspaceId,
    WorkspaceRef,
};

#[test]
fn window_address_from_hex_string() {
    let addr: WindowAddress = "0x55a3f2c0".parse().unwrap();
    assert_eq!(addr.0, 0x55a3f2c0);
}

#[test]
fn window_address_display_hex() {
    let addr = WindowAddress(0x55a3f2c0);
    assert_eq!(addr.to_string(), "0x55a3f2c0");
}

#[test]
fn window_address_serde_roundtrip() {
    let addr = WindowAddress(0xdead);
    let json = serde_json::to_string(&addr).unwrap();
    assert_eq!(json, "\"0xdead\"");
    let back: WindowAddress = serde_json::from_str(&json).unwrap();
    assert_eq!(back, addr);
}

#[test]
fn workspace_id_serde() {
    let id = WorkspaceId(3);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "3");
    let back: WorkspaceId = serde_json::from_str(&json).unwrap();
    assert_eq!(back, id);
}

#[test]
fn monitor_id_serde() {
    let id = MonitorId(0);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "0");
}

#[test]
fn workspace_id_special() {
    let special = WorkspaceId::SPECIAL;
    assert!(special.is_special());
    assert!(!WorkspaceId(1).is_special());
}

#[test]
fn workspace_id_valid() {
    assert!(!WorkspaceId::INVALID.is_valid());
    assert!(WorkspaceId(1).is_valid());
}

// WHY: Needed for correctness and maintainability: -- WorkspaceRef --

#[test]
fn workspace_ref_serde() {
    let r = WorkspaceRef {
        id: WorkspaceId(1),
        name: "default".into(),
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: WorkspaceRef = serde_json::from_str(&json).unwrap();
    assert_eq!(back, r);
}

#[test]
fn workspace_ref_from_ipc_json() {
    let json = r#"{"id": 2, "name": "browser"}"#;
    let r: WorkspaceRef = serde_json::from_str(json).unwrap();
    assert_eq!(r.id, WorkspaceId(2));
    assert_eq!(r.name, "browser");
}

// WHY: Needed for correctness and maintainability: -- FullscreenMode --

#[test]
fn fullscreen_mode_serde_roundtrip() {
    for (mode, raw) in [
        (FullscreenMode::None, 0i8),
        (FullscreenMode::Maximized, 1),
        (FullscreenMode::Fullscreen, 2),
        (FullscreenMode::MaximizedFullscreen, 3),
    ] {
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, raw.to_string());
        let back: FullscreenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

#[test]
fn fullscreen_mode_queries() {
    assert!(FullscreenMode::Fullscreen.is_fullscreen());
    assert!(FullscreenMode::MaximizedFullscreen.is_fullscreen());
    assert!(!FullscreenMode::Maximized.is_fullscreen());
    assert!(FullscreenMode::Maximized.is_maximized());
    assert!(!FullscreenMode::Fullscreen.is_maximized());
}

#[test]
fn fullscreen_mode_unknown_raw_defaults() {
    assert_eq!(FullscreenMode::from_raw(99), FullscreenMode::None);
}

// WHY: Needed for correctness and maintainability: -- OutputTransform --

#[test]
fn output_transform_default() {
    assert_eq!(OutputTransform::default(), OutputTransform::Normal);
}

// WHY: Needed for correctness and maintainability: -- Layer --

#[test]
fn layer_from_raw() {
    assert_eq!(Layer::from_raw(0), Some(Layer::Background));
    assert_eq!(Layer::from_raw(3), Some(Layer::Overlay));
    assert_eq!(Layer::from_raw(99), None);
}

// WHY: Needed for correctness and maintainability: -- ContentType --

#[test]
fn content_type_serde_roundtrip() {
    let ct = ContentType::Video;
    let json = serde_json::to_string(&ct).unwrap();
    assert_eq!(json, "\"video\"");
    let back: ContentType = serde_json::from_str(&json).unwrap();
    assert_eq!(back, ct);
}

#[test]
fn content_type_unknown_defaults_to_none() {
    let ct: ContentType = serde_json::from_str("\"whatever\"").unwrap();
    assert_eq!(ct, ContentType::None);
}
