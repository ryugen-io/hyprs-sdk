//! Monitor configuration rule.
//!
//! Maps to `SMonitorRule` from `src/helpers/Monitor.hpp`.

use crate::types::common::OutputTransform;

/// Auto-positioning direction for monitor placement.
///
/// Maps to `eAutoDirs` in Hyprland.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum AutoDirection {
    /// No auto-direction (treated as right).
    #[default]
    None = 0,
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
    CenterUp = 5,
    CenterDown = 6,
    CenterLeft = 7,
    CenterRight = 8,
}

/// Monitor configuration rule.
///
/// Defines how a monitor should be configured (resolution, position, scale, etc.).
/// Maps to `SMonitorRule` in Hyprland.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorRule {
    /// Monitor name or connector (e.g. `DP-1`, `HDMI-A-1`).
    pub name: String,
    /// Auto-positioning direction.
    pub auto_dir: AutoDirection,
    /// Resolution in pixels.
    pub resolution_x: f64,
    pub resolution_y: f64,
    /// Position offset in pixels.
    pub offset_x: f64,
    pub offset_y: f64,
    /// Display scale factor.
    pub scale: f32,
    /// Refresh rate in Hz.
    pub refresh_rate: f32,
    /// Whether the monitor is disabled.
    pub disabled: bool,
    /// Output transform (rotation/flip).
    pub transform: OutputTransform,
    /// Name of monitor to mirror, if any.
    pub mirror_of: String,
    /// Enable 10-bit color output.
    pub enable_10bit: bool,
    /// Color management type.
    pub cm_type: super::ColorManagementType,
    /// SDR EOTF value.
    pub sdr_eotf: i32,
    /// SDR to HDR saturation factor.
    pub sdr_saturation: f32,
    /// SDR to HDR brightness factor.
    pub sdr_brightness: f32,
    /// Wide color support: 0=auto, 1=force enable, -1=force disable.
    pub supports_wide_color: i32,
    /// HDR support: 0=auto, 1=force enable, -1=force disable.
    pub supports_hdr: i32,
    /// SDR minimum luminance for HDR mapping.
    pub sdr_min_luminance: f32,
    /// SDR maximum luminance for HDR mapping.
    pub sdr_max_luminance: i32,
    /// Minimum luminance override (>=0 overrides EDID).
    pub min_luminance: f32,
    /// Maximum luminance override (>=0 overrides EDID).
    pub max_luminance: i32,
    /// Maximum average luminance override (>=0 overrides EDID).
    pub max_avg_luminance: i32,
    /// VRR (variable refresh rate) mode.
    pub vrr: Option<i32>,
}

impl Default for MonitorRule {
    fn default() -> Self {
        Self {
            name: String::new(),
            auto_dir: AutoDirection::None,
            resolution_x: 1280.0,
            resolution_y: 720.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            refresh_rate: 60.0,
            disabled: false,
            transform: OutputTransform::Normal,
            mirror_of: String::new(),
            enable_10bit: false,
            cm_type: super::ColorManagementType::Srgb,
            sdr_eotf: 0,
            sdr_saturation: 1.0,
            sdr_brightness: 1.0,
            supports_wide_color: 0,
            supports_hdr: 0,
            sdr_min_luminance: 0.2,
            sdr_max_luminance: 80,
            min_luminance: -1.0,
            max_luminance: -1,
            max_avg_luminance: -1,
            vrr: None,
        }
    }
}
