//! wlr-output-management: monitor configuration protocol.

/// A display mode supported by an output.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputMode {
    /// Horizontal resolution in pixels.
    pub width: i32,
    /// Vertical resolution in pixels.
    pub height: i32,
    /// Refresh rate in millihertz (e.g., 60000 = 60 Hz).
    pub refresh: i32,
    /// Whether this is the output's preferred mode.
    pub preferred: bool,
}

impl OutputMode {
    /// Convert the refresh rate from millihertz to hertz.
    #[must_use]
    pub fn refresh_hz(&self) -> f64 {
        self.refresh as f64 / 1000.0
    }
}

/// Describes the current state of an output (monitor/display).
#[derive(Debug, Clone, Default)]
pub struct OutputHead {
    /// Connector name (e.g., "DP-1", "HDMI-A-1").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Physical width in millimeters.
    pub physical_width: i32,
    /// Physical height in millimeters.
    pub physical_height: i32,
    /// Supported display modes.
    pub modes: Vec<OutputMode>,
    /// Whether the output is currently enabled.
    pub enabled: bool,
    /// Index into `modes` for the currently active mode.
    pub current_mode: Option<usize>,
    /// Horizontal position in the global compositor space.
    pub position_x: i32,
    /// Vertical position in the global compositor space.
    pub position_y: i32,
    /// Output scale factor.
    pub scale: f64,
    /// Output transform (rotation/reflection).
    pub transform: i32,
    /// Manufacturer name.
    pub make: String,
    /// Model name.
    pub model: String,
    /// Serial number.
    pub serial_number: String,
}

/// A configuration entry for applying output settings.
#[derive(Debug, Clone)]
pub struct OutputConfigEntry {
    /// Connector name to configure.
    pub name: String,
    /// Whether to enable or disable the output.
    pub enabled: bool,
    /// Desired display mode, if changing.
    pub mode: Option<OutputMode>,
    /// Desired horizontal position, if changing.
    pub position_x: Option<i32>,
    /// Desired vertical position, if changing.
    pub position_y: Option<i32>,
    /// Desired scale factor, if changing.
    pub scale: Option<f64>,
    /// Desired transform, if changing.
    pub transform: Option<i32>,
}
