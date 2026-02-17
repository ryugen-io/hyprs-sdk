//! Safe wrappers for plugin config registration.
//!
//! Plugins register config values during `pluginInit`. Values live in the
//! `plugin:<name>:` namespace and persist for the plugin's lifetime.
//!
//! # ABI Note
//!
//! All functions in this module call through the C++ bridge shim. They are
//! only usable when linked into a Hyprland plugin shared library.

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::PluginHandle;

/// Default value for a plugin config option.
///
/// Maps to the `Hyprlang::CConfigValue` variants that Hyprland supports.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigDefault {
    /// Boolean value (`Hyprlang::INT` 0 or 1).
    Bool(bool),
    /// Integer value (`Hyprlang::INT`).
    Int(i64),
    /// Floating-point value (`Hyprlang::FLOAT`).
    Float(f64),
    /// String value (`Hyprlang::STRING`).
    String(std::string::String),
    /// Color value (`Hyprlang::INT` as RGBA u32).
    Color(u32),
    /// 2D vector (`Hyprlang::VEC2`).
    Vec2(f64, f64),
}

/// Opaque handle to a live config value.
///
/// Obtained via [`get_config_handle`] and valid for the plugin's lifetime.
/// The pointer refers to a `Hyprlang::CConfigValue` inside Hyprland's
/// config system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ConfigValueHandle(pub *mut c_void);

// SAFETY: ConfigValueHandle is an opaque pointer managed by Hyprland.
// It is only used for reading config values and is valid for the plugin lifetime.
unsafe impl Send for ConfigValueHandle {}
unsafe impl Sync for ConfigValueHandle {}

impl ConfigValueHandle {
    /// Null (invalid) handle.
    pub const NULL: Self = Self(std::ptr::null_mut());

    /// Whether this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Register a config value with Hyprland's config system.
///
/// Must be called during `pluginInit` only. The value name should be
/// in the `plugin:<plugin_name>:<key>` format.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub fn register_config_value(
    handle: PluginHandle,
    name: &str,
    default: &ConfigDefault,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let (value_type, int_val, float_val, float_val2, str_data): (u8, i64, f64, f64, &str) =
        match default {
            ConfigDefault::Bool(b) => (0, i64::from(*b), 0.0, 0.0, ""),
            ConfigDefault::Int(i) => (1, *i, 0.0, 0.0, ""),
            ConfigDefault::Float(f) => (2, 0, *f, 0.0, ""),
            ConfigDefault::String(s) => (3, 0, 0.0, 0.0, s.as_str()),
            ConfigDefault::Color(c) => (4, i64::from(*c), 0.0, 0.0, ""),
            ConfigDefault::Vec2(x, y) => (5, 0, *x, *y, ""),
        };

    // SAFETY: We validated the handle is non-null. All string parameters
    // are valid UTF-8 with correct lengths.
    let result = unsafe {
        ffi::add_config_value(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            value_type,
            int_val,
            float_val,
            float_val2,
            str_data.as_ptr().cast::<c_char>(),
            str_data.len(),
        )
    };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin(format!(
            "failed to register config value: {name}"
        )))
    }
}

/// Get a handle to a live config value.
///
/// The returned handle is valid for the plugin's lifetime and can be
/// used to read the current value.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the config value is not found.
pub fn get_config_handle(handle: PluginHandle, name: &str) -> HyprResult<ConfigValueHandle> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle is non-null. The FFI call returns
    // a pointer valid for the plugin's lifetime.
    let ptr =
        unsafe { ffi::get_config_value(handle.0, name.as_ptr().cast::<c_char>(), name.len()) };

    if ptr.is_null() {
        Err(HyprError::Plugin(format!("config value not found: {name}")))
    } else {
        Ok(ConfigValueHandle(ptr))
    }
}

/// Options for a custom config keyword handler.
///
/// Maps to `Hyprlang::SHandlerOptions` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeywordHandlerOptions {
    /// Allow flags before the keyword value.
    pub allow_flags: bool,
}

/// Type alias for keyword handler callbacks.
///
/// The callback receives the raw value string from the config line.
/// Return `Ok(())` on success or `Err(message)` on failure.
pub type KeywordHandler = Box<dyn Fn(&str) -> Result<(), String> + Send + 'static>;

/// Stored context for a keyword handler trampoline.
struct KeywordCallbackData {
    handler: KeywordHandler,
}

/// Trampoline function that bridges C callback to Rust closure.
///
/// # Safety
///
/// Called by the C++ bridge with user_data pointing to a `KeywordCallbackData`.
unsafe extern "C" fn keyword_trampoline(
    user_data: *mut c_void,
    value_ptr: *const c_char,
    value_len: usize,
) -> bool {
    if user_data.is_null() {
        return false;
    }
    // SAFETY: user_data was created by Box::into_raw in register_config_keyword.
    let data = unsafe { &*(user_data as *const KeywordCallbackData) };
    let value = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(value_ptr.cast(), value_len))
    };

    (data.handler)(value).is_ok()
}

/// Register a custom config keyword handler.
///
/// Custom keywords let plugins define their own config syntax beyond
/// simple key-value pairs. Must be called during `pluginInit` only.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub fn register_config_keyword(
    handle: PluginHandle,
    name: &str,
    handler: KeywordHandler,
    options: KeywordHandlerOptions,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let data = Box::new(KeywordCallbackData { handler });
    let data_ptr = Box::into_raw(data);

    // SAFETY: We validated the handle is non-null. The trampoline function
    // pointer is valid for the plugin's lifetime.
    let result = unsafe {
        ffi::add_config_keyword(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            Some(keyword_trampoline),
            data_ptr.cast::<c_void>(),
            options.allow_flags,
            false, // allow_default not supported by Hyprlang
        )
    };

    if !result {
        // Reclaim on failure.
        // SAFETY: We just created this pointer and nobody else holds it.
        unsafe {
            drop(Box::from_raw(data_ptr));
        }
        return Err(HyprError::Plugin(format!(
            "failed to register config keyword: {name}"
        )));
    }

    // Note: data_ptr is intentionally leaked — it lives for the plugin's
    // lifetime. The C++ bridge stores the pointer in a global map.
    Ok(())
}
