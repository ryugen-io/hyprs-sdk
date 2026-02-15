#![cfg(feature = "wayland")]
use hypr_sdk::protocols::global_shortcuts::*;

#[test]
fn shortcut_info_construction() {
    let info = ShortcutInfo {
        id: "toggle-mic".to_string(),
        description: "Toggle microphone".to_string(),
        preferred_trigger: "SUPER+M".to_string(),
    };
    assert_eq!(info.id, "toggle-mic");
}

#[test]
fn shortcut_event_construction() {
    let ev = ShortcutEvent {
        id: "toggle-mic".to_string(),
        time: 12345,
    };
    assert_eq!(ev.time, 12345);
}
