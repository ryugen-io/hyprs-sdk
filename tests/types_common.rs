use hypr_sdk::types::common::{MonitorId, WindowAddress, WorkspaceId};

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
