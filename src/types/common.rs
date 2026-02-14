use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Unique address of a Hyprland window (hex pointer value).
///
/// Hyprland identifies windows by their memory address, represented as a
/// hexadecimal value. This newtype wraps `u64` and handles hex
/// serialization/deserialization automatically.
///
/// # Examples
///
/// ```
/// use hypr_sdk::types::common::WindowAddress;
///
/// let addr: WindowAddress = "0x55a3f2c0".parse().unwrap();
/// assert_eq!(addr.to_string(), "0x55a3f2c0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowAddress(pub u64);

impl fmt::Display for WindowAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl FromStr for WindowAddress {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix("0x").unwrap_or(s);
        u64::from_str_radix(hex, 16).map(WindowAddress)
    }
}

impl Serialize for WindowAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WindowAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Workspace identifier. Negative values are special workspaces.
///
/// Hyprland uses signed integers for workspace IDs. Regular workspaces have
/// positive IDs, while special (scratchpad) workspaces use negative IDs.
///
/// # Examples
///
/// ```
/// use hypr_sdk::types::common::WorkspaceId;
///
/// let ws = WorkspaceId(3);
/// assert!(!ws.is_special());
///
/// let special = WorkspaceId::SPECIAL;
/// assert!(special.is_special());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct WorkspaceId(pub i64);

impl WorkspaceId {
    /// The special (scratchpad) workspace base ID.
    pub const SPECIAL: Self = Self(-99);

    /// Returns true if this is a special workspace (negative ID).
    #[must_use]
    pub fn is_special(self) -> bool {
        self.0 < 0
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monitor identifier.
///
/// Hyprland assigns integer IDs to connected monitors. These IDs are stable
/// for the lifetime of a monitor connection but may be reassigned if monitors
/// are disconnected and reconnected.
///
/// # Examples
///
/// ```
/// use hypr_sdk::types::common::MonitorId;
///
/// let mon = MonitorId(0);
/// assert_eq!(mon.to_string(), "0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MonitorId(pub i64);

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
