#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::ext_foreign_toplevel_list::*;

#[test]
fn ext_toplevel_info_defaults() {
    let info = ExtToplevelInfo::default();
    assert!(info.identifier.is_empty());
    assert!(info.title.is_empty());
    assert!(info.app_id.is_empty());
    assert!(!info.closed);
}

#[test]
fn ext_toplevel_info_clone() {
    let info = ExtToplevelInfo {
        identifier: "abc-123".into(),
        title: "My Window".into(),
        app_id: "org.example.app".into(),
        closed: false,
    };
    let cloned = info.clone();
    assert_eq!(cloned.identifier, "abc-123");
    assert_eq!(cloned.title, "My Window");
    assert_eq!(cloned.app_id, "org.example.app");
    assert!(!cloned.closed);
}

#[test]
fn ext_toplevel_info_debug() {
    let info = ExtToplevelInfo {
        identifier: "test-id".into(),
        title: "Test".into(),
        app_id: "test.app".into(),
        closed: true,
    };
    let debug = format!("{info:?}");
    assert!(debug.contains("ExtToplevelInfo"));
    assert!(debug.contains("test-id"));
    assert!(debug.contains("true"));
}
