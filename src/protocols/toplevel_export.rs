//! hyprland-toplevel-export: capture individual window content.
//!
//! Like screencopy but for individual windows instead of full outputs.

/// Format info for a toplevel capture frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToplevelFrameFormat {
    /// DRM fourcc format code.
    pub format: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row stride in bytes.
    pub stride: u32,
}

impl ToplevelFrameFormat {
    /// Total buffer size needed in bytes.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.stride as usize * self.height as usize
    }
}

/// Flags for a captured toplevel frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ToplevelFrameFlags(u32);

impl ToplevelFrameFlags {
    /// The frame is vertically inverted (Y axis flipped).
    pub const Y_INVERT: Self = Self(1);

    /// Create an empty set of flags.
    #[must_use]
    pub fn empty() -> Self {
        Self(0)
    }

    /// Returns `true` if no flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
