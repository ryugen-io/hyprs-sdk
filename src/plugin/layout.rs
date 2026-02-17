//! Safe wrappers for custom layout registration.
//!
//! Layouts control how tiled windows are arranged on screen. Plugins
//! can register custom layouts that users activate via config.
//!
//! # Architecture
//!
//! The [`Layout`] trait maps to the C++ `IHyprLayout` interface.
//! Implementors define window tiling logic; the SDK handles the FFI
//! boundary via a vtable bridge in the C++ shim.

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::PluginHandle;

/// Opaque handle to a registered layout.
///
/// Used for unregistration. Valid for the plugin's lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct LayoutHandle(pub *mut c_void);

// SAFETY: LayoutHandle is an opaque pointer managed by Hyprland.
unsafe impl Send for LayoutHandle {}
unsafe impl Sync for LayoutHandle {}

impl LayoutHandle {
    /// Null (invalid) handle.
    pub const NULL: Self = Self(std::ptr::null_mut());

    /// Whether this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Direction for window creation placement.
///
/// Maps to `eDirection` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum Direction {
    /// Let the layout decide.
    #[default]
    Default = -1,
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

/// Corner of a rectangle.
///
/// Maps to `eRectCorner` in C++ (bitflags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum RectCorner {
    #[default]
    None = 0,
    TopLeft = 1,
    TopRight = 2,
    BottomRight = 4,
    BottomLeft = 8,
}

/// Trait for custom window layouts.
///
/// Maps to the pure virtual methods of `IHyprLayout` in C++. The C++
/// bridge shim creates a C++ `IHyprLayout` subclass that forwards
/// calls to these trait methods.
///
/// # Required Methods
///
/// Implementors must define the core tiling logic. Optional methods
/// have sensible defaults.
pub trait Layout: Send + 'static {
    /// Called when this layout is activated.
    fn on_enable(&mut self);

    /// Called when this layout is deactivated.
    fn on_disable(&mut self);

    /// The layout's display name (e.g. "dwindle", "master").
    fn name(&self) -> &str;

    /// Called when a tiled window is created. Must set the window's
    /// goal position and size for the animation manager.
    fn on_window_created_tiling(&mut self, window: *mut c_void, direction: Direction);

    /// Called when a tiled window is removed.
    fn on_window_removed_tiling(&mut self, window: *mut c_void);

    /// Whether a window is tiled by this layout.
    fn is_window_tiled(&self, window: *mut c_void) -> bool;

    /// Recalculate layout for a monitor (e.g. after reserved area changes).
    fn recalculate_monitor(&mut self, monitor_id: i64);

    /// Recalculate layout for a specific window.
    fn recalculate_window(&mut self, window: *mut c_void);

    /// Resize the active (or specified) window by a delta.
    fn resize_active_window(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        corner: RectCorner,
        window: *mut c_void,
    );

    /// Handle fullscreen request for a window.
    fn fullscreen_request(&mut self, window: *mut c_void, current_mode: i8, requested_mode: i8);

    /// Handle a custom layout message from a dispatcher.
    /// Returns an optional response string.
    fn layout_message(&mut self, window: *mut c_void, message: &str) -> Option<String>;

    /// Swap two windows' positions.
    fn switch_windows(&mut self, a: *mut c_void, b: *mut c_void);

    /// Move a window in a direction.
    fn move_window_to(&mut self, window: *mut c_void, direction: &str, silent: bool);

    /// Alter the split ratio of a window.
    fn alter_split_ratio(&mut self, window: *mut c_void, ratio: f32, exact: bool);

    /// Replace layout data when a window is replaced.
    fn replace_window_data(&mut self, from: *mut c_void, to: *mut c_void);

    /// Predict the size for a new tiled window. Return `(0, 0)` if unknown.
    fn predict_size_for_new_window_tiled(&self) -> (f64, f64);
}

// ── Internal: fat-pointer wrapper ────────────────────────────────────
//
// `Box<dyn Layout>` is a fat pointer (2 words) that can't round-trip
// through a single `*mut c_void`. We box it once more to get a thin
// `*mut LayoutData` that safely passes through C.

struct LayoutData {
    inner: Box<dyn Layout>,
}

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

/// Allocate a copy of `data` with malloc (C++ bridge frees it).
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

// ── Trampolines ─────────────────────────────────────────────────────
//
// Each function is called by the C++ RustLayoutBridge via the LayoutVtable.
// `rust_layout` is always `*mut LayoutData` created in `register_layout`.

unsafe extern "C" fn trampoline_on_enable(rust_layout: *mut c_void) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.on_enable();
}

unsafe extern "C" fn trampoline_on_disable(rust_layout: *mut c_void) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.on_disable();
}

unsafe extern "C" fn trampoline_get_layout_name(
    rust_layout: *mut c_void,
    out_ptr: *mut *const c_char,
    out_len: *mut usize,
) {
    let data = unsafe { &*(rust_layout as *const LayoutData) };
    let name = data.inner.name();
    unsafe {
        *out_ptr = name.as_ptr().cast();
        *out_len = name.len();
    }
}

unsafe extern "C" fn trampoline_on_window_created_tiling(
    rust_layout: *mut c_void,
    window: *mut c_void,
    direction: i8,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    let dir = match direction {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        3 => Direction::Left,
        _ => Direction::Default,
    };
    data.inner.on_window_created_tiling(window, dir);
}

unsafe extern "C" fn trampoline_on_window_removed_tiling(
    rust_layout: *mut c_void,
    window: *mut c_void,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.on_window_removed_tiling(window);
}

unsafe extern "C" fn trampoline_is_window_tiled(
    rust_layout: *mut c_void,
    window: *mut c_void,
) -> bool {
    let data = unsafe { &*(rust_layout as *const LayoutData) };
    data.inner.is_window_tiled(window)
}

unsafe extern "C" fn trampoline_recalculate_monitor(rust_layout: *mut c_void, monitor_id: i64) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.recalculate_monitor(monitor_id);
}

unsafe extern "C" fn trampoline_recalculate_window(rust_layout: *mut c_void, window: *mut c_void) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.recalculate_window(window);
}

unsafe extern "C" fn trampoline_resize_active_window(
    rust_layout: *mut c_void,
    dx: f64,
    dy: f64,
    corner: u8,
    window: *mut c_void,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    let c = match corner {
        1 => RectCorner::TopLeft,
        2 => RectCorner::TopRight,
        4 => RectCorner::BottomRight,
        8 => RectCorner::BottomLeft,
        _ => RectCorner::None,
    };
    data.inner.resize_active_window(dx, dy, c, window);
}

unsafe extern "C" fn trampoline_fullscreen_request(
    rust_layout: *mut c_void,
    window: *mut c_void,
    current_mode: i8,
    effective_mode: i8,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner
        .fullscreen_request(window, current_mode, effective_mode);
}

unsafe extern "C" fn trampoline_layout_message(
    rust_layout: *mut c_void,
    window: *mut c_void,
    msg_ptr: *const c_char,
    msg_len: usize,
    out_ptr: *mut *mut c_char,
    out_len: *mut usize,
) -> bool {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    let msg = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg_ptr.cast(), msg_len))
    };
    match data.inner.layout_message(window, msg) {
        Some(response) if !response.is_empty() => {
            let buf = malloc_copy(response.as_bytes());
            unsafe {
                *out_ptr = buf;
                *out_len = response.len();
            }
            true
        }
        _ => {
            unsafe {
                *out_ptr = std::ptr::null_mut();
                *out_len = 0;
            }
            false
        }
    }
}

unsafe extern "C" fn trampoline_switch_windows(
    rust_layout: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.switch_windows(a, b);
}

unsafe extern "C" fn trampoline_move_window_to(
    rust_layout: *mut c_void,
    window: *mut c_void,
    dir_ptr: *const c_char,
    dir_len: usize,
    silent: bool,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    let dir = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(dir_ptr.cast(), dir_len))
    };
    data.inner.move_window_to(window, dir, silent);
}

unsafe extern "C" fn trampoline_alter_split_ratio(
    rust_layout: *mut c_void,
    window: *mut c_void,
    ratio: f32,
    exact: bool,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.alter_split_ratio(window, ratio, exact);
}

unsafe extern "C" fn trampoline_replace_window_data(
    rust_layout: *mut c_void,
    from: *mut c_void,
    to: *mut c_void,
) {
    let data = unsafe { &mut *(rust_layout as *mut LayoutData) };
    data.inner.replace_window_data(from, to);
}

unsafe extern "C" fn trampoline_predict_size(
    rust_layout: *mut c_void,
    out_x: *mut f64,
    out_y: *mut f64,
) {
    let data = unsafe { &*(rust_layout as *const LayoutData) };
    let (x, y) = data.inner.predict_size_for_new_window_tiled();
    unsafe {
        *out_x = x;
        *out_y = y;
    }
}

unsafe extern "C" fn trampoline_drop(rust_layout: *mut c_void) {
    if !rust_layout.is_null() {
        // SAFETY: Reclaiming the LayoutData we leaked in register_layout.
        unsafe {
            drop(Box::from_raw(rust_layout as *mut LayoutData));
        }
    }
}

/// Build the vtable passed to the C++ bridge.
fn build_layout_vtable() -> ffi::LayoutVtable {
    ffi::LayoutVtable {
        on_enable: trampoline_on_enable,
        on_disable: trampoline_on_disable,
        get_layout_name: trampoline_get_layout_name,
        on_window_created_tiling: trampoline_on_window_created_tiling,
        on_window_removed_tiling: trampoline_on_window_removed_tiling,
        is_window_tiled: trampoline_is_window_tiled,
        recalculate_monitor: trampoline_recalculate_monitor,
        recalculate_window: trampoline_recalculate_window,
        resize_active_window: trampoline_resize_active_window,
        fullscreen_request: trampoline_fullscreen_request,
        layout_message: trampoline_layout_message,
        switch_windows: trampoline_switch_windows,
        move_window_to: trampoline_move_window_to,
        alter_split_ratio: trampoline_alter_split_ratio,
        replace_window_data: trampoline_replace_window_data,
        predict_size_for_new_window_tiled: trampoline_predict_size,
        drop: trampoline_drop,
    }
}

/// Register a custom window layout.
///
/// The layout becomes available to users via `general:layout = <name>`.
///
/// # Safety
///
/// Calls FFI. Only valid inside a Hyprland plugin process.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub fn register_layout(
    handle: PluginHandle,
    name: &str,
    layout: Box<dyn Layout>,
) -> HyprResult<LayoutHandle> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let data = Box::new(LayoutData { inner: layout });
    let data_ptr = Box::into_raw(data).cast::<c_void>();

    let vtable = build_layout_vtable();

    // SAFETY: We validated the handle. data_ptr is a valid heap allocation.
    // The C++ bridge wraps it in a RustLayoutBridge (IHyprLayout subclass).
    // On failure, the bridge destructor calls vtable.drop to reclaim data_ptr.
    let bridge_ptr = unsafe {
        ffi::add_layout(
            handle.0,
            name.as_ptr().cast::<c_char>(),
            name.len(),
            data_ptr,
            &vtable,
        )
    };

    if bridge_ptr.is_null() {
        // The C++ bridge handled cleanup of data_ptr via vtable.drop.
        Err(HyprError::Plugin(format!(
            "failed to register layout: {name}"
        )))
    } else {
        Ok(LayoutHandle(bridge_ptr))
    }
}

/// Unregister a custom layout.
///
/// The C++ bridge destructor drops the Rust `Layout` impl.
///
/// # Safety
///
/// Calls FFI. Only valid inside a Hyprland plugin process.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the layout was not found.
pub fn unregister_layout(handle: PluginHandle, layout: LayoutHandle) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }
    if layout.is_null() {
        return Err(HyprError::Plugin("null layout handle".into()));
    }

    // SAFETY: We validated both handles. The bridge deletes the C++
    // wrapper which calls vtable.drop to reclaim the Rust LayoutData.
    let result = unsafe { ffi::remove_layout(handle.0, layout.0) };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to unregister layout".into()))
    }
}
