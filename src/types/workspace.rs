//! Workspace type — virtual desktop representation.
//!
//! Models `CWorkspace` from `src/desktop/Workspace.hpp`.
//! Deserialization target for the `workspaces` and `activeworkspace` IPC queries.

use serde::Deserialize;

use super::common::{FullscreenMode, MonitorId, WindowAddress, WorkspaceId};

/// A Hyprland workspace.
///
/// Combines fields from IPC JSON responses and the internal `CWorkspace` struct.
/// Fields only available via the plugin API are `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    // -- IPC JSON fields (from HyprCtl getWorkspaceData) ---------------------

    /// Workspace ID. Positive = regular, negative = special/name-based.
    pub id: WorkspaceId,

    /// Workspace name.
    pub name: String,

    /// Name of the monitor this workspace is on.
    pub monitor: String,

    /// Numeric ID of the monitor, or `None` if unassigned.
    #[serde(rename = "monitorID")]
    pub monitor_id: Option<MonitorId>,

    /// Number of windows on this workspace.
    pub windows: i32,

    /// Whether any window on this workspace is fullscreen.
    #[serde(rename = "hasfullscreen")]
    pub has_fullscreen: bool,

    /// Address of the last focused window on this workspace.
    #[serde(rename = "lastwindow")]
    pub last_window: WindowAddress,

    /// Title of the last focused window.
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,

    /// Whether this workspace is persistent (survives having no windows).
    #[serde(rename = "ispersistent")]
    pub is_persistent: bool,

    // -- Fields from CWorkspace (plugin API) ---------------------------------

    /// Current fullscreen mode of the workspace.
    #[serde(default)]
    pub fullscreen_mode: FullscreenMode,

    /// Whether this is a special (scratchpad) workspace.
    #[serde(default)]
    pub is_special: bool,

    /// Whether new windows on this workspace default to floating.
    #[serde(default)]
    pub default_floating: bool,

    /// Whether new windows on this workspace default to pseudo-tiled.
    #[serde(default)]
    pub default_pseudo: bool,

    /// Whether the workspace is currently visible on its monitor.
    #[serde(default)]
    pub visible: bool,

    /// Last monitor name (used for reconnection).
    #[serde(default)]
    pub last_monitor: String,
}
