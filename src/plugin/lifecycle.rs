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

#[doc(hidden)]
#[inline]
pub fn __ensure_lifecycle_bridge_linked() {
    #[cfg(feature = "plugin-ffi")]
    {
        // SAFETY: This function is a no-op marker used solely to force the
        // lifecycle bridge object file to be linked into plugin artifacts.
        unsafe {
            crate::plugin::ffi::lifecycle_bridge_marker();
        }
    }
}

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
/// - `hyprland_rs_plugin_api_version` — C-compatible API version accessor
/// - `hyprland_rs_plugin_init` — Calls your init function and stores metadata
/// - `hyprland_rs_plugin_get_description` — Returns init description fields
/// - `hyprland_rs_plugin_get_error` — Returns init error text, if any
/// - `hyprland_rs_plugin_exit` — Calls your exit function (if provided)
///
/// A C++ lifecycle bridge translates these shims to Hyprland's C++ ABI
/// (`pluginAPIVersion`, `pluginInit`, `pluginExit`).
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
        #[derive(Default)]
        struct __HyprPluginLifecycleState {
            description: Option<$crate::plugin::types::PluginDescription>,
            error: Option<::std::ffi::CString>,
        }

        static __HYPR_PLUGIN_LIFECYCLE_STATE: ::std::sync::Mutex<__HyprPluginLifecycleState> =
            ::std::sync::Mutex::new(__HyprPluginLifecycleState {
                description: None,
                error: None,
            });

        #[inline]
        fn __set_out_bytes(
            out_ptr: *mut *const ::std::os::raw::c_char,
            out_len: *mut usize,
            value: &[u8],
        ) {
            // SAFETY: The caller provides output pointers or null. We only
            // write to non-null pointers.
            unsafe {
                if !out_ptr.is_null() {
                    *out_ptr = value.as_ptr().cast();
                }
                if !out_len.is_null() {
                    *out_len = value.len();
                }
            }
        }

        #[inline]
        fn __set_out_none(out_ptr: *mut *const ::std::os::raw::c_char, out_len: *mut usize) {
            // SAFETY: The caller provides output pointers or null. We only
            // write to non-null pointers.
            unsafe {
                if !out_ptr.is_null() {
                    *out_ptr = ::std::ptr::null();
                }
                if !out_len.is_null() {
                    *out_len = 0;
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_api_version() -> *const ::std::os::raw::c_char {
            $crate::plugin::lifecycle::__ensure_lifecycle_bridge_linked();
            concat!($crate::plugin::types::HYPRLAND_API_VERSION, "\0")
                .as_ptr()
                .cast()
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_init(handle: *mut ::std::ffi::c_void) -> bool {
            $crate::plugin::lifecycle::__ensure_lifecycle_bridge_linked();
            let plugin_handle = $crate::plugin::types::PluginHandle(handle);
            let mut state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            match $init_fn(plugin_handle) {
                Ok(desc) => {
                    state.description = Some(desc);
                    state.error = None;
                    true
                }
                Err(err) => {
                    let cleaned = err.replace('\0', " ");
                    let c_error = match ::std::ffi::CString::new(cleaned) {
                        Ok(v) => v,
                        Err(_) => ::std::ffi::CString::new("plugin init failed")
                            .expect("static error string cannot contain NUL"),
                    };
                    state.description = None;
                    state.error = Some(c_error);
                    false
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_get_description(
            out_name_ptr: *mut *const ::std::os::raw::c_char,
            out_name_len: *mut usize,
            out_description_ptr: *mut *const ::std::os::raw::c_char,
            out_description_len: *mut usize,
            out_author_ptr: *mut *const ::std::os::raw::c_char,
            out_author_len: *mut usize,
            out_version_ptr: *mut *const ::std::os::raw::c_char,
            out_version_len: *mut usize,
        ) -> bool {
            let state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            if let Some(ref desc) = state.description {
                __set_out_bytes(out_name_ptr, out_name_len, desc.name.as_bytes());
                __set_out_bytes(
                    out_description_ptr,
                    out_description_len,
                    desc.description.as_bytes(),
                );
                __set_out_bytes(out_author_ptr, out_author_len, desc.author.as_bytes());
                __set_out_bytes(out_version_ptr, out_version_len, desc.version.as_bytes());
                true
            } else {
                __set_out_none(out_name_ptr, out_name_len);
                __set_out_none(out_description_ptr, out_description_len);
                __set_out_none(out_author_ptr, out_author_len);
                __set_out_none(out_version_ptr, out_version_len);
                false
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_get_error(
            out_error_ptr: *mut *const ::std::os::raw::c_char,
            out_error_len: *mut usize,
        ) -> bool {
            let state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            if let Some(ref err) = state.error {
                __set_out_bytes(out_error_ptr, out_error_len, err.as_bytes());
                true
            } else {
                __set_out_none(out_error_ptr, out_error_len);
                false
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_exit() {
            $exit_fn();
            let mut state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");
            state.description = None;
            state.error = None;
        }
    };
    ($init_fn:path) => {
        #[derive(Default)]
        struct __HyprPluginLifecycleState {
            description: Option<$crate::plugin::types::PluginDescription>,
            error: Option<::std::ffi::CString>,
        }

        static __HYPR_PLUGIN_LIFECYCLE_STATE: ::std::sync::Mutex<__HyprPluginLifecycleState> =
            ::std::sync::Mutex::new(__HyprPluginLifecycleState {
                description: None,
                error: None,
            });

        #[inline]
        fn __set_out_bytes(
            out_ptr: *mut *const ::std::os::raw::c_char,
            out_len: *mut usize,
            value: &[u8],
        ) {
            // SAFETY: The caller provides output pointers or null. We only
            // write to non-null pointers.
            unsafe {
                if !out_ptr.is_null() {
                    *out_ptr = value.as_ptr().cast();
                }
                if !out_len.is_null() {
                    *out_len = value.len();
                }
            }
        }

        #[inline]
        fn __set_out_none(out_ptr: *mut *const ::std::os::raw::c_char, out_len: *mut usize) {
            // SAFETY: The caller provides output pointers or null. We only
            // write to non-null pointers.
            unsafe {
                if !out_ptr.is_null() {
                    *out_ptr = ::std::ptr::null();
                }
                if !out_len.is_null() {
                    *out_len = 0;
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_api_version() -> *const ::std::os::raw::c_char {
            $crate::plugin::lifecycle::__ensure_lifecycle_bridge_linked();
            concat!($crate::plugin::types::HYPRLAND_API_VERSION, "\0")
                .as_ptr()
                .cast()
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_init(handle: *mut ::std::ffi::c_void) -> bool {
            $crate::plugin::lifecycle::__ensure_lifecycle_bridge_linked();
            let plugin_handle = $crate::plugin::types::PluginHandle(handle);
            let mut state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            match $init_fn(plugin_handle) {
                Ok(desc) => {
                    state.description = Some(desc);
                    state.error = None;
                    true
                }
                Err(err) => {
                    let cleaned = err.replace('\0', " ");
                    let c_error = match ::std::ffi::CString::new(cleaned) {
                        Ok(v) => v,
                        Err(_) => ::std::ffi::CString::new("plugin init failed")
                            .expect("static error string cannot contain NUL"),
                    };
                    state.description = None;
                    state.error = Some(c_error);
                    false
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_get_description(
            out_name_ptr: *mut *const ::std::os::raw::c_char,
            out_name_len: *mut usize,
            out_description_ptr: *mut *const ::std::os::raw::c_char,
            out_description_len: *mut usize,
            out_author_ptr: *mut *const ::std::os::raw::c_char,
            out_author_len: *mut usize,
            out_version_ptr: *mut *const ::std::os::raw::c_char,
            out_version_len: *mut usize,
        ) -> bool {
            let state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            if let Some(ref desc) = state.description {
                __set_out_bytes(out_name_ptr, out_name_len, desc.name.as_bytes());
                __set_out_bytes(
                    out_description_ptr,
                    out_description_len,
                    desc.description.as_bytes(),
                );
                __set_out_bytes(out_author_ptr, out_author_len, desc.author.as_bytes());
                __set_out_bytes(out_version_ptr, out_version_len, desc.version.as_bytes());
                true
            } else {
                __set_out_none(out_name_ptr, out_name_len);
                __set_out_none(out_description_ptr, out_description_len);
                __set_out_none(out_author_ptr, out_author_len);
                __set_out_none(out_version_ptr, out_version_len);
                false
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_get_error(
            out_error_ptr: *mut *const ::std::os::raw::c_char,
            out_error_len: *mut usize,
        ) -> bool {
            let state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");

            if let Some(ref err) = state.error {
                __set_out_bytes(out_error_ptr, out_error_len, err.as_bytes());
                true
            } else {
                __set_out_none(out_error_ptr, out_error_len);
                false
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn hyprland_rs_plugin_exit() {
            let mut state = __HYPR_PLUGIN_LIFECYCLE_STATE
                .lock()
                .expect("plugin lifecycle state poisoned");
            state.description = None;
            state.error = None;
        }
    };
}
