//! Workspace configuration rule.
//!
//! Maps to `SWorkspaceRule` from `src/config/ConfigManager.hpp`.

use super::CssGapData;
use crate::types::common::WorkspaceId;
use std::collections::HashMap;

/// Workspace configuration rule.
///
/// Most fields are `Option` because rules can be partial — only specified
/// fields override the defaults.
/// Maps to `SWorkspaceRule` in Hyprland.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspaceRule {
    /// Target monitor name.
    pub monitor: String,
    /// Raw workspace string from config.
    pub workspace_string: String,
    /// Workspace name.
    pub workspace_name: String,
    /// Workspace numeric ID.
    pub workspace_id: WorkspaceId,
    /// Whether this is the default workspace for the monitor.
    pub is_default: bool,
    /// Whether the workspace persists even when empty.
    pub is_persistent: bool,
    /// Inner gaps (CSS-style: top, right, bottom, left).
    pub gaps_in: Option<CssGapData>,
    /// Outer gaps.
    pub gaps_out: Option<CssGapData>,
    /// Float window gaps (defaults to gaps_out).
    pub float_gaps: Option<CssGapData>,
    /// Border size override.
    pub border_size: Option<i64>,
    /// Whether to show window decorations.
    pub decorate: Option<bool>,
    /// Disable rounding on this workspace.
    pub no_rounding: Option<bool>,
    /// Disable borders on this workspace.
    pub no_border: Option<bool>,
    /// Disable shadows on this workspace.
    pub no_shadow: Option<bool>,
    /// Command to run when this workspace is created empty.
    pub on_created_empty_run_cmd: Option<String>,
    /// Default name for the workspace.
    pub default_name: Option<String>,
    /// Per-layout options.
    pub layout_opts: HashMap<String, String>,
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self(-1)
    }
}
