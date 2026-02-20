use std::ops::BitOr;

use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

/// The layer a surface should be placed on.
///
/// Layers are rendered in order from background to overlay, with each
/// layer stacking above the previous one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShellLayer {
    /// Behind all other surfaces (e.g., desktop wallpaper).
    Background = 0,
    /// Below normal windows (e.g., desktop widgets).
    Bottom = 1,
    /// Above normal windows (e.g., taskbars, panels).
    Top = 2,
    /// Above everything else (e.g., lock screens, notifications).
    Overlay = 3,
}

impl ShellLayer {
    /// Convert a raw protocol value to a `ShellLayer`.
    ///
    /// Returns `None` for unrecognized values.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Background),
            1 => Some(Self::Bottom),
            2 => Some(Self::Top),
            3 => Some(Self::Overlay),
            _ => None,
        }
    }

    pub(super) fn to_protocol(self) -> zwlr_layer_shell_v1::Layer {
        match self {
            Self::Background => zwlr_layer_shell_v1::Layer::Background,
            Self::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
            Self::Top => zwlr_layer_shell_v1::Layer::Top,
            Self::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
        }
    }
}

/// Edge anchoring bitmask for layer surfaces.
///
/// Anchoring a surface to opposite edges (e.g., left and right) causes
/// it to stretch to fill that dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Anchor(u32);

impl Anchor {
    /// Anchor to the top edge.
    pub const TOP: Self = Self(1);
    /// Anchor to the bottom edge.
    pub const BOTTOM: Self = Self(2);
    /// Anchor to the left edge.
    pub const LEFT: Self = Self(4);
    /// Anchor to the right edge.
    pub const RIGHT: Self = Self(8);

    /// Returns `true` if no edges are anchored.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if this anchor describes a horizontal bar
    /// (anchored to left, right, and exactly one of top/bottom).
    #[must_use]
    pub fn is_horizontal_bar(self) -> bool {
        self.contains(Self::LEFT)
            && self.contains(Self::RIGHT)
            && (self.contains(Self::TOP) || self.contains(Self::BOTTOM))
            && !(self.contains(Self::TOP) && self.contains(Self::BOTTOM))
    }

    /// Returns `true` if this anchor describes a vertical bar
    /// (anchored to top, bottom, and exactly one of left/right).
    #[must_use]
    pub fn is_vertical_bar(self) -> bool {
        self.contains(Self::TOP)
            && self.contains(Self::BOTTOM)
            && (self.contains(Self::LEFT) || self.contains(Self::RIGHT))
            && !(self.contains(Self::LEFT) && self.contains(Self::RIGHT))
    }

    pub(super) fn to_protocol(self) -> zwlr_layer_surface_v1::Anchor {
        zwlr_layer_surface_v1::Anchor::from_bits_truncate(self.0)
    }
}

impl BitOr for Anchor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Keyboard interactivity mode for a layer surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum KeyboardInteractivity {
    /// The surface does not receive keyboard focus.
    #[default]
    None = 0,
    /// The surface receives exclusive keyboard focus when mapped.
    Exclusive = 1,
    /// The surface receives keyboard focus on demand (e.g., when clicked).
    OnDemand = 2,
}

impl KeyboardInteractivity {
    pub(super) fn to_protocol(self) -> zwlr_layer_surface_v1::KeyboardInteractivity {
        match self {
            Self::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
            Self::Exclusive => zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
            Self::OnDemand => zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
        }
    }
}

/// Configuration for creating a layer surface.
#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    /// The layer to place the surface on.
    pub layer: ShellLayer,
    /// Application-defined namespace (e.g., "panel", "taskbar").
    pub namespace: String,
    /// Desired width (0 means the compositor decides).
    pub width: u32,
    /// Desired height (0 means the compositor decides).
    pub height: u32,
    /// Edge anchoring.
    pub anchor: Anchor,
    /// Size of the exclusive zone in pixels, or -1 for auto.
    pub exclusive_zone: i32,
    /// Keyboard interactivity mode.
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Top margin in pixels.
    pub margin_top: i32,
    /// Bottom margin in pixels.
    pub margin_bottom: i32,
    /// Left margin in pixels.
    pub margin_left: i32,
    /// Right margin in pixels.
    pub margin_right: i32,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            layer: ShellLayer::Top,
            namespace: String::new(),
            width: 0,
            height: 0,
            anchor: Anchor::default(),
            exclusive_zone: 0,
            keyboard_interactivity: KeyboardInteractivity::None,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
        }
    }
}

/// A created layer surface with its configure state.
#[derive(Debug)]
pub struct LayerSurfaceHandle {
    /// The underlying `wl_surface`. Attach a buffer and commit to display content.
    pub wl_surface: wl_surface::WlSurface,
    /// The layer surface protocol handle.
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    /// Configured width from the compositor (after first configure).
    pub width: u32,
    /// Configured height from the compositor (after first configure).
    pub height: u32,
    /// Whether the surface has been closed by the compositor.
    pub closed: bool,
}
