//! hyprland-pointer-warp: warp the cursor to a specific position.

/// A cursor warp target position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarpTarget {
    /// X position in global compositor coordinates.
    pub x: f64,
    /// Y position in global compositor coordinates.
    pub y: f64,
}

impl WarpTarget {
    /// Create a new warp target.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
