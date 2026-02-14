//! Miscellaneous dispatchers.

use super::DispatchCmd;

/// Force reload the graphics renderer.
#[must_use]
pub fn force_renderer_reload() -> DispatchCmd {
    DispatchCmd::no_args("forcerendererreload")
}

/// Post a custom IPC event (appears on Socket2).
#[must_use]
pub fn event(data: &str) -> DispatchCmd {
    DispatchCmd {
        name: "event",
        args: data.to_string(),
    }
}

/// Force idle timeout.
///
/// Takes milliseconds or `+/-` relative adjustment.
#[must_use]
pub fn force_idle(duration: &str) -> DispatchCmd {
    DispatchCmd {
        name: "forceidle",
        args: duration.to_string(),
    }
}
