//! ext-workspace: workspace management protocol.
//!
//! Manage and observe workspaces across workspace groups and outputs.

/// Workspace state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceState(u32);

impl WorkspaceState {
    /// The workspace is currently active/focused.
    pub const ACTIVE: Self = Self(1);
    /// The workspace has an urgent notification.
    pub const URGENT: Self = Self(2);
    /// The workspace is hidden.
    pub const HIDDEN: Self = Self(4);

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

    /// Returns `true` if the workspace is active.
    #[must_use]
    pub fn is_active(self) -> bool {
        self.contains(Self::ACTIVE)
    }

    /// Returns `true` if the workspace has an urgent notification.
    #[must_use]
    pub fn is_urgent(self) -> bool {
        self.contains(Self::URGENT)
    }

    /// Returns `true` if the workspace is hidden.
    #[must_use]
    pub fn is_hidden(self) -> bool {
        self.contains(Self::HIDDEN)
    }
}

impl std::ops::BitOr for WorkspaceState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Capabilities of a workspace group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceGroupCapabilities(u32);

impl WorkspaceGroupCapabilities {
    /// The group supports creating new workspaces.
    pub const CREATE_WORKSPACE: Self = Self(1);

    /// Returns `true` if no capabilities are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all capabilities in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Capabilities of an individual workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceCapabilities(u32);

impl WorkspaceCapabilities {
    /// The workspace can be activated.
    pub const ACTIVATE: Self = Self(1);
    /// The workspace can be deactivated.
    pub const DEACTIVATE: Self = Self(2);
    /// The workspace can be removed.
    pub const REMOVE: Self = Self(4);

    /// Returns `true` if no capabilities are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all capabilities in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Workspace coordinate pair for multi-dimensional layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceCoordinates {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}
