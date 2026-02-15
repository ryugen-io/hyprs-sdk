//! Config option types and descriptors.
//!
//! Maps to `eConfigOptionType`, `SConfigOptionDescription`, and related types
//! from `src/config/ConfigManager.hpp`.

use serde::{Deserialize, Serialize};

/// Config option type discriminant.
///
/// Maps to `eConfigOptionType` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConfigOptionType {
    Bool = 0,
    Int = 1,
    Float = 2,
    StringShort = 3,
    StringLong = 4,
    Color = 5,
    Choice = 6,
    Gradient = 7,
    Vector = 8,
}

impl ConfigOptionType {
    /// Parse from the raw integer used in Hyprland source.
    #[must_use]
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Bool),
            1 => Some(Self::Int),
            2 => Some(Self::Float),
            3 => Some(Self::StringShort),
            4 => Some(Self::StringLong),
            5 => Some(Self::Color),
            6 => Some(Self::Choice),
            7 => Some(Self::Gradient),
            8 => Some(Self::Vector),
            _ => None,
        }
    }
}

/// Config option flags (bitmask).
///
/// Maps to `eConfigOptionFlags` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ConfigOptionFlags(pub u8);

impl ConfigOptionFlags {
    /// The value is a percentage.
    pub const PERCENTAGE: Self = Self(1 << 0);

    #[must_use]
    pub fn is_percentage(self) -> bool {
        self.0 & Self::PERCENTAGE.0 != 0
    }
}

/// Type-specific data for a config option.
///
/// Maps to the `std::variant` inside `SConfigOptionDescription`.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigOptionData {
    Bool {
        value: bool,
    },
    Range {
        value: i32,
        min: i32,
        max: i32,
    },
    Float {
        value: f32,
        min: f32,
        max: f32,
    },
    String {
        value: String,
    },
    Color {
        /// RGBA hex color value.
        rgba: u32,
    },
    Choice {
        first_index: i32,
        /// Comma-separated list of valid choices.
        choices: String,
    },
    Gradient {
        /// Gradient definition string.
        gradient: String,
    },
    Vector {
        x: f64,
        y: f64,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    },
}

/// Description of a configuration option.
///
/// Maps to `SConfigOptionDescription` in Hyprland.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigOptionDescription {
    /// Full option path (e.g. `general:gaps_in`).
    pub value: String,
    /// Human-readable description.
    pub description: String,
    /// Special category (e.g. `device:name` for per-device options).
    pub special_category: String,
    /// Whether the key itself is special.
    pub special_key: bool,
    /// Option type discriminant.
    pub option_type: ConfigOptionType,
    /// Option flags.
    pub flags: ConfigOptionFlags,
    /// Type-specific data and defaults.
    pub data: ConfigOptionData,
}

/// CSS-like gap values (top, right, bottom, left).
///
/// Maps to `CCssGapData` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CssGapData {
    pub top: i64,
    pub right: i64,
    pub bottom: i64,
    pub left: i64,
}

impl CssGapData {
    /// Uniform gap on all sides.
    #[must_use]
    pub fn uniform(value: i64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Vertical and horizontal gaps.
    #[must_use]
    pub fn symmetric(vertical: i64, horizontal: i64) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Gradient value with colors and angle.
///
/// Maps to `CGradientValueData` in Hyprland.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientValue {
    /// Gradient colors as RGBA u32 values.
    pub colors: Vec<u32>,
    /// Angle in radians.
    pub angle: f32,
}

/// Color management type.
///
/// Maps to `NCMType::eCMType` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ColorManagementType {
    /// Auto-detect based on bit depth.
    #[default]
    Auto = 0,
    /// sRGB primaries (default for 8bpc).
    Srgb = 1,
    /// Wide color gamut, BT2020 primaries.
    Wide = 2,
    /// Primaries from EDID (may be inaccurate).
    Edid = 3,
    /// Wide gamut + HDR PQ transfer function.
    Hdr = 4,
    /// HDR with EDID primaries.
    HdrEdid = 5,
    /// DCI-P3 (cinema, greenish white point).
    DciP3 = 6,
    /// Display P3 (Apple, blueish white point).
    DisplayP3 = 7,
    /// Adobe RGB colorspace.
    AdobeRgb = 8,
}
