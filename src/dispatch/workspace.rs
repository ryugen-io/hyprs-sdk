//! Workspace management dispatchers.

use super::DispatchCmd;

/// Switch to a workspace (ID, name, `previous`, `+1`, `-1`, etc.).
#[must_use]
pub fn switch(target: &str) -> DispatchCmd {
    DispatchCmd {
        name: "workspace",
        args: target.to_string(),
    }
}

/// Rename a workspace.
#[must_use]
pub fn rename(id: i64, new_name: &str) -> DispatchCmd {
    DispatchCmd {
        name: "renameworkspace",
        args: if new_name.is_empty() {
            id.to_string()
        } else {
            format!("{id} {new_name}")
        },
    }
}

/// Toggle a special (scratchpad) workspace.
#[must_use]
pub fn toggle_special(name: &str) -> DispatchCmd {
    DispatchCmd {
        name: "togglespecialworkspace",
        args: name.to_string(),
    }
}

/// Set workspace options (`allpseudo` or `allfloat`).
#[must_use]
pub fn workspace_opt(opt: &str) -> DispatchCmd {
    DispatchCmd {
        name: "workspaceopt",
        args: opt.to_string(),
    }
}

/// Focus a workspace on the current monitor (swap if needed).
#[must_use]
pub fn focus_on_current_monitor(workspace: &str) -> DispatchCmd {
    DispatchCmd {
        name: "focusworkspaceoncurrentmonitor",
        args: workspace.to_string(),
    }
}

/// Move the current workspace to a different monitor.
#[must_use]
pub fn move_current_to_monitor(monitor: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movecurrentworkspacetomonitor",
        args: monitor.to_string(),
    }
}

/// Move a specific workspace to a monitor.
#[must_use]
pub fn move_to_monitor(workspace: &str, monitor: &str) -> DispatchCmd {
    DispatchCmd {
        name: "moveworkspacetomonitor",
        args: format!("{workspace} {monitor}"),
    }
}

/// Swap active workspaces between two monitors.
#[must_use]
pub fn swap_active_workspaces(monitor1: &str, monitor2: &str) -> DispatchCmd {
    DispatchCmd {
        name: "swapactiveworkspaces",
        args: format!("{monitor1} {monitor2}"),
    }
}
