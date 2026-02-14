//! Layer surface types — panels, overlays, backgrounds.
//!
//! Models `CLayerSurface` from `src/desktop/view/LayerSurface.hpp`.
//! Deserialization target for the `layers` IPC query.

use std::collections::HashMap;

use serde::Deserialize;

use super::common::WindowAddress;

/// A single layer surface (panel, bar, overlay, etc.).
///
/// Combines fields from IPC JSON and the internal `CLayerSurface` struct.
#[derive(Debug, Clone, Deserialize)]
pub struct LayerSurface {
    // -- IPC JSON fields -----------------------------------------------------
    /// Unique address of this layer surface.
    pub address: WindowAddress,

    /// X position on the monitor.
    pub x: i32,

    /// Y position on the monitor.
    pub y: i32,

    /// Width in pixels.
    pub w: i32,

    /// Height in pixels.
    pub h: i32,

    /// Layer namespace (identifies the application, e.g. "waybar").
    pub namespace: String,

    /// Process ID of the owning application.
    pub pid: i32,

    // -- Fields from CLayerSurface (plugin API) ------------------------------
    /// Layer level (0=background, 1=bottom, 2=top, 3=overlay).
    #[serde(default)]
    pub layer: u32,

    /// Whether the surface is currently mapped.
    #[serde(default)]
    pub mapped: bool,
}

/// Layer surfaces grouped by level for a single monitor.
///
/// Levels are string keys "0" through "3" mapping to layer enum values:
/// - "0" = Background
/// - "1" = Bottom
/// - "2" = Top
/// - "3" = Overlay
#[derive(Debug, Clone, Deserialize)]
pub struct MonitorLayers {
    /// Map from level ("0"-"3") to layer surfaces at that level.
    pub levels: HashMap<String, Vec<LayerSurface>>,
}

/// Full response from the `layers` IPC query.
///
/// Outer map: monitor name → layer data.
#[derive(Debug, Clone, Deserialize)]
pub struct LayersResponse(pub HashMap<String, MonitorLayers>);
