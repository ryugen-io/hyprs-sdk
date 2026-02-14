//! Focus navigation dispatchers.

use super::{Direction, DispatchCmd};

/// Move focus in a direction.
#[must_use]
pub fn move_focus(dir: Direction) -> DispatchCmd {
    DispatchCmd {
        name: "movefocus",
        args: dir.as_str().to_string(),
    }
}

/// Focus a specific window by regex (class/title).
#[must_use]
pub fn focus_window(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "focuswindow",
        args: regex.to_string(),
    }
}

/// Focus a window by class (alias for `focuswindow`).
#[must_use]
pub fn focus_window_by_class(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "focuswindowbyclass",
        args: regex.to_string(),
    }
}

/// Focus the urgent window, or the last focused one.
#[must_use]
pub fn focus_urgent_or_last() -> DispatchCmd {
    DispatchCmd::no_args("focusurgentorlast")
}

/// Focus current window, or the last focused one.
#[must_use]
pub fn focus_current_or_last() -> DispatchCmd {
    DispatchCmd::no_args("focuscurrentorlast")
}

/// Cycle to the next/previous window.
///
/// Options: `prev`, `next`, `float`, `tiled`, `visible`, `hist`, etc.
#[must_use]
pub fn cycle_next(opts: &str) -> DispatchCmd {
    DispatchCmd {
        name: "cyclenext",
        args: opts.to_string(),
    }
}

/// Focus a monitor by ID, name, or direction.
#[must_use]
pub fn focus_monitor(target: &str) -> DispatchCmd {
    DispatchCmd {
        name: "focusmonitor",
        args: target.to_string(),
    }
}
