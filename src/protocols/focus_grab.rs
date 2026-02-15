//! hyprland-focus-grab: grab keyboard/pointer focus.
//!
//! Allows a client to grab input focus, e.g. for popup menus or dropdowns
//! that need to dismiss when clicking outside.

/// State of a focus grab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusGrabState {
    /// Grab is active, client has exclusive focus.
    Active,
    /// Grab was cleared (user clicked outside, compositor dismissed it).
    Cleared,
}
