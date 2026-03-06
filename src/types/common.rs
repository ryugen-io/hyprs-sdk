//! Common newtypes, shared enums, and utility types used across the SDK.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

// Newtypes enforce type safety: a WorkspaceId cannot accidentally be passed where a
// MonitorId is expected, even though both wrap i64. This catches misuse at compile time.

/// Unique address of a Hyprland window (hex pointer value).
///
/// # Examples
///
/// ```
/// use hyprs_sdk::types::common::WindowAddress;
///
/// let addr: WindowAddress = "0x55a3f2c0".parse().unwrap();
/// assert_eq!(addr.to_string(), "0x55a3f2c0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowAddress(pub u64);

impl fmt::Display for WindowAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl FromStr for WindowAddress {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix("0x").unwrap_or(s);
        u64::from_str_radix(hex, 16).map(WindowAddress)
    }
}

impl Serialize for WindowAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WindowAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Workspace identifier. Negative values are special workspaces.
///
/// Regular workspaces have positive IDs. Name-based workspaces start at -1337.
/// Special (scratchpad) workspaces use negative IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct WorkspaceId(pub i64);

impl WorkspaceId {
    /// Invalid workspace sentinel.
    pub const INVALID: Self = Self(-1);

    /// The special (scratchpad) workspace base ID.
    pub const SPECIAL: Self = Self(-99);

    /// Returns true if this is a special workspace (negative ID).
    #[must_use]
    pub fn is_special(self) -> bool {
        self.0 < 0
    }

    /// Returns true if this is the invalid sentinel value.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self != Self::INVALID
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monitor identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MonitorId(pub i64);

impl MonitorId {
    /// Invalid monitor sentinel.
    pub const INVALID: Self = Self(-1);
}

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Compound types like WorkspaceRef appear in Window, Monitor, and other responses.
// Defining them here avoids duplication and keeps deserialization consistent.

/// Lightweight workspace reference (id + name pair).
///
/// Used in Window, Monitor, and other types that reference a workspace
/// without embedding the full workspace data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRef {
    pub id: WorkspaceId,
    pub name: String,
}

// Enums like FullscreenMode and Layer are shared because multiple types reference them
// (e.g. Window.fullscreen and Workspace.fullscreen_mode both use FullscreenMode).

/// Fullscreen mode flags (bitmask).
///
/// Maps to `eFullscreenMode` in Hyprland source. Values can be combined:
/// `MAXIMIZED | FULLSCREEN` means both maximized and fullscreen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum FullscreenMode {
    #[default]
    None = 0,
    Maximized = 1,
    Fullscreen = 2,
    /// Both maximized and fullscreen.
    MaximizedFullscreen = 3,
}

impl FullscreenMode {
    /// Parse from the integer value used in IPC JSON and C++ source.
    #[must_use]
    pub fn from_raw(value: i8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Maximized,
            2 => Self::Fullscreen,
            3 => Self::MaximizedFullscreen,
            _ => Self::None,
        }
    }

    #[must_use]
    pub fn is_fullscreen(self) -> bool {
        matches!(self, Self::Fullscreen | Self::MaximizedFullscreen)
    }

    #[must_use]
    pub fn is_maximized(self) -> bool {
        matches!(self, Self::Maximized | Self::MaximizedFullscreen)
    }
}

impl Serialize for FullscreenMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8(*self as i8)
    }
}

impl<'de> Deserialize<'de> for FullscreenMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = i8::deserialize(deserializer)?;
        Ok(Self::from_raw(v))
    }
}

/// Wayland output transform.
///
/// Maps to `wl_output_transform` from the Wayland protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(i32)]
pub enum OutputTransform {
    #[default]
    Normal = 0,
    Rotated90 = 1,
    Rotated180 = 2,
    Rotated270 = 3,
    Flipped = 4,
    Flipped90 = 5,
    Flipped180 = 6,
    Flipped270 = 7,
}

/// Layer shell layer levels.
///
/// Maps to `zwlr_layer_shell_v1_layer` from the wlr-layer-shell protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Layer {
    Background = 0,
    Bottom = 1,
    Top = 2,
    Overlay = 3,
}

impl Layer {
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
}

/// Content type hint for a window surface.
///
/// Maps to `NContentType::eContentType` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContentType {
    #[default]
    None,
    Photo,
    Video,
    Game,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Photo => write!(f, "photo"),
            Self::Video => write!(f, "video"),
            Self::Game => write!(f, "game"),
        }
    }
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "photo" => Ok(Self::Photo),
            "video" => Ok(Self::Video),
            "game" => Ok(Self::Game),
            _ => Ok(Self::None),
        }
    }
}

impl Serialize for ContentType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ContentType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(s.parse().unwrap_or_default())
    }
}
