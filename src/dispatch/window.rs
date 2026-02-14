//! Window state dispatchers — kill, float, pin, fullscreen, properties.

use super::DispatchCmd;

/// Kill the active window (SIGKILL).
#[must_use]
pub fn kill_active() -> DispatchCmd {
    DispatchCmd::no_args("killactive")
}

/// Force-kill the active window (same as kill_active).
#[must_use]
pub fn force_kill_active() -> DispatchCmd {
    DispatchCmd::no_args("forcekillactive")
}

/// Close a window matching the regex.
#[must_use]
pub fn close_window(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "closewindow",
        args: regex.to_string(),
    }
}

/// Kill a window matching the regex (SIGKILL).
#[must_use]
pub fn kill_window(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "killwindow",
        args: regex.to_string(),
    }
}

/// Send a Unix signal (1-31) to the active window's process.
#[must_use]
pub fn signal(sig: u8) -> DispatchCmd {
    DispatchCmd {
        name: "signal",
        args: sig.to_string(),
    }
}

/// Send a Unix signal to a specific window's process.
#[must_use]
pub fn signal_window(regex: &str, sig: u8) -> DispatchCmd {
    DispatchCmd {
        name: "signalwindow",
        args: format!("{regex},{sig}"),
    }
}

/// Toggle floating mode for a window.
#[must_use]
pub fn toggle_floating(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "togglefloating",
        args: regex.to_string(),
    }
}

/// Force a window to floating.
#[must_use]
pub fn set_floating(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "setfloating",
        args: regex.to_string(),
    }
}

/// Force a window to tiled.
#[must_use]
pub fn set_tiled(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "settiled",
        args: regex.to_string(),
    }
}

/// Toggle pin (sticky across workspaces, float-only).
#[must_use]
pub fn pin(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "pin",
        args: regex.to_string(),
    }
}

/// Toggle window swallowing.
#[must_use]
pub fn toggle_swallow() -> DispatchCmd {
    DispatchCmd::no_args("toggleswallow")
}

/// Bring active floating window to top of z-order.
#[must_use]
pub fn bring_active_to_top() -> DispatchCmd {
    DispatchCmd::no_args("bringactivetotop")
}

/// Change window z-order.
#[must_use]
pub fn alter_zorder(position: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "alterzorder",
        args: format!("{position},{regex}"),
    }
}

/// Center the active floating window on its monitor.
#[must_use]
pub fn center_window() -> DispatchCmd {
    DispatchCmd::no_args("centerwindow")
}

/// Set a window property.
#[must_use]
pub fn set_prop(regex: &str, property: &str, value: &str) -> DispatchCmd {
    DispatchCmd {
        name: "setprop",
        args: format!("{regex} {property} {value}"),
    }
}

/// Tag a window.
#[must_use]
pub fn tag_window(args: &str) -> DispatchCmd {
    DispatchCmd {
        name: "tagwindow",
        args: args.to_string(),
    }
}

/// Toggle fullscreen (0 = fullscreen, 1 = maximized).
#[must_use]
pub fn fullscreen(mode: u8) -> DispatchCmd {
    DispatchCmd {
        name: "fullscreen",
        args: mode.to_string(),
    }
}

/// Set specific fullscreen state (internal + client modes).
#[must_use]
pub fn fullscreen_state(internal: &str, client: &str) -> DispatchCmd {
    DispatchCmd {
        name: "fullscreenstate",
        args: format!("{internal} {client}"),
    }
}
