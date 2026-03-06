//! Shared types for the layout algorithm system.
//!
//! Both tiled and floating algorithms share these base types and the
//! `ModeAlgorithm` trait — factored here to avoid duplication and to
//! mirror the C++ `IModeAlgorithm` inheritance hierarchy.

use std::ffi::c_void;

/// Direction for target movement.
///
/// Maps to `Math::eDirection` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum Direction {
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

/// Optional focal point for target placement.
#[derive(Debug, Clone, Copy)]
pub struct FocalPoint {
    pub x: f64,
    pub y: f64,
}

/// Base trait for layout algorithms (shared by tiled and floating).
///
/// Maps to the pure virtual methods of `IModeAlgorithm` in C++.
/// Both [`super::tiled::TiledAlgorithm`] and [`super::floating::FloatingAlgorithm`]
/// extend this trait with their specific methods.
pub trait ModeAlgorithm: Send + 'static {
    /// A new target (window) was added to this algorithm.
    fn new_target(&mut self, target: *mut c_void);

    /// A target was moved into this algorithm from another.
    fn moved_target(&mut self, target: *mut c_void, focal_point: Option<FocalPoint>);

    /// A target was removed.
    fn remove_target(&mut self, target: *mut c_void);

    /// Resize a target by a delta. `corner` indicates interactive resize origin.
    fn resize_target(&mut self, dx: f64, dy: f64, target: *mut c_void, corner: RectCorner);

    /// Recalculate the entire layout.
    fn recalculate(&mut self);

    /// Swap two targets' positions.
    fn swap_targets(&mut self, a: *mut c_void, b: *mut c_void);

    /// Move a target in a direction.
    fn move_target_in_direction(&mut self, target: *mut c_void, dir: Direction, silent: bool);

    /// Handle a layout message. Returns `Ok(())` on success or `Err(message)` on failure.
    fn layout_msg(&mut self, _msg: &str) -> Result<(), String> {
        Err("not implemented".into())
    }

    /// Predict the size for a new target. Return `None` if unknown.
    fn predict_size_for_new_target(&self) -> Option<(f64, f64)> {
        None
    }
}
