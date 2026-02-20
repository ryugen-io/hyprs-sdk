use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{FunctionMatch, PluginHandle};

/// Opaque handle to a function hook.
///
/// **WARNING**: Function hooking is unstable. Internal Hyprland functions
/// may change between any version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FunctionHookHandle(pub *mut c_void);

// SAFETY: FunctionHookHandle is an opaque pointer managed by Hyprland.
unsafe impl Send for FunctionHookHandle {}
unsafe impl Sync for FunctionHookHandle {}

impl FunctionHookHandle {
    /// Null (invalid) handle.
    pub const NULL: Self = Self(std::ptr::null_mut());

    /// Whether this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Create a function hook (trampoline) to an internal Hyprland function.
///
/// **WARNING**: No API stability guaranteed.
///
/// # Safety
///
/// `source` and `destination` must be valid function pointers.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if hook creation fails.
pub unsafe fn create_function_hook(
    handle: PluginHandle,
    source: *const c_void,
    destination: *const c_void,
) -> HyprResult<FunctionHookHandle> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: Caller guarantees source and destination are valid function pointers.
    let hook = unsafe { ffi::create_function_hook(handle.0, source, destination) };

    if hook.is_null() {
        Err(HyprError::Plugin("failed to create function hook".into()))
    } else {
        Ok(FunctionHookHandle(hook))
    }
}

/// Remove a function hook.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if removal fails.
pub fn remove_function_hook(handle: PluginHandle, hook: FunctionHookHandle) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }
    if hook.is_null() {
        return Err(HyprError::Plugin("null function hook handle".into()));
    }

    // SAFETY: We validated both handles.
    let result = unsafe { ffi::remove_function_hook(handle.0, hook.0) };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to remove function hook".into()))
    }
}

/// Find internal Hyprland functions by demangled name.
///
/// **WARNING**: Unstable API. Function names and addresses change between versions.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the search fails.
pub fn find_functions_by_name(handle: PluginHandle, name: &str) -> HyprResult<Vec<FunctionMatch>> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let mut out_addresses: *const c_void = std::ptr::null();
    let mut out_count: usize = 0;

    // SAFETY: We validated the handle. out pointers are valid.
    let result = unsafe {
        ffi::find_functions_by_name(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            &mut out_addresses,
            &mut out_count,
        )
    };

    if !result {
        return Err(HyprError::Plugin(format!(
            "failed to find functions: {name}"
        )));
    }

    let mut matches = Vec::with_capacity(out_count);
    if !out_addresses.is_null() && out_count > 0 {
        // SAFETY: The bridge allocated out_addresses with out_count entries.
        let addrs =
            unsafe { std::slice::from_raw_parts(out_addresses.cast::<*const c_void>(), out_count) };
        for &addr in addrs {
            matches.push(FunctionMatch {
                address: addr,
                signature: String::new(),
                demangled: name.to_owned(),
            });
        }
        // SAFETY: out_addresses was allocated by the C++ bridge.
        unsafe {
            ffi::free_bridge_array(out_addresses as *mut c_void);
        }
    }

    Ok(matches)
}
