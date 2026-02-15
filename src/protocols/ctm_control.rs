//! hyprland-ctm-control: color transform matrix for outputs.
//!
//! Apply a 3x3 color correction matrix to output content.

/// A 3x3 color transform matrix.
///
/// Applied to output pixel colors as: `[R', G', B'] = matrix * [R, G, B]`.
/// Row-major order: `[r0c0, r0c1, r0c2, r1c0, r1c1, r1c2, r2c0, r2c1, r2c2]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorTransformMatrix {
    /// 9 matrix elements in row-major order.
    pub elements: [f64; 9],
}

impl ColorTransformMatrix {
    /// Identity matrix (no color transformation).
    pub const IDENTITY: Self = Self {
        elements: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Create a matrix that scales RGB channels independently.
    #[must_use]
    pub fn scale(r: f64, g: f64, b: f64) -> Self {
        Self {
            elements: [r, 0.0, 0.0, 0.0, g, 0.0, 0.0, 0.0, b],
        }
    }

    /// Create a grayscale conversion matrix using standard luminance weights.
    #[must_use]
    pub fn grayscale() -> Self {
        Self {
            elements: [
                0.2126, 0.7152, 0.0722, 0.2126, 0.7152, 0.0722, 0.2126, 0.7152, 0.0722,
            ],
        }
    }
}

impl Default for ColorTransformMatrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}
