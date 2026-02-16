//! Public types for screencopy.

use wayland_client::WEnum;
use wayland_client::protocol::wl_shm;

/// Pixel format for captured frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PixelFormat {
    /// 32-bit ARGB with alpha channel.
    Argb8888 = 0,
    /// 32-bit XRGB without alpha channel (alpha ignored).
    Xrgb8888 = 1,
}

impl PixelFormat {
    /// Convert a raw protocol value to a `PixelFormat`.
    ///
    /// Returns `None` for unrecognized values.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Argb8888),
            1 => Some(Self::Xrgb8888),
            _ => None,
        }
    }

    /// Returns the number of bytes per pixel for this format.
    #[must_use]
    pub fn bytes_per_pixel(self) -> u32 {
        4
    }

    pub(super) fn from_wl_format(format: WEnum<wl_shm::Format>) -> Option<Self> {
        match format {
            WEnum::Value(wl_shm::Format::Argb8888) => Some(Self::Argb8888),
            WEnum::Value(wl_shm::Format::Xrgb8888) => Some(Self::Xrgb8888),
            _ => None,
        }
    }

    pub(super) fn to_wl_format(self) -> wl_shm::Format {
        match self {
            Self::Argb8888 => wl_shm::Format::Argb8888,
            Self::Xrgb8888 => wl_shm::Format::Xrgb8888,
        }
    }
}

/// Describes the format and dimensions of a captured frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameFormat {
    /// Pixel format of the frame.
    pub pixel_format: PixelFormat,
    /// Width of the frame in pixels.
    pub width: u32,
    /// Height of the frame in pixels.
    pub height: u32,
    /// Number of bytes per row.
    pub stride: u32,
}

impl FrameFormat {
    /// Calculate the total buffer size in bytes needed for this frame.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.stride as usize * self.height as usize
    }
}

/// A rectangular region for partial screen capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    /// X offset of the capture region.
    pub x: i32,
    /// Y offset of the capture region.
    pub y: i32,
    /// Width of the capture region.
    pub width: i32,
    /// Height of the capture region.
    pub height: i32,
}

/// Bitflags describing properties of a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameFlags(pub(super) u32);

impl FrameFlags {
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

/// Result of a successful frame capture.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Frame format and dimensions.
    pub format: FrameFormat,
    /// Frame flags (e.g. Y_INVERT).
    pub flags: FrameFlags,
    /// Raw pixel data.
    pub data: Vec<u8>,
}
