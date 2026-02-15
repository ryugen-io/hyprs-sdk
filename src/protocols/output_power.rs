//! wlr-output-power-management: DPMS control for outputs.

use std::fmt;

/// Power mode for an output (DPMS state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PowerMode {
    /// Output is powered on and displaying content.
    On = 0,
    /// Output is powered off (DPMS standby/suspend/off).
    Off = 1,
}

impl PowerMode {
    /// Convert a raw protocol value to a `PowerMode`.
    ///
    /// Returns `None` for unrecognized values.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::On),
            1 => Some(Self::Off),
            _ => None,
        }
    }
}

impl fmt::Display for PowerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
        }
    }
}
