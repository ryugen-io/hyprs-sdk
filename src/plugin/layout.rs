//! Layout algorithm registration.
//!
//! Hyprland v0.54 split the monolithic `IHyprLayout` into two interfaces:
//! [`tiled::TiledAlgorithm`] for window tiling and [`floating::FloatingAlgorithm`]
//! for floating window placement. The old API remains in [`legacy`] for plugins
//! targeting older versions.

pub mod common;
#[allow(unsafe_code)]
pub mod floating;
#[allow(unsafe_code)]
pub mod legacy;
#[allow(unsafe_code)]
pub mod tiled;

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::types::PluginHandle;

pub use common::{Direction, FocalPoint, ModeAlgorithm, RectCorner};
pub use floating::{FloatingAlgorithm, FloatingAlgorithmFactory, register_floating_algo};
pub use legacy::{Layout, LayoutHandle, register_layout, unregister_layout};
pub use tiled::{TiledAlgorithm, TiledAlgorithmFactory, register_tiled_algo};

unsafe extern "C" {
    /// Shared by both tiled and floating — v0.54 uses a single `removeAlgo` for both.
    #[link_name = "hyprland_api_remove_algo"]
    fn ffi_remove_algo(handle: *mut c_void, name_ptr: *const c_char, name_len: usize) -> bool;
}

/// Remove a tiled or floating algorithm by name (v0.54+ API).
///
/// Works for algorithms registered via [`register_tiled_algo`] or
/// [`register_floating_algo`].
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the algorithm was not found.
pub fn remove_algo(handle: PluginHandle, name: &str) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let ok = unsafe { ffi_remove_algo(handle.0, name.as_ptr().cast(), name.len()) };

    if ok {
        Ok(())
    } else {
        Err(HyprError::Plugin(format!("failed to remove algo: {name}")))
    }
}
