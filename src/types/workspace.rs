//! Workspace type — virtual desktop representation.
//!
//! Deserialization target for the `workspaces` and `activeworkspace` IPC queries.

use serde::Deserialize;

use super::common::{MonitorId, WindowAddress, WorkspaceId};

/// A Hyprland workspace, as returned by the `workspaces` IPC query.
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    /// Workspace ID. Negative values indicate special workspaces.
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
}
