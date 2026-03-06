#![cfg(feature = "wayland")]

use hyprs_sdk::HyprError;

#[test]
fn protocol_not_supported_error() {
    let err = HyprError::ProtocolNotSupported("zwlr_layer_shell_v1".to_string());
    let msg = err.to_string();
    assert!(msg.contains("zwlr_layer_shell_v1"));
}

#[test]
fn wayland_connect_error_display() {
    let err = HyprError::WaylandConnect("no display".to_string());
    assert!(err.to_string().contains("no display"));
}
