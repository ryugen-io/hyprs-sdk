use hypr_sdk::types::common::WindowAddress;
use hypr_sdk::types::layer::{LayerSurface, LayersResponse};

const SAMPLE_JSON: &str = r#"{
    "DP-1": {
        "levels": {
            "0": [],
            "1": [],
            "2": [
                {
                    "address": "0xaaa",
                    "x": 0,
                    "y": 0,
                    "w": 2560,
                    "h": 40,
                    "namespace": "waybar",
                    "pid": 1234
                }
            ],
            "3": [
                {
                    "address": "0xbbb",
                    "x": 0,
                    "y": 0,
                    "w": 2560,
                    "h": 1440,
                    "namespace": "hyprlock",
                    "pid": 5678
                }
            ]
        }
    }
}"#;

#[test]
fn deserialize_layers_response() {
    let resp: LayersResponse = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert!(resp.0.contains_key("DP-1"));
    let dp1 = &resp.0["DP-1"];
    assert!(dp1.levels.contains_key("2"));
    assert_eq!(dp1.levels["2"].len(), 1);
    assert_eq!(dp1.levels["2"][0].namespace, "waybar");
}

#[test]
fn deserialize_layer_surface() {
    let json = r#"{
        "address": "0xaaa",
        "x": 10,
        "y": 20,
        "w": 800,
        "h": 40,
        "namespace": "waybar",
        "pid": 1234
    }"#;
    let ls: LayerSurface = serde_json::from_str(json).unwrap();
    assert_eq!(ls.address, WindowAddress(0xaaa));
    assert_eq!(ls.x, 10);
    assert_eq!(ls.y, 20);
    assert_eq!(ls.w, 800);
    assert_eq!(ls.h, 40);
    assert_eq!(ls.namespace, "waybar");
    assert_eq!(ls.pid, 1234);
}

#[test]
fn deserialize_empty_monitor_layers() {
    let json = r#"{
        "HDMI-A-1": {
            "levels": {
                "0": [],
                "1": [],
                "2": [],
                "3": []
            }
        }
    }"#;
    let resp: LayersResponse = serde_json::from_str(json).unwrap();
    let hdmi = &resp.0["HDMI-A-1"];
    assert!(hdmi.levels["0"].is_empty());
}

#[test]
fn deserialize_multi_monitor_layers() {
    let json = r#"{
        "DP-1": { "levels": { "0": [], "1": [], "2": [], "3": [] } },
        "DP-2": { "levels": { "0": [], "1": [], "2": [], "3": [] } }
    }"#;
    let resp: LayersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.0.len(), 2);
}

#[test]
fn layers_response_ignores_unknown_fields() {
    let json = r#"{
        "DP-1": {
            "levels": { "0": [], "1": [], "2": [], "3": [] },
            "futureField": true
        }
    }"#;
    let resp: LayersResponse = serde_json::from_str(json).unwrap();
    assert!(resp.0.contains_key("DP-1"));
}

// WHY: Needed for correctness and maintainability: -- Plugin-only fields default --

#[test]
fn ipc_json_defaults_plugin_fields() {
    let json = r#"{
        "address": "0x1",
        "x": 0, "y": 0, "w": 100, "h": 100,
        "namespace": "test",
        "pid": 1
    }"#;
    let ls: LayerSurface = serde_json::from_str(json).unwrap();
    assert_eq!(ls.layer, 0);
    assert!(!ls.mapped);
}
