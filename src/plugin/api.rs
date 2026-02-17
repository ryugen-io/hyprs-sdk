//! Safe wrappers for the Hyprland plugin API.
//!
//! Provides high-level Rust APIs for:
//! - Hook event registration
//! - HyprCtl command invocation and registration
//! - Notifications
//! - Config reload
//! - Function hooking (advanced/unstable)
//! - Version queries

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::hooks::HookEvent;
use crate::plugin::types::{
    FunctionMatch, HyprCtlOutputFormat, NotificationIcon, PluginHandle, VersionInfo,
};

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

// ── Hooks ────────────────────────────────────────────────────────────

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
    _callback_data: *mut HookCallbackData,
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
        if !self.handle.is_null() && !self.callback_ptr.is_null() {
            // SAFETY: Unregistering our own callback.
            unsafe {
                ffi::unregister_callback(self.handle.0, self.callback_ptr);
            }
        }
        if !self._callback_data.is_null() {
            // SAFETY: We created this via Box::into_raw.
            unsafe {
                drop(Box::from_raw(self._callback_data));
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
        // Reclaim on failure.
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
            _callback_data: data_ptr,
        })
    }
}

// ── HyprCtl ──────────────────────────────────────────────────────────

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

    // Write output via malloc (C++ bridge frees it).
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
        // Reclaim on failure.
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

// ── Notifications ────────────────────────────────────────────────────

/// RGBA color for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// Create a new color from RGBA components (0.0–1.0).
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Create a fully opaque color from RGB.
    #[must_use]
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// White.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Red.
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green.
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue.
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

/// Show a notification in Hyprland's notification area.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the notification fails.
pub fn add_notification(
    handle: PluginHandle,
    text: &str,
    color: Color,
    time_ms: f32,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle. text is valid UTF-8.
    let result = unsafe {
        ffi::add_notification(
            handle.0,
            text.as_ptr().cast::<c_char>(),
            text.len(),
            color.r,
            color.g,
            color.b,
            color.a,
            time_ms,
        )
    };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to add notification".into()))
    }
}

/// Show a notification with an icon (v2 API).
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the notification fails.
pub fn add_notification_v2(
    handle: PluginHandle,
    text: &str,
    time_ms: u64,
    color: Color,
    icon: NotificationIcon,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle. text is valid UTF-8.
    let result = unsafe {
        ffi::add_notification_v2(
            handle.0,
            text.as_ptr().cast::<c_char>(),
            text.len(),
            time_ms,
            color.r,
            color.g,
            color.b,
            color.a,
            icon as u8,
        )
    };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to add notification".into()))
    }
}

// ── Config Reload ────────────────────────────────────────────────────

/// Queue an asynchronous config reload.
///
/// # Errors
///
/// Returns [`HyprError::Plugin`] if the reload request fails.
pub fn reload_config() -> HyprResult<()> {
    // SAFETY: This function has no handle parameter — it's process-global.
    let result = unsafe { ffi::reload_config() };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to reload config".into()))
    }
}

// ── Function Hooking (Advanced/Unstable) ─────────────────────────────

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
        // Free the bridge-allocated array.
        unsafe {
            ffi::free_bridge_array(out_addresses as *mut c_void);
        }
    }

    Ok(matches)
}

// ── Version ──────────────────────────────────────────────────────────

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
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    Ok(cstr.to_string_lossy().into_owned())
}
