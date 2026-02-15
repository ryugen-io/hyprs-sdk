//! virtual-keyboard: synthetic keyboard input.
//!
//! Create virtual keyboard devices to send synthetic key events.

/// Key state for virtual keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyState {
    /// Key is released.
    Released = 0,
    /// Key is pressed.
    Pressed = 1,
}

/// A virtual key event.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Evdev keycode.
    pub key: u32,
    /// Key state.
    pub state: KeyState,
}

/// Modifier state for the virtual keyboard.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    /// Depressed modifiers (currently held keys).
    pub mods_depressed: u32,
    /// Latched modifiers (toggled on for next key).
    pub mods_latched: u32,
    /// Locked modifiers (e.g. Caps Lock).
    pub mods_locked: u32,
    /// Active keyboard group/layout.
    pub group: u32,
}
