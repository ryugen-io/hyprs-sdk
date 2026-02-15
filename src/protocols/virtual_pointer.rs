//! wlr-virtual-pointer: synthetic mouse/pointer input.
//!
//! Create virtual pointer devices to send synthetic mouse events.

/// Virtual pointer button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ButtonState {
    /// Button is released.
    Released = 0,
    /// Button is pressed.
    Pressed = 1,
}

/// Axis source for scroll events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AxisSource {
    /// Physical scroll wheel.
    Wheel = 0,
    /// Finger on a touchpad.
    Finger = 1,
    /// Continuous scroll source.
    Continuous = 2,
    /// Tilted scroll wheel.
    WheelTilt = 3,
}

/// Scroll axis direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Axis {
    /// Vertical scroll axis.
    Vertical = 0,
    /// Horizontal scroll axis.
    Horizontal = 1,
}

/// A virtual pointer motion event.
#[derive(Debug, Clone, Copy)]
pub struct MotionEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Relative X displacement.
    pub dx: f64,
    /// Relative Y displacement.
    pub dy: f64,
}

/// An absolute motion event.
#[derive(Debug, Clone, Copy)]
pub struct MotionAbsoluteEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Absolute X position (0.0 to 1.0, normalized).
    pub x: f64,
    /// Absolute Y position (0.0 to 1.0, normalized).
    pub y: f64,
    /// Bounding box width.
    pub x_extent: u32,
    /// Bounding box height.
    pub y_extent: u32,
}

/// A button event.
#[derive(Debug, Clone, Copy)]
pub struct ButtonEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Linux input event code (e.g. BTN_LEFT = 0x110).
    pub button: u32,
    /// Button state.
    pub state: ButtonState,
}
