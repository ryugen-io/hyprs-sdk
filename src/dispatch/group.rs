//! Window group dispatchers.

use super::{Direction, DispatchCmd, ToggleState};

/// Toggle grouping on the active window.
#[must_use]
pub fn toggle_group() -> DispatchCmd {
    DispatchCmd::no_args("togglegroup")
}

/// Cycle to next/previous window in the group.
///
/// Options: `f` (forward), `b` (backward), or a numeric index.
#[must_use]
pub fn change_active(target: &str) -> DispatchCmd {
    DispatchCmd {
        name: "changegroupactive",
        args: target.to_string(),
    }
}

/// Reorder a window within its group.
///
/// Options: `f` (forward), `b` (backward).
#[must_use]
pub fn move_window(target: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movegroupwindow",
        args: target.to_string(),
    }
}

/// Lock/unlock group manipulation globally.
#[must_use]
pub fn lock_groups(state: ToggleState) -> DispatchCmd {
    DispatchCmd {
        name: "lockgroups",
        args: state.as_str().to_string(),
    }
}

/// Lock/unlock the active group.
#[must_use]
pub fn lock_active_group(state: ToggleState) -> DispatchCmd {
    DispatchCmd {
        name: "lockactivegroup",
        args: state.as_str().to_string(),
    }
}

/// Move a window into the group in the given direction.
#[must_use]
pub fn move_into_group(dir: Direction) -> DispatchCmd {
    DispatchCmd {
        name: "moveintogroup",
        args: dir.as_str().to_string(),
    }
}

/// Move a window out of its group.
#[must_use]
pub fn move_out_of_group(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "moveoutofgroup",
        args: regex.to_string(),
    }
}

/// Move window or entire group in a direction.
#[must_use]
pub fn move_window_or_group(dir: Direction) -> DispatchCmd {
    DispatchCmd {
        name: "movewindoworgroup",
        args: dir.as_str().to_string(),
    }
}

/// Set ignore group lock state.
#[must_use]
pub fn set_ignore_group_lock(state: ToggleState) -> DispatchCmd {
    DispatchCmd {
        name: "setignoregrouplock",
        args: state.as_str().to_string(),
    }
}

/// Prevent a window from being added to groups.
#[must_use]
pub fn deny_window_from_group(state: ToggleState) -> DispatchCmd {
    DispatchCmd {
        name: "denywindowfromgroup",
        args: state.as_str().to_string(),
    }
}
