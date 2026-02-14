//! Monitor type — physical output representation.
//!
//! Models `CMonitor` from `src/helpers/Monitor.hpp`.
//! Deserialization target for the `monitors` IPC query.

use serde::Deserialize;

use super::common::{MonitorId, WindowAddress, WorkspaceRef};

/// A Hyprland monitor (physical output).
///
/// Combines fields from IPC JSON responses and the internal `CMonitor` struct.
/// Fields only available via the plugin API are `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct Monitor {
    // -- Identity ------------------------------------------------------------

    /// Numeric monitor ID.
    pub id: MonitorId,

    /// Monitor name (e.g. "DP-1", "HDMI-A-1").
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// Manufacturer name.
    pub make: String,

    /// Model name.
    pub model: String,

    /// Serial number.
    pub serial: String,

    // -- Resolution & physical size ------------------------------------------

    /// Pixel width.
    pub width: i32,

    /// Pixel height.
    pub height: i32,

    /// Physical width in millimeters.
    #[serde(rename = "physicalWidth")]
    pub physical_width: i32,

    /// Physical height in millimeters.
    #[serde(rename = "physicalHeight")]
    pub physical_height: i32,

    /// Refresh rate in Hz.
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,

    // -- Position ------------------------------------------------------------

    /// X position on the virtual desktop.
    pub x: i32,

    /// Y position on the virtual desktop.
    pub y: i32,

    // -- Workspaces ----------------------------------------------------------

    /// Currently active workspace on this monitor.
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WorkspaceRef,

    /// Active special (scratchpad) workspace, if any.
    #[serde(rename = "specialWorkspace")]
    pub special_workspace: WorkspaceRef,

    // -- Display settings ----------------------------------------------------

    /// Reserved pixel areas `[left, top, right, bottom]` (e.g. for bars).
    pub reserved: [i32; 4],

    /// Display scale factor.
    pub scale: f64,

    /// Output transform (`wl_output_transform` integer value).
    pub transform: i32,

    // -- State flags ---------------------------------------------------------

    /// Whether this monitor is currently focused.
    pub focused: bool,

    /// DPMS power state (`true` = on).
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,

    /// Whether VRR (adaptive sync) is active.
    pub vrr: bool,

    /// Whether the monitor is disabled.
    pub disabled: bool,

    // -- Solitary client -----------------------------------------------------

    /// Address of the solitary (only visible) client, or `0x0`.
    pub solitary: WindowAddress,

    /// Bitmask of reasons solitary mode is blocked.
    #[serde(rename = "solitaryBlockedBy")]
    pub solitary_blocked_by: u32,

    // -- Tearing -------------------------------------------------------------

    /// Whether frame tearing is actively occurring.
    #[serde(rename = "activelyTearing")]
    pub actively_tearing: bool,

    /// Bitmask of reasons tearing is blocked.
    #[serde(rename = "tearingBlockedBy")]
    pub tearing_blocked_by: u8,

    // -- Direct scanout ------------------------------------------------------

    /// Address of the direct-scanout client, or `0x0`.
    #[serde(rename = "directScanoutTo")]
    pub direct_scanout_to: WindowAddress,

    /// Bitmask of reasons direct scanout is blocked.
    #[serde(rename = "directScanoutBlockedBy")]
    pub direct_scanout_blocked_by: u16,

    // -- Format & mirroring --------------------------------------------------

    /// Current DRM pixel format (e.g. "DRM_FORMAT_XRGB8888").
    #[serde(rename = "currentFormat")]
    pub current_format: String,

    /// Name of the monitor being mirrored, or "none".
    #[serde(rename = "mirrorOf")]
    pub mirror_of: String,

    /// Available display modes (e.g. `["2560x1440@144.00Hz"]`).
    #[serde(rename = "availableModes")]
    pub available_modes: Vec<String>,

    // -- Color management ----------------------------------------------------

    /// Color management preset name (e.g. "sRGB").
    #[serde(rename = "colorManagementPreset")]
    pub color_management_preset: String,

    /// SDR brightness multiplier.
    #[serde(rename = "sdrBrightness")]
    pub sdr_brightness: f64,

    /// SDR saturation multiplier.
    #[serde(rename = "sdrSaturation")]
    pub sdr_saturation: f64,

    /// SDR minimum luminance (nits).
    #[serde(rename = "sdrMinLuminance")]
    pub sdr_min_luminance: f64,

    /// SDR maximum luminance (nits).
    #[serde(rename = "sdrMaxLuminance")]
    pub sdr_max_luminance: i32,

    // -- Fields from CMonitor (plugin API) -----------------------------------

    /// Whether 10-bit color is enabled.
    #[serde(default)]
    pub enabled_10bit: bool,

    /// Whether this monitor was created by user configuration.
    #[serde(default)]
    pub created_by_user: bool,
}
