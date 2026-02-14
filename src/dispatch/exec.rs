//! Process execution and compositor lifecycle dispatchers.

use super::DispatchCmd;

/// Execute a shell command (with window rule support).
///
/// Supports `[rules] command` syntax for window rules.
#[must_use]
pub fn exec(command: &str) -> DispatchCmd {
    DispatchCmd {
        name: "exec",
        args: command.to_string(),
    }
}

/// Execute a shell command (raw, no rule parsing).
#[must_use]
pub fn execr(command: &str) -> DispatchCmd {
    DispatchCmd {
        name: "execr",
        args: command.to_string(),
    }
}

/// Shutdown Hyprland.
#[must_use]
pub fn exit() -> DispatchCmd {
    DispatchCmd::no_args("exit")
}
