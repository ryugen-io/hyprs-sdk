//! Plugin lifecycle macros.
//!
//! Generates the `extern "C"` entry points that Hyprland expects when
//! loading a plugin shared library (`pluginAPIVersion`, `pluginInit`,
//! `pluginExit`).
//!
//! # ABI Note
//!
//! The actual Hyprland plugin entry points return C++ types
//! (`std::string`, `PLUGIN_DESCRIPTION_INFO`). This macro generates
//! C-compatible shims. A companion C++ translation unit is required
//! to bridge between the C signatures and the C++ types Hyprland
//! expects.
//!
//! # Example
//!
//! ```rust,ignore
//! use hypr_sdk::plugin::*;
//!
//! fn my_init(handle: PluginHandle) -> Result<PluginDescription, String> {
//!     // Register config values, hooks, dispatchers...
//!     Ok(PluginDescription {
//!         name: "my-plugin".into(),
//!         description: "Does cool things".into(),
//!         author: "Me".into(),
//!         version: "0.1.0".into(),
//!     })
//! }
//!
//! fn my_exit() {
//!     // Cleanup...
//! }
//!
//! hyprland_plugin! {
//!     init: my_init,
//!     exit: my_exit,
//! }
//! ```

/// Generate the plugin entry points that Hyprland resolves via `dlsym`.
///
/// # Parameters
///
/// - `init: <fn_name>` — Function with signature
///   `fn(PluginHandle) -> Result<PluginDescription, String>`.
///   Called during plugin initialization. Must register all config
///   values, hooks, and dispatchers synchronously.
///
/// - `exit: <fn_name>` (optional) — Function with signature `fn()`.
///   Called on user-initiated unload. Hooks are cleaned up automatically
///   after this returns.
///
/// # Generated Symbols
///
/// - `pluginAPIVersion` — Returns `HYPRLAND_API_VERSION` ("0.1")
/// - `pluginInit` — Calls your init function, stores the handle
/// - `pluginExit` — Calls your exit function (if provided)
/// - `__hyprland_api_get_client_hash` — Returns the ABI hash
///   (must match the Hyprland build)
#[macro_export]
macro_rules! hyprland_plugin {
    (
        init: $init_fn:path,
        exit: $exit_fn:path $(,)?
    ) => {
        $crate::_hyprland_plugin_impl!($init_fn, $exit_fn);
    };
    (
        init: $init_fn:path $(,)?
    ) => {
        $crate::_hyprland_plugin_impl!($init_fn);
    };
}

/// Internal implementation macro. Do not use directly.
#[doc(hidden)]
#[macro_export]
macro_rules! _hyprland_plugin_impl {
    ($init_fn:path, $exit_fn:path) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginAPIVersion() -> *const u8 {
            concat!($crate::plugin::types::HYPRLAND_API_VERSION, "\0").as_ptr()
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginInit(handle: *mut ::std::ffi::c_void) -> *const u8 {
            static mut PLUGIN_HANDLE: $crate::plugin::types::PluginHandle =
                $crate::plugin::types::PluginHandle::NULL;

            // SAFETY: pluginInit is called once by Hyprland, single-threaded.
            unsafe {
                PLUGIN_HANDLE = $crate::plugin::types::PluginHandle(handle);
            }

            match $init_fn(unsafe { PLUGIN_HANDLE }) {
                Ok(_desc) => {
                    // The C++ bridge reads the PluginDescription from
                    // a known location. Return success marker.
                    ::std::ptr::null()
                }
                Err(_e) => {
                    // Return non-null to signal error
                    b"error\0".as_ptr()
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginExit() {
            $exit_fn();
        }
    };
    ($init_fn:path) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginAPIVersion() -> *const u8 {
            concat!($crate::plugin::types::HYPRLAND_API_VERSION, "\0").as_ptr()
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginInit(handle: *mut ::std::ffi::c_void) -> *const u8 {
            static mut PLUGIN_HANDLE: $crate::plugin::types::PluginHandle =
                $crate::plugin::types::PluginHandle::NULL;

            unsafe {
                PLUGIN_HANDLE = $crate::plugin::types::PluginHandle(handle);
            }

            match $init_fn(unsafe { PLUGIN_HANDLE }) {
                Ok(_desc) => ::std::ptr::null(),
                Err(_e) => b"error\0".as_ptr(),
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn pluginExit() {}
    };
}
