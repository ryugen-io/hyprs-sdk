use hypr_sdk::types::common::{
    ContentType, FullscreenMode, WindowAddress, WorkspaceId, WorkspaceRef,
};
use hypr_sdk::types::window::Window;

const SAMPLE_JSON: &str = r#"{
    "address": "0x55a3f2c0",
    "mapped": true,
    "hidden": false,
    "at": [100, 200],
    "size": [1920, 1080],
    "workspace": { "id": 1, "name": "1" },
    "floating": false,
    "pseudo": false,
    "monitor": 0,
    "class": "kitty",
    "title": "nvim",
    "initialClass": "kitty",
    "initialTitle": "fish",
    "pid": 12345,
    "xwayland": false,
    "pinned": false,
    "fullscreen": 0,
    "fullscreenClient": 0,
    "overFullscreen": false,
    "grouped": [],
    "tags": [],
    "swallowing": "0x0",
    "focusHistoryID": 0,
    "inhibitingIdle": false,
    "xdgTag": "",
    "xdgDescription": "",
    "contentType": "none"
}"#;

// WHY: Needed for correctness and maintainability: -- Basic deserialization --

#[test]
fn deserialize_window() {
    let w: Window = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert_eq!(w.address, WindowAddress(0x55a3f2c0));
    assert!(w.mapped);
    assert!(!w.hidden);
    assert_eq!(w.position, [100, 200]);
    assert_eq!(w.size, [1920, 1080]);
    assert_eq!(
        w.workspace,
        WorkspaceRef {
            id: WorkspaceId(1),
            name: "1".into()
        }
    );
    assert!(!w.floating);
    assert!(!w.pseudo);
    assert_eq!(w.monitor, 0);
    assert_eq!(w.class, "kitty");
    assert_eq!(w.title, "nvim");
    assert_eq!(w.initial_class, "kitty");
    assert_eq!(w.initial_title, "fish");
    assert_eq!(w.pid, 12345);
    assert!(!w.xwayland);
    assert!(!w.pinned);
    assert_eq!(w.fullscreen, FullscreenMode::None);
    assert_eq!(w.fullscreen_client, FullscreenMode::None);
    assert!(!w.over_fullscreen);
    assert!(w.grouped.is_empty());
    assert!(w.tags.is_empty());
    assert_eq!(w.swallowing, WindowAddress(0));
    assert_eq!(w.focus_history_id, 0);
    assert!(!w.inhibiting_idle);
    assert_eq!(w.xdg_tag, "");
    assert_eq!(w.xdg_description, "");
    assert_eq!(w.content_type, ContentType::None);
}

// WHY: Needed for correctness and maintainability: -- Edge cases --

#[test]
fn window_with_grouped_and_tags() {
    let json = r#"{
        "address": "0xabc",
        "mapped": true,
        "hidden": false,
        "at": [0, 0],
        "size": [800, 600],
        "workspace": { "id": 2, "name": "browser" },
        "floating": true,
        "pseudo": false,
        "monitor": 1,
        "class": "firefox",
        "title": "GitHub",
        "initialClass": "firefox",
        "initialTitle": "Mozilla Firefox",
        "pid": 9999,
        "xwayland": false,
        "pinned": true,
        "fullscreen": 2,
        "fullscreenClient": 1,
        "overFullscreen": false,
        "grouped": ["0xabc", "0xdef"],
        "tags": ["browser", "important"],
        "swallowing": "0xfeed",
        "focusHistoryID": 3,
        "inhibitingIdle": true,
        "xdgTag": "app-tag",
        "xdgDescription": "A browser",
        "contentType": "video"
    }"#;
    let w: Window = serde_json::from_str(json).unwrap();
    assert!(w.floating);
    assert!(w.pinned);
    assert_eq!(w.fullscreen, FullscreenMode::Fullscreen);
    assert_eq!(w.fullscreen_client, FullscreenMode::Maximized);
    assert_eq!(w.grouped, vec![WindowAddress(0xabc), WindowAddress(0xdef)]);
    assert_eq!(w.tags, vec!["browser", "important"]);
    assert_eq!(w.swallowing, WindowAddress(0xfeed));
    assert_eq!(w.focus_history_id, 3);
    assert!(w.inhibiting_idle);
    assert_eq!(w.content_type, ContentType::Video);
}

#[test]
fn window_ignores_unknown_fields() {
    let json = r#"{
        "address": "0x1",
        "mapped": false,
        "hidden": false,
        "at": [0, 0],
        "size": [0, 0],
        "workspace": { "id": -1, "name": "" },
        "floating": false,
        "pseudo": false,
        "monitor": -1,
        "class": "",
        "title": "",
        "initialClass": "",
        "initialTitle": "",
        "pid": 0,
        "xwayland": false,
        "pinned": false,
        "fullscreen": 0,
        "fullscreenClient": 0,
        "overFullscreen": false,
        "grouped": [],
        "tags": [],
        "swallowing": "0x0",
        "focusHistoryID": -1,
        "inhibitingIdle": false,
        "xdgTag": "",
        "xdgDescription": "",
        "contentType": "",
        "newFutureField": 42
    }"#;
    let w: Window = serde_json::from_str(json).unwrap();
    assert_eq!(w.address, WindowAddress(1));
}

// WHY: Needed for correctness and maintainability: -- Plugin-only fields default --

#[test]
fn ipc_json_defaults_plugin_fields() {
    let w: Window = serde_json::from_str(SAMPLE_JSON).unwrap();
    assert!(!w.is_urgent);
    assert!(!w.tearing_hint);
    assert!(!w.no_initial_focus);
    assert!(!w.x11_doesnt_want_borders);
    assert!(!w.requests_float);
    assert!(!w.group_head);
    assert!(!w.group_locked);
}
