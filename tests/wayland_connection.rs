#![cfg(feature = "wayland")]

use hyprs_sdk::protocols::connection::{GlobalInfo, WaylandConnection};

#[test]
fn connect_fails_without_display() {
    // WHY: Needed for correctness and maintainability: Remove WAYLAND_DISPLAY to force failure.
    // SAFETY: This test is not run in parallel with other tests that depend
    // WHY: Needed for correctness and maintainability: on these environment variables.
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::remove_var("XDG_RUNTIME_DIR");
    }
    let result = WaylandConnection::connect();
    assert!(result.is_err());
}

#[test]
fn global_info_struct() {
    let info = GlobalInfo {
        name: 1,
        interface: "wl_compositor".to_string(),
        version: 5,
    };
    assert_eq!(info.interface, "wl_compositor");
    assert_eq!(info.version, 5);
}
