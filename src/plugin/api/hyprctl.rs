use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{HyprCtlOutputFormat, PluginHandle};

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

/// Allocate a copy of bytes using malloc (C++ bridge frees with `std::free`).
fn malloc_copy(data: &[u8]) -> *mut c_char {
    if data.is_empty() {
        return std::ptr::null_mut();
    }
    // SAFETY: Standard C malloc.
    unsafe {
        let ptr = malloc(data.len()).cast::<c_char>();
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
        }
        ptr
    }
}

/// Invoke a hyprctl command synchronously.
///
/// Equivalent to running `hyprctl <command> <args>` but from within the
/// compositor process (no IPC overhead).
///
/// # Errors
///
/// Returns [`HyprError::Plugin`] if the command fails.
pub fn invoke_hyprctl(
    command: &str,
    args: &str,
    format: HyprCtlOutputFormat,
) -> HyprResult<String> {
    let format_str = match format {
        HyprCtlOutputFormat::Normal => "",
        HyprCtlOutputFormat::Json => "j",
    };

    let mut out_ptr: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;

    // SAFETY: All string parameters are valid UTF-8 with correct lengths.
    let result = unsafe {
        ffi::invoke_hyprctl(
            command.as_ptr().cast::<c_char>(),
            command.len(),
            args.as_ptr().cast::<c_char>(),
            args.len(),
            format_str.as_ptr().cast::<c_char>(),
            format_str.len(),
            &mut out_ptr,
            &mut out_len,
        )
    };

    if !result {
        return Err(HyprError::Plugin("hyprctl invocation failed".into()));
    }

    if out_ptr.is_null() || out_len == 0 {
        return Ok(String::new());
    }

    // SAFETY: The C++ bridge allocated out_ptr with the given length via malloc.
    let output = unsafe {
        let slice = std::slice::from_raw_parts(out_ptr.cast::<u8>(), out_len);
        let s = String::from_utf8_lossy(slice).into_owned();
        // Free the bridge-allocated string.
        ffi::free_bridge_string(out_ptr);
        s
    };

    Ok(output)
}

/// Type alias for hyprctl command handler callbacks.
///
/// Receives the output format flag and the argument string. Returns
/// the command output string.
pub type HyprCtlCommandHandler = Box<dyn Fn(HyprCtlOutputFormat, &str) -> String + Send + 'static>;

/// Stored context for a hyprctl command trampoline.
struct HyprCtlCommandCallbackData {
    handler: HyprCtlCommandHandler,
}

/// RAII guard that unregisters a custom hyprctl command on drop.
pub struct HyprCtlCommandGuard {
    handle: PluginHandle,
    command_ptr: *mut c_void,
}

impl std::fmt::Debug for HyprCtlCommandGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyprCtlCommandGuard")
            .field("command_ptr", &self.command_ptr)
            .finish()
    }
}

// SAFETY: Contains only stable pointers.
unsafe impl Send for HyprCtlCommandGuard {}

impl Drop for HyprCtlCommandGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() && !self.command_ptr.is_null() {
            // SAFETY: Unregistering our own command.
            unsafe {
                ffi::unregister_hyprctl_command(self.handle.0, self.command_ptr);
            }
        }
    }
}

/// Trampoline for custom hyprctl command callbacks.
///
/// # Safety
///
/// Called by the C++ bridge. `user_data` points to a `HyprCtlCommandCallbackData`.
unsafe extern "C" fn hyprctl_trampoline(
    user_data: *mut c_void,
    format: u8,
    args_ptr: *const c_char,
    args_len: usize,
    out_ptr: *mut *mut c_char,
    out_len: *mut usize,
) {
    if user_data.is_null() {
        // SAFETY: out_ptr and out_len are owned by the C++ caller and valid.
        unsafe {
            *out_ptr = std::ptr::null_mut();
            *out_len = 0;
        }
        return;
    }

    // SAFETY: user_data was created by Box::into_raw in register_hyprctl_command.
    let data = unsafe { &*(user_data as *const HyprCtlCommandCallbackData) };
    let args = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(args_ptr.cast(), args_len))
    };

    let fmt = if format == 1 {
        HyprCtlOutputFormat::Json
    } else {
        HyprCtlOutputFormat::Normal
    };

    let result = (data.handler)(fmt, args);

    // SAFETY: out_ptr and out_len are owned by the C++ caller and valid.
    unsafe {
        *out_ptr = malloc_copy(result.as_bytes());
        *out_len = result.len();
    }
}

/// Register a custom hyprctl command.
///
/// The command becomes available as `hyprctl <name>`.
///
/// # Arguments
///
/// - `exact` — If `true`, the command name must match exactly. If `false`,
///   it acts as a prefix match.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if registration fails.
pub fn register_hyprctl_command(
    handle: PluginHandle,
    name: &str,
    exact: bool,
    handler: HyprCtlCommandHandler,
) -> HyprResult<HyprCtlCommandGuard> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let data = Box::new(HyprCtlCommandCallbackData { handler });
    let data_ptr = Box::into_raw(data);

    // SAFETY: We validated the handle.
    let command_ptr = unsafe {
        ffi::register_hyprctl_command(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            exact,
            Some(hyprctl_trampoline),
            data_ptr.cast::<c_void>(),
        )
    };

    if command_ptr.is_null() {
        // SAFETY: We just created this pointer.
        unsafe {
            drop(Box::from_raw(data_ptr));
        }
        Err(HyprError::Plugin(format!(
            "failed to register hyprctl command: {name}"
        )))
    } else {
        // Note: data_ptr is intentionally leaked — it lives as long as the
        // command is registered. It gets cleaned up when the bridge data is
        // freed (on unregister or plugin exit).
        Ok(HyprCtlCommandGuard {
            handle,
            command_ptr,
        })
    }
}
