//! Safe wrappers for custom dispatcher registration.
//!
//! Dispatchers are custom keybind actions. Plugins register them during
//! `pluginInit` and they become available as `dispatch <name> <args>`.
//!
//! # RAII
//!
//! [`DispatcherGuard`] automatically unregisters the dispatcher when dropped.

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{DispatchResult, PluginHandle};

/// Type alias for dispatcher callback functions.
///
/// Receives the argument string from the keybind config and returns
/// a [`DispatchResult`] indicating success/failure.
pub type DispatcherFn = Box<dyn Fn(&str) -> DispatchResult + Send + 'static>;

/// Stored context for a dispatcher trampoline.
struct DispatcherCallbackData {
    callback: DispatcherFn,
}

/// Trampoline that bridges the C callback to a Rust closure.
///
/// # Safety
///
/// Called by the C++ bridge. `user_data` must point to a valid
/// `DispatcherCallbackData` created via `Box::into_raw`.
unsafe extern "C" fn dispatcher_trampoline(
    user_data: *mut c_void,
    args_ptr: *const c_char,
    args_len: usize,
    out_pass: *mut bool,
    out_success: *mut bool,
    out_error_ptr: *mut *mut c_char,
    out_error_len: *mut usize,
) {
    if user_data.is_null() {
        unsafe {
            *out_pass = false;
            *out_success = false;
        }
        return;
    }

    // SAFETY: user_data was created by Box::into_raw in register_dispatcher.
    let data = unsafe { &*(user_data as *const DispatcherCallbackData) };
    let args = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(args_ptr.cast(), args_len))
    };

    let result = (data.callback)(args);

    // SAFETY: out pointers are valid and provided by the C++ bridge.
    unsafe {
        *out_pass = result.pass_event;
        *out_success = result.success;

        // Error strings must be malloc'd because ownership crosses the FFI boundary --
        // the C++ bridge constructs a std::string from this buffer and frees it.
        if !result.error.is_empty() {
            let buf = malloc_copy(result.error.as_bytes());
            *out_error_ptr = buf;
            *out_error_len = result.error.len();
        } else {
            *out_error_ptr = std::ptr::null_mut();
            *out_error_len = 0;
        }
    }
}

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

/// Allocate a copy of bytes using malloc (C++ bridge frees it with `std::free`).
fn malloc_copy(data: &[u8]) -> *mut c_char {
    if data.is_empty() {
        return std::ptr::null_mut();
    }
    // SAFETY: Allocating memory with standard C malloc.
    unsafe {
        let ptr = malloc(data.len()).cast::<c_char>();
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
        }
        ptr
    }
}

/// RAII guard that unregisters a dispatcher on drop.
///
/// Created by [`register_dispatcher`]. The dispatcher remains active
/// as long as this guard is alive.
pub struct DispatcherGuard {
    handle: PluginHandle,
    name: String,
    _callback_data: *mut DispatcherCallbackData,
}

impl std::fmt::Debug for DispatcherGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatcherGuard")
            .field("name", &self.name)
            .finish()
    }
}

// SAFETY: DispatcherGuard only stores a PluginHandle (which is Send+Sync)
// and a string name. The callback data pointer is stable for the plugin lifetime.
unsafe impl Send for DispatcherGuard {}

impl Drop for DispatcherGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: We're in drop, unregistering the dispatcher we registered.
            unsafe {
                ffi::remove_dispatcher(
                    self.handle.0,
                    self.name.as_ptr().cast::<c_char>(),
                    self.name.len(),
                );
            }

            // SAFETY: We created this pointer via Box::into_raw in register_dispatcher.
            if !self._callback_data.is_null() {
                unsafe {
                    drop(Box::from_raw(self._callback_data));
                }
            }
        }
    }
}

/// Register a custom keybind dispatcher.
///
/// The dispatcher becomes available as `dispatch <name> <args>` in
/// Hyprland's config and via `hyprctl dispatch`.
///
/// Returns a [`DispatcherGuard`] that unregisters the dispatcher on drop.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub fn register_dispatcher(
    handle: PluginHandle,
    name: &str,
    callback: DispatcherFn,
) -> HyprResult<DispatcherGuard> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let data = Box::new(DispatcherCallbackData { callback });
    let data_ptr = Box::into_raw(data);

    // SAFETY: We validated the handle. The trampoline and data_ptr are
    // valid for the plugin's lifetime.
    let result = unsafe {
        ffi::add_dispatcher(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            Some(dispatcher_trampoline),
            data_ptr.cast::<c_void>(),
        )
    };

    if result {
        Ok(DispatcherGuard {
            handle,
            name: name.to_owned(),
            _callback_data: data_ptr,
        })
    } else {
        // The FFI call failed, so the C++ side never took ownership of data_ptr.
        // We must reclaim it here to avoid leaking the Box we created above.
        // SAFETY: We just created this pointer and nobody else holds it.
        unsafe {
            drop(Box::from_raw(data_ptr));
        }
        Err(HyprError::Plugin(format!(
            "failed to register dispatcher: {name}"
        )))
    }
}

/// Unregister a custom dispatcher by name.
///
/// Prefer dropping the [`DispatcherGuard`] instead of calling this directly.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the dispatcher was not found.
pub fn unregister_dispatcher(handle: PluginHandle, name: &str) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle. name_ptr/name_len are valid UTF-8.
    let result =
        unsafe { ffi::remove_dispatcher(handle.0, name.as_ptr().cast::<c_char>(), name.len()) };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin(format!(
            "failed to unregister dispatcher: {name}"
        )))
    }
}
