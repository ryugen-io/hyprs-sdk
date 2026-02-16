#![cfg(feature = "wayland")]
use hypr_sdk::protocols::global_shortcuts::*;

#[test]
fn shortcut_info_construction() {
    let info = ShortcutInfo {
        id: "toggle-mic".to_string(),
        app_id: "my-app".to_string(),
        description: "Toggle microphone".to_string(),
        trigger_description: "SUPER+M".to_string(),
    };
    assert_eq!(info.id, "toggle-mic");
    assert_eq!(info.trigger_description, "SUPER+M");
}

#[test]
fn shortcut_event_construction() {
    let ev = ShortcutEvent {
        id: "toggle-mic".to_string(),
        kind: ShortcutEventKind::Pressed,
        timestamp_ns: 12345_000_000_000,
    };
    assert_eq!(ev.timestamp_ns, 12345_000_000_000);
    assert_eq!(ev.kind, ShortcutEventKind::Pressed);
}
