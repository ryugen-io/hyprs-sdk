use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::hooks::HookEvent;
use crate::plugin::types::PluginHandle;

/// Type alias for hook callbacks.
///
/// The callback receives opaque pointers to the callback info and event data.
/// Use the event's documentation to determine the data type.
pub type HookCallback = Box<dyn Fn(*mut c_void, *mut c_void) + Send + 'static>;

/// Stored context for a hook callback trampoline.
struct HookCallbackData {
    callback: HookCallback,
}

/// Trampoline that bridges the C callback to a Rust closure.
///
/// # Safety
///
/// Called by the C++ bridge with `user_data` pointing to a `HookCallbackData`.
unsafe extern "C" fn hook_trampoline(
    user_data: *mut c_void,
    callback_info: *mut c_void,
    event_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    // SAFETY: user_data was created by Box::into_raw in register_hook.
    let data = unsafe { &*(user_data as *const HookCallbackData) };
    (data.callback)(callback_info, event_data);
}

/// RAII guard that unregisters a hook callback on drop.
pub struct HookCallbackGuard {
    handle: PluginHandle,
    callback_ptr: *mut c_void,
    callback_data: *mut HookCallbackData,
}

impl std::fmt::Debug for HookCallbackGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookCallbackGuard")
            .field("callback_ptr", &self.callback_ptr)
            .finish()
    }
}

// SAFETY: HookCallbackGuard contains only pointers that are stable
// for the plugin lifetime.
unsafe impl Send for HookCallbackGuard {}

impl Drop for HookCallbackGuard {
    fn drop(&mut self) {
        let unregistered = if self.handle.is_null() || self.callback_ptr.is_null() {
            false
        } else {
            // SAFETY: Unregistering our own callback.
            unsafe { ffi::unregister_callback(self.handle.0, self.callback_ptr) }
        };

        // Only reclaim callback data when we're sure the callback is no longer registered.
        // If unregistering fails we intentionally leak to avoid potential use-after-free.
        if !self.callback_data.is_null()
            && (unregistered || self.callback_ptr.is_null() || self.handle.is_null())
        {
            // SAFETY: We created this via Box::into_raw.
            unsafe {
                drop(Box::from_raw(self.callback_data));
            }
        }
    }
}

/// Register a callback for a hook event.
///
/// Returns a [`HookCallbackGuard`] that unregisters the callback on drop.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if registration fails.
pub fn register_hook(
    handle: PluginHandle,
    event: HookEvent,
    callback: HookCallback,
) -> HyprResult<HookCallbackGuard> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let data = Box::new(HookCallbackData { callback });
    let data_ptr = Box::into_raw(data);
    let event_name = event.event_name();

    // SAFETY: We validated the handle. The event name is a static &str.
    let callback_ptr = unsafe {
        ffi::register_callback(
            handle.0,
            event_name.as_ptr().cast::<c_char>(),
            event_name.len(),
            Some(hook_trampoline),
            data_ptr.cast::<c_void>(),
        )
    };

    if callback_ptr.is_null() {
        // SAFETY: We just created this pointer.
        unsafe {
            drop(Box::from_raw(data_ptr));
        }
        Err(HyprError::Plugin(format!(
            "failed to register hook: {event_name}"
        )))
    } else {
        Ok(HookCallbackGuard {
            handle,
            callback_ptr,
            callback_data: data_ptr,
        })
    }
}
