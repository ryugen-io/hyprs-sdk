//! Command builders for Socket1 wire format.
//!
//! Covers all 37 registered IPC commands. Each builder returns the raw
//! command string to send over the socket.
//!
//! Wire format: `[FLAGS]/COMMAND[ ARGS]`
//! Flags: `j` = JSON, `r` = reload, `a` = all, `c` = config.

// Hyprland's wire format prefixes commands with single-char flags (j/r/a/c) before a slash.
// Centralising flag encoding here keeps every command builder consistent with the protocol.

/// Output format flags for commands that support them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    /// Request JSON output.
    pub json: bool,
    /// Reload configs before responding.
    pub reload: bool,
    /// Include all objects (e.g. unmapped windows).
    pub all: bool,
    /// Include config information.
    pub config: bool,
}

impl Flags {
    /// JSON-only flags.
    #[must_use]
    pub fn json() -> Self {
        Self {
            json: true,
            ..Default::default()
        }
    }

    fn prefix(&self) -> String {
        let mut s = String::new();
        if self.json {
            s.push('j');
        }
        if self.reload {
            s.push('r');
        }
        if self.all {
            s.push('a');
        }
        if self.config {
            s.push('c');
        }
        s
    }
}

pub(crate) fn flagged_pub(flags: Flags, command: &str) -> String {
    flagged(flags, command)
}

fn flagged(flags: Flags, command: &str) -> String {
    let p = flags.prefix();
    if p.is_empty() {
        command.to_string()
    } else {
        format!("{p}/{command}")
    }
}

// Parameterless commands: Hyprland matches these by exact name with no trailing space.
// They only vary by output-format flags, so each builder just delegates to `flagged()`.

/// List all workspaces.
pub fn workspaces(flags: Flags) -> String {
    flagged(flags, "workspaces")
}

/// List all workspace rules.
pub fn workspace_rules(flags: Flags) -> String {
    flagged(flags, "workspacerules")
}

/// Get the currently focused workspace.
pub fn active_workspace(flags: Flags) -> String {
    flagged(flags, "activeworkspace")
}

/// List all clients (windows).
pub fn clients(flags: Flags) -> String {
    flagged(flags, "clients")
}

/// Activate kill mode (click to kill a window).
pub fn kill() -> String {
    "kill".into()
}

/// Get the currently focused window.
pub fn active_window(flags: Flags) -> String {
    flagged(flags, "activewindow")
}

/// List all layer shell surfaces.
pub fn layers(flags: Flags) -> String {
    flagged(flags, "layers")
}

/// Get Hyprland version info.
pub fn version(flags: Flags) -> String {
    flagged(flags, "version")
}

/// List all input devices.
pub fn devices(flags: Flags) -> String {
    flagged(flags, "devices")
}

/// Get the current splash screen message.
pub fn splash() -> String {
    "splash".into()
}

/// Get the current cursor position.
pub fn cursor_pos(flags: Flags) -> String {
    flagged(flags, "cursorpos")
}

/// List all keybindings.
pub fn binds(flags: Flags) -> String {
    flagged(flags, "binds")
}

/// List registered global shortcuts.
pub fn global_shortcuts(flags: Flags) -> String {
    flagged(flags, "globalshortcuts")
}

/// Get system information.
pub fn system_info(flags: Flags) -> String {
    flagged(flags, "systeminfo")
}

/// List animation states.
pub fn animations(flags: Flags) -> String {
    flagged(flags, "animations")
}

/// Get rolling log output.
pub fn rolling_log(flags: Flags) -> String {
    flagged(flags, "rollinglog")
}

/// List configuration errors.
pub fn config_errors(flags: Flags) -> String {
    flagged(flags, "configerrors")
}

/// Get lock state.
pub fn locked(flags: Flags) -> String {
    flagged(flags, "locked")
}

/// Get command descriptions.
pub fn descriptions(flags: Flags) -> String {
    flagged(flags, "descriptions")
}

/// Get current keybind submap.
pub fn submap() -> String {
    "submap".into()
}

/// Reload shader programs.
pub fn reload_shaders() -> String {
    "reloadshaders".into()
}

// Commands with arguments: Hyprland prefix-matches the command name and treats everything
// after the first space as the argument payload. Each builder handles formatting constraints.

/// List monitors.
pub fn monitors(flags: Flags) -> String {
    flagged(flags, "monitors")
}

/// Reload configuration.
pub fn reload(args: &str) -> String {
    if args.is_empty() {
        "reload".into()
    } else {
        format!("reload {args}")
    }
}

/// Plugin management.
pub fn plugin(operation: &str) -> String {
    format!("plugin {operation}")
}

/// Create a notification.
pub fn notify(icon: i32, time_ms: u32, color: &str, message: &str) -> String {
    format!("notify {icon} {time_ms} {color} {message}")
}

/// Dismiss notifications.
pub fn dismiss_notify(count: i32) -> String {
    format!("dismissnotify {count}")
}

/// Get a window property.
pub fn get_prop(window_address: &str, property: &str, flags: Flags) -> String {
    flagged(flags, &format!("getprop {window_address} {property}"))
}

/// Set error message display.
pub fn set_error(message: &str) -> String {
    if message.is_empty() {
        "seterror disable".into()
    } else {
        format!("seterror {message}")
    }
}

/// Switch XKB keyboard layout.
pub fn switch_xkb_layout(device: &str, cmd: &str) -> String {
    format!("switchxkblayout {device} {cmd}")
}

/// Output/monitor configuration.
pub fn output(args: &str) -> String {
    format!("output {args}")
}

/// Dispatch a compositor action.
pub fn dispatch(dispatcher: &str, args: &str) -> String {
    if args.is_empty() {
        format!("dispatch {dispatcher}")
    } else {
        format!("dispatch {dispatcher} {args}")
    }
}

/// Set a configuration keyword.
pub fn keyword(key: &str, value: &str) -> String {
    format!("keyword {key} {value}")
}

/// Set cursor theme and size.
pub fn set_cursor(theme: &str, size: u32) -> String {
    format!("setcursor {theme} {size}")
}

/// Get a configuration option value.
pub fn get_option(name: &str, flags: Flags) -> String {
    flagged(flags, &format!("getoption {name}"))
}

/// Get window decorations.
pub fn decorations(window_address: &str, flags: Flags) -> String {
    flagged(flags, &format!("decorations {window_address}"))
}

// Batch wraps commands in a `[[BATCH]]` prefix so Hyprland executes them atomically
// in a single compositor tick, avoiding intermediate redraws between related operations.

/// Wrap multiple commands in a batch.
pub fn batch(commands: &[String]) -> String {
    format!("[[BATCH]]{}", commands.join(";"))
}
