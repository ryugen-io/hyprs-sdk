use hypr_sdk::types::common::{MonitorId, WindowAddress, WorkspaceId, WorkspaceRef};
use hypr_sdk::types::monitor::Monitor;

const SAMPLE_JSON: &str = r#"{
    "id": 0,
    "name": "DP-1",
    "description": "Samsung Electric Company 27\" (DP-1)",
    "make": "Samsung Electric Company",
    "model": "Odyssey G7",
    "serial": "H4ZN500123",
    "width": 2560,
    "height": 1440,
    "physicalWidth": 597,
    "physicalHeight": 336,
    "refreshRate": 144.00000,
    "x": 0,
    "y": 0,
    "activeWorkspace": { "id": 1, "name": "1" },
    "specialWorkspace": { "id": 0, "name": "" },
    "reserved": [0, 40, 0, 0],
    "scale": 1.00,
    "transform": 0,
    "focused": true,
    "dpmsStatus": true,
    "vrr": false,
    "solitary": "0x0",
    "solitaryBlockedBy": 0,
    "activelyTearing": false,
    "tearingBlockedBy": 0,
    "directScanoutTo": "0x0",
    "directScanoutBlockedBy": 0,
    "disabled": false,
    "currentFormat": "DRM_FORMAT_XRGB8888",
    "mirrorOf": "none",
    "availableModes": ["2560x1440@144.00Hz", "1920x1080@60.00Hz"],
    "colorManagementPreset": "sRGB",
    "sdrBrightness": 1.00,
    "sdrSaturation": 1.00,
    "sdrMinLuminance": 0.20,
    "sdrMaxLuminance": 80
}"#;

// WHY: Needed for correctness and maintainability: -- Basic deserialization --

#[test]
fn deserialize_monitor() {
    let m: Monitor = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert_eq!(m.id, MonitorId(0));
    assert_eq!(m.name, "DP-1");
    assert_eq!(m.make, "Samsung Electric Company");
    assert_eq!(m.model, "Odyssey G7");
    assert_eq!(m.serial, "H4ZN500123");
    assert_eq!(m.width, 2560);
    assert_eq!(m.height, 1440);
    assert_eq!(m.physical_width, 597);
    assert_eq!(m.physical_height, 336);
    assert!((m.refresh_rate - 144.0).abs() < 0.01);
    assert_eq!(m.x, 0);
    assert_eq!(m.y, 0);
    assert_eq!(
        m.active_workspace,
        WorkspaceRef {
            id: WorkspaceId(1),
            name: "1".into()
        }
    );
    assert_eq!(
        m.special_workspace,
        WorkspaceRef {
            id: WorkspaceId(0),
            name: "".into()
        }
    );
    assert_eq!(m.reserved, [0, 40, 0, 0]);
    assert!((m.scale - 1.0).abs() < 0.01);
    assert_eq!(m.transform, 0);
    assert!(m.focused);
    assert!(m.dpms_status);
    assert!(!m.vrr);
    assert_eq!(m.solitary, WindowAddress(0));
    assert_eq!(m.solitary_blocked_by, 0);
    assert!(!m.actively_tearing);
    assert_eq!(m.tearing_blocked_by, 0);
    assert_eq!(m.direct_scanout_to, WindowAddress(0));
    assert_eq!(m.direct_scanout_blocked_by, 0);
    assert!(!m.disabled);
    assert_eq!(m.current_format, "DRM_FORMAT_XRGB8888");
    assert_eq!(m.mirror_of, "none");
    assert_eq!(
        m.available_modes,
        vec!["2560x1440@144.00Hz", "1920x1080@60.00Hz"]
    );
    assert_eq!(m.color_management_preset, "sRGB");
    assert!((m.sdr_brightness - 1.0).abs() < 0.01);
    assert!((m.sdr_saturation - 1.0).abs() < 0.01);
    assert!((m.sdr_min_luminance - 0.2).abs() < 0.01);
    assert_eq!(m.sdr_max_luminance, 80);
}

// WHY: Needed for correctness and maintainability: -- Edge cases --

#[test]
fn monitor_ignores_unknown_fields() {
    let mut json: serde_json::Value = serde_json::from_str(SAMPLE_JSON).unwrap();
    json["newField"] = serde_json::json!("future");
    let m: Monitor = serde_json::from_value(json).unwrap();
    assert_eq!(m.id, MonitorId(0));
}

#[test]
fn monitor_array() {
    let json = format!("[{}]", SAMPLE_JSON);
    let monitors: Vec<Monitor> = serde_json::from_str(&json).unwrap();
    assert_eq!(monitors.len(), 1);
}

#[test]
fn monitor_accepts_blocked_by_arrays_and_null() {
    let mut json: serde_json::Value = serde_json::from_str(SAMPLE_JSON).unwrap();
    json["solitaryBlockedBy"] = serde_json::Value::Null;
    json["tearingBlockedBy"] = serde_json::json!(["NOT_TORN", "WINDOW"]);
    json["directScanoutBlockedBy"] = serde_json::json!(["USER", "CANDIDATE"]);

    let m: Monitor = serde_json::from_value(json).unwrap();
    assert_eq!(m.solitary_blocked_by, 0);
    assert_eq!(m.tearing_blocked_by, 0);
    assert_eq!(m.direct_scanout_blocked_by, 0);
}

// WHY: Needed for correctness and maintainability: -- Plugin-only fields default --

#[test]
fn ipc_json_defaults_plugin_fields() {
    let m: Monitor = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert!(!m.enabled_10bit);
    assert!(!m.created_by_user);
}
