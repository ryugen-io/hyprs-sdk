#![cfg(feature = "wayland")]
use hypr_sdk::protocols::hyprland_surface::*;

#[test]
fn surface_handle_debug_format() {
    // SurfaceHandle and HyprlandSurfaceClient implement Debug.
    // We cannot construct them without a live Wayland connection,
    // but we verify the types are accessible and Debug is derived.
    let _: fn(&HyprlandSurfaceClient) -> String = |c| format!("{c:?}");
    let _: fn(&SurfaceHandle) -> String = |s| format!("{s:?}");
}
