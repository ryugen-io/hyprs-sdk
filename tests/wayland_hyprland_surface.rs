#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::hyprland_surface::*;

#[test]
fn surface_handle_debug_format() {
    // WHY: Needed for correctness and maintainability: SurfaceHandle and HyprlandSurfaceClient implement Debug.
    // WHY: Needed for correctness and maintainability: We cannot construct them without a live Wayland connection,
    // WHY: Needed for correctness and maintainability: but we verify the types are accessible and Debug is derived.
    let _: fn(&HyprlandSurfaceClient) -> String = |c| format!("{c:?}");
    let _: fn(&SurfaceHandle) -> String = |s| format!("{s:?}");
}
