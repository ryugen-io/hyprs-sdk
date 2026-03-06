//! Tiled layout algorithm registration.
//!
//! v0.54 split the monolithic `IHyprLayout` into tiled and floating halves.
//! This module covers the tiled side — plugins that need to arrange non-floating
//! windows (e.g. dwindle, master) register here via `ITiledAlgorithm`.

use std::ffi::c_void;
use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::types::PluginHandle;

use super::common::{Direction, FocalPoint, ModeAlgorithm, RectCorner};

/// Trait for tiled layout algorithms.
///
/// Extends [`ModeAlgorithm`] with tiled-specific operations.
/// Maps to `ITiledAlgorithm` in C++ (v0.54+).
pub trait TiledAlgorithm: ModeAlgorithm {
    /// Get the next candidate target for focus traversal.
    /// Return a target pointer, or null if none.
    fn get_next_candidate(&mut self, _old: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }
}

/// Factory trait for creating tiled algorithm instances.
///
/// Hyprland calls the factory each time a workspace needs a new algorithm
/// instance. The factory must be `Send + Sync + 'static`.
pub trait TiledAlgorithmFactory: Send + Sync + 'static {
    /// The algorithm type this factory creates.
    type Algo: TiledAlgorithm;

    /// Create a new algorithm instance.
    fn create(&self) -> Self::Algo;
}

// Vtable layout must match the C++ struct in bridge_layout_tiled.cpp exactly,
// or the bridge will call wrong function pointers and corrupt memory.

#[repr(C)]
struct ModeAlgoVtable {
    new_target: unsafe extern "C" fn(*mut c_void, *mut c_void),
    moved_target: unsafe extern "C" fn(*mut c_void, *mut c_void, bool, f64, f64),
    remove_target: unsafe extern "C" fn(*mut c_void, *mut c_void),
    resize_target: unsafe extern "C" fn(*mut c_void, f64, f64, *mut c_void, u8),
    recalculate: unsafe extern "C" fn(*mut c_void),
    swap_targets: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    move_target_in_direction: unsafe extern "C" fn(*mut c_void, *mut c_void, i8, bool),
    layout_msg: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        usize,
        *mut *mut c_char,
        *mut usize,
    ) -> bool,
    predict_size: unsafe extern "C" fn(*mut c_void, *mut f64, *mut f64) -> bool,
    drop_fn: unsafe extern "C" fn(*mut c_void),
}

#[repr(C)]
struct TiledAlgoVtable {
    base: ModeAlgoVtable,
    get_next_candidate: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
    factory_fn: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
}

struct AlgoData<T: TiledAlgorithm> {
    inner: T,
}

struct FactoryData<F: TiledAlgorithmFactory> {
    factory: F,
}

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

fn malloc_copy(data: &[u8]) -> *mut c_char {
    if data.is_empty() {
        return std::ptr::null_mut();
    }
    unsafe {
        let ptr = malloc(data.len()).cast::<c_char>();
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len());
        }
        ptr
    }
}

unsafe extern "C" fn tramp_new_target<T: TiledAlgorithm>(ctx: *mut c_void, target: *mut c_void) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    data.inner.new_target(target);
}

unsafe extern "C" fn tramp_moved_target<T: TiledAlgorithm>(
    ctx: *mut c_void,
    target: *mut c_void,
    has_focal: bool,
    fx: f64,
    fy: f64,
) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    let focal = if has_focal {
        Some(FocalPoint { x: fx, y: fy })
    } else {
        None
    };
    data.inner.moved_target(target, focal);
}

unsafe extern "C" fn tramp_remove_target<T: TiledAlgorithm>(ctx: *mut c_void, target: *mut c_void) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    data.inner.remove_target(target);
}

unsafe extern "C" fn tramp_resize_target<T: TiledAlgorithm>(
    ctx: *mut c_void,
    dx: f64,
    dy: f64,
    target: *mut c_void,
    corner: u8,
) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    let c = match corner {
        1 => RectCorner::TopLeft,
        2 => RectCorner::TopRight,
        4 => RectCorner::BottomRight,
        8 => RectCorner::BottomLeft,
        _ => RectCorner::None,
    };
    data.inner.resize_target(dx, dy, target, c);
}

unsafe extern "C" fn tramp_recalculate<T: TiledAlgorithm>(ctx: *mut c_void) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    data.inner.recalculate();
}

unsafe extern "C" fn tramp_swap_targets<T: TiledAlgorithm>(
    ctx: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    data.inner.swap_targets(a, b);
}

unsafe extern "C" fn tramp_move_dir<T: TiledAlgorithm>(
    ctx: *mut c_void,
    target: *mut c_void,
    dir: i8,
    silent: bool,
) {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    let d = match dir {
        0 => Direction::Up,
        1 => Direction::Right,
        2 => Direction::Down,
        3 => Direction::Left,
        _ => Direction::Default,
    };
    data.inner.move_target_in_direction(target, d, silent);
}

unsafe extern "C" fn tramp_layout_msg<T: TiledAlgorithm>(
    ctx: *mut c_void,
    msg_ptr: *const c_char,
    msg_len: usize,
    out_ptr: *mut *mut c_char,
    out_len: *mut usize,
) -> bool {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    let msg = if msg_len > 0 && !msg_ptr.is_null() {
        let bytes = unsafe { std::slice::from_raw_parts(msg_ptr.cast::<u8>(), msg_len) };
        String::from_utf8_lossy(bytes)
    } else {
        std::borrow::Cow::Borrowed("")
    };

    match data.inner.layout_msg(msg.as_ref()) {
        Ok(()) => {
            unsafe {
                *out_ptr = std::ptr::null_mut();
                *out_len = 0;
            }
            true
        }
        Err(err) => {
            let buf = malloc_copy(err.as_bytes());
            unsafe {
                *out_ptr = buf;
                *out_len = err.len();
            }
            false
        }
    }
}

unsafe extern "C" fn tramp_predict_size<T: TiledAlgorithm>(
    ctx: *mut c_void,
    out_x: *mut f64,
    out_y: *mut f64,
) -> bool {
    let data = unsafe { &*(ctx as *const AlgoData<T>) };
    match data.inner.predict_size_for_new_target() {
        Some((x, y)) => {
            unsafe {
                *out_x = x;
                *out_y = y;
            }
            true
        }
        None => false,
    }
}

unsafe extern "C" fn tramp_drop<T: TiledAlgorithm>(ctx: *mut c_void) {
    if !ctx.is_null() {
        unsafe {
            drop(Box::from_raw(ctx as *mut AlgoData<T>));
        }
    }
}

unsafe extern "C" fn tramp_get_next_candidate<T: TiledAlgorithm>(
    ctx: *mut c_void,
    old: *mut c_void,
) -> *mut c_void {
    let data = unsafe { &mut *(ctx as *mut AlgoData<T>) };
    data.inner.get_next_candidate(old)
}

unsafe extern "C" fn tramp_factory<F: TiledAlgorithmFactory>(
    factory_data: *mut c_void,
) -> *mut c_void {
    let fdata = unsafe { &*(factory_data as *const FactoryData<F>) };
    let algo = fdata.factory.create();
    let boxed = Box::new(AlgoData { inner: algo });
    Box::into_raw(boxed).cast()
}

fn build_tiled_vtable<F: TiledAlgorithmFactory>() -> TiledAlgoVtable {
    TiledAlgoVtable {
        base: ModeAlgoVtable {
            new_target: tramp_new_target::<F::Algo>,
            moved_target: tramp_moved_target::<F::Algo>,
            remove_target: tramp_remove_target::<F::Algo>,
            resize_target: tramp_resize_target::<F::Algo>,
            recalculate: tramp_recalculate::<F::Algo>,
            swap_targets: tramp_swap_targets::<F::Algo>,
            move_target_in_direction: tramp_move_dir::<F::Algo>,
            layout_msg: tramp_layout_msg::<F::Algo>,
            predict_size: tramp_predict_size::<F::Algo>,
            drop_fn: tramp_drop::<F::Algo>,
        },
        get_next_candidate: tramp_get_next_candidate::<F::Algo>,
        factory_fn: tramp_factory::<F>,
    }
}

unsafe extern "C" {
    #[link_name = "hyprland_api_add_tiled_algo"]
    fn ffi_add_tiled_algo(
        handle: *mut c_void,
        name_ptr: *const c_char,
        name_len: usize,
        factory_data: *mut c_void,
        vtable: *const TiledAlgoVtable,
    ) -> bool;
}

/// Register a tiled layout algorithm factory.
///
/// Hyprland will call the factory to create instances as needed.
/// Use [`super::remove_algo`] to unregister by name.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if Hyprland rejects the registration.
pub fn register_tiled_algo<F: TiledAlgorithmFactory>(
    handle: PluginHandle,
    name: &str,
    factory: F,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    let fdata = Box::new(FactoryData { factory });
    let fdata_ptr = Box::into_raw(fdata).cast::<c_void>();
    let vtable = build_tiled_vtable::<F>();

    let ok = unsafe {
        ffi_add_tiled_algo(
            handle.0,
            name.as_ptr().cast(),
            name.len(),
            fdata_ptr,
            &vtable,
        )
    };

    if ok {
        Ok(())
    } else {
        // Factory data ownership was transferred to C++ on success.
        // On failure, we need to reclaim it.
        unsafe {
            drop(Box::from_raw(fdata_ptr as *mut FactoryData<F>));
        }
        Err(HyprError::Plugin(format!(
            "failed to register tiled algo: {name}"
        )))
    }
}
