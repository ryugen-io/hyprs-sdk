//! Move, resize, and swap dispatchers.

use super::{Corner, Direction, DispatchCmd};

/// Move the active window in a direction.
#[must_use]
pub fn move_window(dir: Direction, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movewindow",
        args: if regex.is_empty() {
            dir.as_str().to_string()
        } else {
            format!("{},{regex}", dir.as_str())
        },
    }
}

/// Swap the active window with another in a direction.
#[must_use]
pub fn swap_window(dir: Direction) -> DispatchCmd {
    DispatchCmd {
        name: "swapwindow",
        args: dir.as_str().to_string(),
    }
}

/// Swap with the next/previous window in cycle.
///
/// Options: `last`, `prev`.
#[must_use]
pub fn swap_next(opts: &str) -> DispatchCmd {
    DispatchCmd {
        name: "swapnext",
        args: opts.to_string(),
    }
}

/// Move active window by relative offset.
///
/// Takes expressions like `+50 -30`.
#[must_use]
pub fn move_active(x: &str, y: &str) -> DispatchCmd {
    DispatchCmd {
        name: "moveactive",
        args: format!("{x} {y}"),
    }
}

/// Resize active window by relative offset.
///
/// Takes expressions like `+100 -50`.
#[must_use]
pub fn resize_active(w: &str, h: &str) -> DispatchCmd {
    DispatchCmd {
        name: "resizeactive",
        args: format!("{w} {h}"),
    }
}

/// Move a window by exact pixels.
#[must_use]
pub fn move_window_pixel(x: &str, y: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movewindowpixel",
        args: format!("{x} {y},{regex}"),
    }
}

/// Resize a window by exact pixels.
#[must_use]
pub fn resize_window_pixel(w: &str, h: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "resizewindowpixel",
        args: format!("{w} {h},{regex}"),
    }
}

/// Move active window to a workspace.
#[must_use]
pub fn move_to_workspace(workspace: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movetoworkspace",
        args: workspace.to_string(),
    }
}

/// Move active window to workspace (with optional window regex).
#[must_use]
pub fn move_to_workspace_window(workspace: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movetoworkspace",
        args: format!("{workspace},{regex}"),
    }
}

/// Move active window to workspace silently (no focus change).
#[must_use]
pub fn move_to_workspace_silent(workspace: &str) -> DispatchCmd {
    DispatchCmd {
        name: "movetoworkspacesilent",
        args: workspace.to_string(),
    }
}

/// Move cursor to absolute position.
#[must_use]
pub fn move_cursor(x: i32, y: i32) -> DispatchCmd {
    DispatchCmd {
        name: "movecursor",
        args: format!("{x} {y}"),
    }
}

/// Move cursor to a corner of the active window.
#[must_use]
pub fn move_cursor_to_corner(corner: Corner) -> DispatchCmd {
    DispatchCmd {
        name: "movecursortocorner",
        args: (corner as u8).to_string(),
    }
}
