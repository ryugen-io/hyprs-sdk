//! hyprland-global-shortcuts: register global keyboard shortcuts.
//!
//! External apps can register shortcuts that work even when the app is not focused.

/// A global shortcut registration.
#[derive(Debug, Clone)]
pub struct ShortcutInfo {
    /// Unique identifier for the shortcut within the app.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Preferred trigger key binding (e.g. `"SUPER+P"`). Advisory only.
    pub preferred_trigger: String,
}

/// Event when a global shortcut is triggered.
#[derive(Debug, Clone)]
pub struct ShortcutEvent {
    /// The shortcut ID that was triggered.
    pub id: String,
    /// Timestamp in milliseconds.
    pub time: u32,
}
