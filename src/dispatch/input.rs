//! Input, keyboard, and DPMS dispatchers.

use super::DispatchCmd;

/// Bind mouse drag mode.
///
/// `args` format: `<1|0><movewindow|resizewindow>[1|2]`.
#[must_use]
pub fn mouse(args: &str) -> DispatchCmd {
    DispatchCmd {
        name: "mouse",
        args: args.to_string(),
    }
}

/// Pass input events through to a window matching regex.
#[must_use]
pub fn pass(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "pass",
        args: regex.to_string(),
    }
}

/// Send a synthetic keyboard/mouse shortcut to a window.
///
/// Format: `<modifiers> <key> <window_regex>`.
#[must_use]
pub fn send_shortcut(modifiers: &str, key: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "sendshortcut",
        args: format!("{modifiers} {key} {regex}"),
    }
}

/// Send a key state event to a window.
///
/// Format: `<modifiers> <key> <down|up|repeat> <window_regex>`.
#[must_use]
pub fn send_key_state(modifiers: &str, key: &str, state: &str, regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "sendkeystate",
        args: format!("{modifiers} {key} {state} {regex}"),
    }
}

/// Switch to a keybind submap (or "reset" for default).
#[must_use]
pub fn submap(name: &str) -> DispatchCmd {
    DispatchCmd {
        name: "submap",
        args: name.to_string(),
    }
}

/// Trigger a global shortcut registered by another app.
///
/// Format: `<appid>:<action_name>`.
#[must_use]
pub fn global(shortcut: &str) -> DispatchCmd {
    DispatchCmd {
        name: "global",
        args: shortcut.to_string(),
    }
}

/// Control DPMS (display power management).
///
/// `state`: `on`, `off`, or `toggle`. Optional monitor name.
#[must_use]
pub fn dpms(state: &str, monitor: &str) -> DispatchCmd {
    DispatchCmd {
        name: "dpms",
        args: if monitor.is_empty() {
            state.to_string()
        } else {
            format!("{state} {monitor}")
        },
    }
}
