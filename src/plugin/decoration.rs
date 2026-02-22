//! Safe wrappers for custom window decoration registration.
//!
//! Window decorations draw visual elements around windows (borders,
//! shadows, group bars). Plugins can register custom decorations.
//!
//! # Architecture
//!
//! The [`WindowDecoration`] trait maps to the C++ `IHyprWindowDecoration`
//! interface. The SDK handles the FFI boundary via a vtable bridge.

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{InputType, PluginHandle};

/// Positioning policy for a decoration.
///
/// Maps to `eDecorationPositioningPolicy` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DecorationPositionPolicy {
    /// Decoration wants absolute positioning.
    #[default]
    Absolute = 0,
    /// Decoration is stuck to an edge of the window.
    Sticky = 1,
}

/// Edges that a decoration can attach to (bitflags).
///
/// Maps to `eDecorationEdges` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct DecorationEdges(pub u8);

impl DecorationEdges {
    pub const NONE: Self = Self(0);
    pub const TOP: Self = Self(1 << 0);
    pub const BOTTOM: Self = Self(1 << 1);
    pub const LEFT: Self = Self(1 << 2);
    pub const RIGHT: Self = Self(1 << 3);
    pub const ALL: Self = Self(0b1111);

    /// Whether this edge set contains a specific edge.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for DecorationEdges {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for DecorationEdges {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// Type of decoration.
///
/// Maps to `eDecorationType` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum DecorationType {
    #[default]
    None = -1,
    GroupBar = 0,
    Shadow = 1,
    Border = 2,
    Custom = 3,
}

/// Layer that a decoration renders on.
///
/// Maps to `eDecorationLayer` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DecorationLayer {
    /// Lowest layer, below everything.
    #[default]
    Bottom = 0,
    /// Under the window, but above Bottom.
    Under = 1,
    /// Above the window, but below its popups.
    Over = 2,
    /// Above everything including popups.
    Overlay = 3,
}

/// Decoration behavior flags (bitflags).
///
/// Maps to `eDecorationFlags` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct DecorationFlags(pub u8);

impl DecorationFlags {
    pub const NONE: Self = Self(0);
    /// This decoration accepts mouse input.
    pub const ALLOWS_MOUSE_INPUT: Self = Self(1 << 0);
    /// This decoration is a seamless part of the main window.
    pub const PART_OF_MAIN_WINDOW: Self = Self(1 << 1);
    /// This decoration is not solid (e.g. shadow).
    pub const NON_SOLID: Self = Self(1 << 2);

    /// Whether this flag set contains a specific flag.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for DecorationFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for DecorationFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// Positioning information for a decoration.
///
/// Maps to `SDecorationPositioningInfo` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DecorationPositioningInfo {
    /// How the decoration should be positioned.
    pub policy: DecorationPositionPolicy,
    /// Which edges the decoration attaches to.
    pub edges: DecorationEdges,
    /// Priority (higher = evaluated first). Default: 10.
    pub priority: u32,
    /// Whether to reserve space in the window geometry.
    pub reserved: bool,
}

/// Opaque handle to a registered decoration.
///
/// Used for unregistration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DecorationHandle(pub *mut c_void);

// SAFETY: DecorationHandle is an opaque pointer managed by Hyprland.
unsafe impl Send for DecorationHandle {}
unsafe impl Sync for DecorationHandle {}

impl DecorationHandle {
    /// Null (invalid) handle.
    pub const NULL: Self = Self(std::ptr::null_mut());

    /// Whether this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Trait for custom window decorations.
///
/// Maps to the virtual methods of `IHyprWindowDecoration` in C++.
/// The C++ bridge shim creates a C++ subclass that forwards calls
/// to these trait methods.
pub trait WindowDecoration: Send + 'static {
    /// Return positioning information for this decoration.
    fn get_positioning_info(&self) -> DecorationPositioningInfo;

    /// Called when the positioner assigns geometry.
    fn on_positioning_reply(&mut self, ephemeral: bool);

    /// Draw the decoration.
    ///
    /// `monitor` is an opaque pointer to the current monitor.
    /// `alpha` is the opacity multiplier.
    fn draw(&mut self, monitor: *mut c_void, alpha: f32);

    /// The type of this decoration.
    fn decoration_type(&self) -> DecorationType;

    /// Called when the parent window is updated.
    fn update_window(&mut self, window: *mut c_void);

    /// Mark the entire decoration as damaged (needs redraw).
    fn damage_entire(&mut self);

    /// Handle input on the decoration. Return `true` to consume the event.
    fn on_input(&mut self, _input_type: InputType, _x: f64, _y: f64) -> bool {
        false
    }

    /// The rendering layer for this decoration.
    fn decoration_layer(&self) -> DecorationLayer {
        DecorationLayer::Bottom
    }

    /// Behavior flags for this decoration.
    fn decoration_flags(&self) -> DecorationFlags {
        DecorationFlags::NONE
    }

    /// Display name for debugging.
    fn display_name(&self) -> &str {
        "custom"
    }
}

// `Box<dyn WindowDecoration>` is a fat pointer (2 words) that can't round-trip through
// a single `*mut c_void`. We box it once more to get a thin `*mut DecorationData` that
// safely passes through the C FFI boundary.

struct DecorationData {
    inner: Box<dyn WindowDecoration>,
}

// Trampolines bridge C++ virtual method calls back into Rust trait methods. The C++
// RustDecorationBridge holds a `rust_deco` pointer (*mut DecorationData) and a vtable;
// each trampoline casts the opaque pointer back to DecorationData and dispatches to the
// inner WindowDecoration trait impl.

unsafe extern "C" fn trampoline_get_positioning_info(
    rust_deco: *mut c_void,
    out_policy: *mut u8,
    out_edges: *mut u8,
    out_priority: *mut u32,
    out_reserved: *mut bool,
) {
    let data = unsafe { &*(rust_deco as *const DecorationData) };
    let info = data.inner.get_positioning_info();
    unsafe {
        *out_policy = info.policy as u8;
        *out_edges = info.edges.0;
        *out_priority = info.priority;
        *out_reserved = info.reserved;
    }
}

unsafe extern "C" fn trampoline_on_positioning_reply(rust_deco: *mut c_void, ephemeral: bool) {
    let data = unsafe { &mut *(rust_deco as *mut DecorationData) };
    data.inner.on_positioning_reply(ephemeral);
}

unsafe extern "C" fn trampoline_draw(rust_deco: *mut c_void, monitor: *mut c_void, alpha: f32) {
    let data = unsafe { &mut *(rust_deco as *mut DecorationData) };
    data.inner.draw(monitor, alpha);
}

unsafe extern "C" fn trampoline_get_decoration_type(rust_deco: *mut c_void) -> i8 {
    let data = unsafe { &*(rust_deco as *const DecorationData) };
    data.inner.decoration_type() as i8
}

unsafe extern "C" fn trampoline_update_window(rust_deco: *mut c_void, window: *mut c_void) {
    let data = unsafe { &mut *(rust_deco as *mut DecorationData) };
    data.inner.update_window(window);
}

unsafe extern "C" fn trampoline_damage_entire(rust_deco: *mut c_void) {
    let data = unsafe { &mut *(rust_deco as *mut DecorationData) };
    data.inner.damage_entire();
}

unsafe extern "C" fn trampoline_on_input(
    rust_deco: *mut c_void,
    input_type: u8,
    x: f64,
    y: f64,
) -> bool {
    let data = unsafe { &mut *(rust_deco as *mut DecorationData) };
    let it = InputType::from_raw(input_type).unwrap_or(InputType::Button);
    data.inner.on_input(it, x, y)
}

unsafe extern "C" fn trampoline_get_decoration_layer(rust_deco: *mut c_void) -> u8 {
    let data = unsafe { &*(rust_deco as *const DecorationData) };
    data.inner.decoration_layer() as u8
}

unsafe extern "C" fn trampoline_get_decoration_flags(rust_deco: *mut c_void) -> u64 {
    let data = unsafe { &*(rust_deco as *const DecorationData) };
    data.inner.decoration_flags().0 as u64
}

unsafe extern "C" fn trampoline_get_display_name(
    rust_deco: *mut c_void,
    out_ptr: *mut *const c_char,
    out_len: *mut usize,
) {
    let data = unsafe { &*(rust_deco as *const DecorationData) };
    let name = data.inner.display_name();
    unsafe {
        *out_ptr = name.as_ptr().cast();
        *out_len = name.len();
    }
}

unsafe extern "C" fn trampoline_drop(rust_deco: *mut c_void) {
    if !rust_deco.is_null() {
        // SAFETY: Reclaiming the DecorationData we leaked in register_decoration.
        unsafe {
            drop(Box::from_raw(rust_deco as *mut DecorationData));
        }
    }
}

/// Build the vtable passed to the C++ bridge.
fn build_decoration_vtable() -> ffi::DecorationVtable {
    ffi::DecorationVtable {
        get_positioning_info: trampoline_get_positioning_info,
        on_positioning_reply: trampoline_on_positioning_reply,
        draw: trampoline_draw,
        get_decoration_type: trampoline_get_decoration_type,
        update_window: trampoline_update_window,
        damage_entire: trampoline_damage_entire,
        on_input: trampoline_on_input,
        get_decoration_layer: trampoline_get_decoration_layer,
        get_decoration_flags: trampoline_get_decoration_flags,
        get_display_name: trampoline_get_display_name,
        drop: trampoline_drop,
    }
}

/// Register a custom window decoration on a specific window.
///
/// `window_handle` is a heap-allocated `PHLWINDOW*` obtained via
/// [`clone_window_handle`](ffi::clone_window_handle) from hook callback
/// data. The C++ bridge consumes this handle.
///
/// # Safety
///
/// Calls FFI. Only valid inside a Hyprland plugin process.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub unsafe fn register_decoration(
    handle: PluginHandle,
    window_handle: *mut c_void,
    decoration: Box<dyn WindowDecoration>,
) -> HyprResult<DecorationHandle> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }
    if window_handle.is_null() {
        return Err(HyprError::Plugin("null window handle".into()));
    }

    let data = Box::new(DecorationData { inner: decoration });
    let data_ptr = Box::into_raw(data).cast::<c_void>();

    let vtable = build_decoration_vtable();

    // SAFETY: We validated the handle. data_ptr is a valid heap allocation.
    // The C++ bridge wraps it in a RustDecorationBridge (IHyprWindowDecoration subclass).
    // On failure, the bridge destructor calls vtable.drop to reclaim data_ptr.
    let bridge_ptr =
        unsafe { ffi::add_window_decoration(handle.0, window_handle, data_ptr, &vtable) };

    if bridge_ptr.is_null() {
        // No Rust-side cleanup needed: the C++ bridge destructor already called vtable.drop
        // to reclaim data_ptr, so we avoid a double-free.
        Err(HyprError::Plugin("failed to register decoration".into()))
    } else {
        Ok(DecorationHandle(bridge_ptr))
    }
}

/// Unregister a custom window decoration.
///
/// The C++ bridge destructor drops the Rust `WindowDecoration` impl.
///
/// # Safety
///
/// Calls FFI. Only valid inside a Hyprland plugin process.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the decoration was not found.
pub fn unregister_decoration(handle: PluginHandle, decoration: DecorationHandle) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }
    if decoration.is_null() {
        return Err(HyprError::Plugin("null decoration handle".into()));
    }

    // SAFETY: We validated both handles. The bridge deletes the C++
    // wrapper which calls vtable.drop to reclaim the Rust DecorationData.
    let result = unsafe { ffi::remove_window_decoration(handle.0, decoration.0) };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to unregister decoration".into()))
    }
}

/// RAII wrapper for a window handle obtained from hook callback data.
///
/// Extracted via [`ffi::clone_window_handle`] and freed on drop via
/// [`ffi::release_window_handle`]. Use [`into_raw`](Self::into_raw)
/// to transfer ownership to [`register_decoration`].
pub struct WindowHandle(*mut c_void);

// SAFETY: WindowHandle wraps a heap-allocated PHLWINDOW managed by the bridge.
unsafe impl Send for WindowHandle {}

impl WindowHandle {
    /// Extract a window handle from hook callback event data.
    ///
    /// `any_ptr` must point to a `std::any*` containing a `PHLWINDOW`.
    /// Returns `None` if extraction fails.
    ///
    /// # Safety
    ///
    /// `any_ptr` must be a valid pointer from a hook callback's event data.
    pub unsafe fn from_hook_data(any_ptr: *mut c_void) -> Option<Self> {
        if any_ptr.is_null() {
            return None;
        }
        let ptr = unsafe { ffi::clone_window_handle(any_ptr) };
        if ptr.is_null() { None } else { Some(Self(ptr)) }
    }

    /// Consume the handle, returning the raw pointer.
    ///
    /// After calling this, the caller is responsible for the pointer
    /// (typically by passing it to [`register_decoration`]).
    pub fn into_raw(self) -> *mut c_void {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Get the raw pointer without consuming the handle.
    #[must_use]
    pub fn as_raw(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for WindowHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: We obtained this via clone_window_handle.
            unsafe {
                ffi::release_window_handle(self.0);
            }
        }
    }
}

impl std::fmt::Debug for WindowHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WindowHandle").field(&self.0).finish()
    }
}
