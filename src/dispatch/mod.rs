//! Typed dispatcher command builders.
//!
//! One function per Hyprland dispatcher (72 total), grouped by domain.
//! Each returns a [`DispatchCmd`] to pass to
//! [`HyprlandClient::dispatch_cmd`](crate::ipc::client::HyprlandClient::dispatch_cmd).

pub mod exec;
pub mod focus;
pub mod group;
pub mod input;
pub mod layout;
pub mod misc;
pub mod movement;
pub mod window;
pub mod workspace;

/// A typed dispatch command ready to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchCmd {
    /// Dispatcher name (e.g. "workspace", "killactive").
    pub name: &'static str,
    /// Arguments string (empty for no-arg dispatchers).
    pub args: String,
}

impl DispatchCmd {
    /// Create a dispatch command with no arguments.
    #[must_use]
    pub fn no_args(name: &'static str) -> Self {
        Self {
            name,
            args: String::new(),
        }
    }
}

/// Cardinal direction for focus/move/swap operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// Wire-format string for this direction.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "l",
            Self::Right => "r",
            Self::Up => "u",
            Self::Down => "d",
        }
    }
}

/// Corner index for `movecursortocorner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    BottomLeft = 0,
    BottomRight = 1,
    TopRight = 2,
    TopLeft = 3,
}

/// Toggle/set/unset for dispatchers that support all three modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleState {
    Toggle,
    On,
    Off,
}

impl ToggleState {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Toggle => "toggle",
            Self::On => "on",
            Self::Off => "off",
        }
    }
}
