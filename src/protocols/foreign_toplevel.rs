//! wlr-foreign-toplevel-management: list and control opened windows.
//!
//! Used by taskbars to list windows and perform actions (maximize, minimize, close, activate).

use std::fmt;

/// State flags for a toplevel window (bitmask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ToplevelState(u32);

impl ToplevelState {
    /// The toplevel is maximized.
    pub const MAXIMIZED: Self = Self(1);
    /// The toplevel is minimized.
    pub const MINIMIZED: Self = Self(2);
    /// The toplevel is currently focused/activated.
    pub const ACTIVATED: Self = Self(4);
    /// The toplevel is fullscreen.
    pub const FULLSCREEN: Self = Self(8);

    /// Returns `true` if no state flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if the maximized flag is set.
    #[must_use]
    pub fn is_maximized(self) -> bool {
        self.contains(Self::MAXIMIZED)
    }

    /// Returns `true` if the minimized flag is set.
    #[must_use]
    pub fn is_minimized(self) -> bool {
        self.contains(Self::MINIMIZED)
    }

    /// Returns `true` if the activated flag is set.
    #[must_use]
    pub fn is_activated(self) -> bool {
        self.contains(Self::ACTIVATED)
    }

    /// Returns `true` if the fullscreen flag is set.
    #[must_use]
    pub fn is_fullscreen(self) -> bool {
        self.contains(Self::FULLSCREEN)
    }
}

impl std::ops::BitOr for ToplevelState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Information about a toplevel window.
#[derive(Debug, Clone, Default)]
pub struct ToplevelInfo {
    /// Application identifier (e.g. `"org.mozilla.firefox"`).
    pub app_id: String,
    /// Window title.
    pub title: String,
    /// Current state flags.
    pub state: ToplevelState,
}

/// Action that can be performed on a toplevel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToplevelAction {
    /// Maximize the toplevel.
    Maximize,
    /// Unmaximize the toplevel.
    Unmaximize,
    /// Minimize the toplevel.
    Minimize,
    /// Unminimize the toplevel.
    Unminimize,
    /// Activate (focus) the toplevel.
    Activate,
    /// Close the toplevel.
    Close,
    /// Make the toplevel fullscreen.
    Fullscreen,
    /// Exit fullscreen mode.
    UnFullscreen,
}

impl fmt::Display for ToplevelAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Maximize => write!(f, "maximize"),
            Self::Unmaximize => write!(f, "unmaximize"),
            Self::Minimize => write!(f, "minimize"),
            Self::Unminimize => write!(f, "unminimize"),
            Self::Activate => write!(f, "activate"),
            Self::Close => write!(f, "close"),
            Self::Fullscreen => write!(f, "fullscreen"),
            Self::UnFullscreen => write!(f, "unfullscreen"),
        }
    }
}
