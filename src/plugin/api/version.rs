use std::ffi::CStr;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{PluginHandle, VersionInfo};

/// Get Hyprland version information.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the query fails.
pub fn get_version(handle: PluginHandle) -> HyprResult<VersionInfo> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let mut hash_ptr: *const c_char = std::ptr::null();
    let mut hash_len: usize = 0;
    let mut tag_ptr: *const c_char = std::ptr::null();
    let mut tag_len: usize = 0;
    let mut dirty: bool = false;
    let mut branch_ptr: *const c_char = std::ptr::null();
    let mut branch_len: usize = 0;

    // SAFETY: We validated the handle. Out pointers are valid.
    let result = unsafe {
        ffi::get_version(
            handle.0,
            &mut hash_ptr,
            &mut hash_len,
            &mut tag_ptr,
            &mut tag_len,
            &mut dirty,
            &mut branch_ptr,
            &mut branch_len,
        )
    };

    if !result {
        return Err(HyprError::Plugin("failed to get version".into()));
    }

    // SAFETY: The C++ bridge provides valid UTF-8 strings in static buffers.
    let hash = if hash_ptr.is_null() {
        String::new()
    } else {
        unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(hash_ptr.cast::<u8>(), hash_len))
                .into_owned()
        }
    };

    let tag = if tag_ptr.is_null() {
        String::new()
    } else {
        unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(tag_ptr.cast::<u8>(), tag_len))
                .into_owned()
        }
    };

    let branch = if branch_ptr.is_null() {
        String::new()
    } else {
        unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(
                branch_ptr.cast::<u8>(),
                branch_len,
            ))
            .into_owned()
        }
    };

    Ok(VersionInfo {
        hash,
        tag,
        dirty,
        branch,
        message: String::new(),
        commits: String::new(),
    })
}

/// Get the Hyprland server's ABI hash string.
///
/// This must match the client plugin's hash for the plugin to load.
///
/// # Errors
///
/// Returns [`HyprError::Plugin`] if the hash pointer is null.
pub fn get_server_hash() -> HyprResult<String> {
    // SAFETY: This is a global function with no handle parameter.
    let ptr = unsafe { ffi::get_server_hash() };

    if ptr.is_null() {
        return Err(HyprError::Plugin("server hash not available".into()));
    }

    // SAFETY: The returned pointer is a static C string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_string_lossy().into_owned())
}
