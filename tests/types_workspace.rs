use hypr_sdk::types::common::{FullscreenMode, MonitorId, WindowAddress, WorkspaceId};
use hypr_sdk::types::workspace::Workspace;

const SAMPLE_JSON: &str = r#"{
    "id": 1,
    "name": "1",
    "monitor": "DP-1",
    "monitorID": 0,
    "windows": 3,
    "hasfullscreen": false,
    "lastwindow": "0x55a3f2c0",
    "lastwindowtitle": "nvim CLAUDE.md",
    "ispersistent": false
}"#;

#[test]
fn deserialize_workspace() {
    let ws: Workspace = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert_eq!(ws.id, WorkspaceId(1));
    assert_eq!(ws.name, "1");
    assert_eq!(ws.monitor, "DP-1");
    assert_eq!(ws.monitor_id, Some(MonitorId(0)));
    assert_eq!(ws.windows, 3);
    assert!(!ws.has_fullscreen);
    assert_eq!(ws.last_window, WindowAddress(0x55a3f2c0));
    assert_eq!(ws.last_window_title, "nvim CLAUDE.md");
    assert!(!ws.is_persistent);
}

#[test]
fn deserialize_workspace_null_monitor() {
    let json = r#"{
        "id": -99,
        "name": "special:scratch",
        "monitor": "?",
        "monitorID": null,
        "windows": 0,
        "hasfullscreen": false,
        "lastwindow": "0x0",
        "lastwindowtitle": "",
        "ispersistent": false
    }"#;
    let ws: Workspace = serde_json::from_str(json).unwrap();
    assert_eq!(ws.id, WorkspaceId(-99));
    assert!(ws.id.is_special());
    assert_eq!(ws.monitor_id, None);
}

#[test]
fn deserialize_workspace_array() {
    let json = format!("[{},{}]", SAMPLE_JSON, SAMPLE_JSON);
    let workspaces: Vec<Workspace> = serde_json::from_str(&json).unwrap();
    assert_eq!(workspaces.len(), 2);
}

#[test]
fn workspace_ignores_unknown_fields() {
    let json = r#"{
        "id": 1,
        "name": "1",
        "monitor": "DP-1",
        "monitorID": 0,
        "windows": 0,
        "hasfullscreen": false,
        "lastwindow": "0x0",
        "lastwindowtitle": "",
        "ispersistent": false,
        "futureField": "should be ignored"
    }"#;
    let ws: Workspace = serde_json::from_str(json).unwrap();
    assert_eq!(ws.id, WorkspaceId(1));
}

#[test]
fn ipc_json_defaults_plugin_fields() {
    // WHY: Needed for correctness and maintainability: IPC JSON doesn't include plugin-only fields; they should default.
    let ws: Workspace = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert_eq!(ws.fullscreen_mode, FullscreenMode::None);
    assert!(!ws.is_special);
    assert!(!ws.default_floating);
    assert!(!ws.default_pseudo);
    assert!(!ws.visible);
    assert_eq!(ws.last_monitor, "");
}
