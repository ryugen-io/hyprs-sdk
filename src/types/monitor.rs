//! Monitor type — physical output representation.
//!
//! Models `CMonitor` from `src/helpers/Monitor.hpp`.
//! Deserialization target for the `monitors` IPC query.

use serde::Deserialize;
use serde::de::Deserializer;

use super::common::{MonitorId, WindowAddress, WorkspaceRef};

/// A Hyprland monitor (physical output).
///
/// Combines fields from IPC JSON responses and the internal `CMonitor` struct.
/// Fields only available via the plugin API are `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct Monitor {
    // Identity fields uniquely identify a physical output. The name (e.g. "DP-1") is
    // the DRM connector name and is stable across restarts; make/model/serial come
    // from EDID and are used for persistent per-monitor config matching.
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

    // Resolution is the active mode's pixel dimensions. Physical size in mm comes from
    // EDID and is needed for DPI calculation and fractional scaling decisions.
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

    // Position in the virtual desktop coordinate space. Multi-monitor layouts place
    // monitors side-by-side or stacked; these offsets define the arrangement.
    /// X position on the virtual desktop.
    pub x: i32,

    /// Y position on the virtual desktop.
    pub y: i32,

    // Each monitor has one active workspace and optionally one special (scratchpad)
    // workspace. Tracking both is needed for workspace-aware bar displays and
    // window-to-monitor routing logic.
    /// Currently active workspace on this monitor.
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WorkspaceRef,

    /// Active special (scratchpad) workspace, if any.
    #[serde(rename = "specialWorkspace")]
    pub special_workspace: WorkspaceRef,

    // Display settings affect how content is rendered on this output. Reserved areas
    // are claimed by bars/panels; scale and transform define the output-to-surface
    // coordinate mapping per the Wayland output protocol.
    /// Reserved pixel areas `[left, top, right, bottom]` (e.g. for bars).
    pub reserved: [i32; 4],

    /// Display scale factor.
    pub scale: f64,

    /// Output transform (`wl_output_transform` integer value).
    pub transform: i32,

    // Runtime state flags reflect transient monitor conditions. These change frequently
    // (e.g. focus follows mouse, DPMS toggles on idle) and drive bar/indicator UIs.
    /// Whether this monitor is currently focused.
    pub focused: bool,

    /// DPMS power state (`true` = on).
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,

    /// Whether VRR (adaptive sync) is active.
    pub vrr: bool,

    /// Whether the monitor is disabled.
    pub disabled: bool,

    // Solitary mode is an optimization: when only one window is visible, the compositor
    // can skip certain rendering steps. The blocked-by bitmask tracks why it cannot.
    /// Address of the solitary (only visible) client, or `0x0`.
    pub solitary: WindowAddress,

    /// Bitmask of reasons solitary mode is blocked.
    #[serde(
        rename = "solitaryBlockedBy",
        deserialize_with = "deserialize_blocked_by_u32"
    )]
    pub solitary_blocked_by: u32,

    // Tearing (immediate presentation without vsync) is used for latency-sensitive apps
    // like games. The blocked-by bitmask lets tools explain why tearing is not active.
    /// Whether frame tearing is actively occurring.
    #[serde(rename = "activelyTearing")]
    pub actively_tearing: bool,

    /// Bitmask of reasons tearing is blocked.
    #[serde(
        rename = "tearingBlockedBy",
        deserialize_with = "deserialize_blocked_by_u8"
    )]
    pub tearing_blocked_by: u8,

    // Direct scanout bypasses composition by presenting the client buffer directly on
    // the display plane. Like tearing, the blocked-by bitmask explains blockers.
    /// Address of the direct-scanout client, or `0x0`.
    #[serde(rename = "directScanoutTo")]
    pub direct_scanout_to: WindowAddress,

    /// Bitmask of reasons direct scanout is blocked.
    #[serde(
        rename = "directScanoutBlockedBy",
        deserialize_with = "deserialize_blocked_by_u16"
    )]
    pub direct_scanout_blocked_by: u16,

    // Pixel format and mirroring are DRM/KMS-level settings. The format determines
    // color depth and HDR capability; mirror_of links two outputs for cloning.
    /// Current DRM pixel format (e.g. "DRM_FORMAT_XRGB8888").
    #[serde(rename = "currentFormat")]
    pub current_format: String,

    /// Name of the monitor being mirrored, or "none".
    #[serde(rename = "mirrorOf")]
    pub mirror_of: String,

    /// Available display modes (e.g. `["2560x1440@144.00Hz"]`).
    #[serde(rename = "availableModes")]
    pub available_modes: Vec<String>,

    // Color management fields control HDR/SDR tone mapping. These are needed for
    // correct rendering on HDR displays and for user-facing brightness/saturation controls.
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

    // Plugin API fields come from CMonitor internals and are only populated when
    // accessed through the Hyprland plugin interface, not standard IPC JSON.
    // They default so that IPC deserialization works without these keys present.
    /// Whether 10-bit color is enabled.
    #[serde(default)]
    pub enabled_10bit: bool,

    /// Whether this monitor was created by user configuration.
    #[serde(default)]
    pub created_by_user: bool,
}

fn parse_mask_from_text(text: &str) -> Option<u64> {
    text.parse::<u64>().ok()
}

fn deserialize_blocked_by_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let out = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::String(t) => parse_mask_from_text(&t).unwrap_or(0),
        serde_json::Value::Null | serde_json::Value::Array(_) => 0,
        _ => 0,
    };
    Ok(u32::try_from(out).unwrap_or(0))
}

fn deserialize_blocked_by_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let out = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::String(t) => parse_mask_from_text(&t).unwrap_or(0),
        serde_json::Value::Null | serde_json::Value::Array(_) => 0,
        _ => 0,
    };
    Ok(u16::try_from(out).unwrap_or(0))
}

fn deserialize_blocked_by_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let out = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::String(t) => parse_mask_from_text(&t).unwrap_or(0),
        serde_json::Value::Null | serde_json::Value::Array(_) => 0,
        _ => 0,
    };
    Ok(u8::try_from(out).unwrap_or(0))
}
