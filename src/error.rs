/// Errors returned by hypr-sdk operations.
#[derive(Debug, thiserror::Error)]
pub enum HyprError {
    /// I/O error (socket connection, read, write).
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to parse a response or event from Hyprland.
    #[error("parse error: {0}")]
    Parse(String),

    /// Command rejected by Hyprland.
    #[error("command failed: {0}")]
    Command(String),

    /// No running Hyprland instance found.
    #[error("no hyprland instance found")]
    NoInstance,

    /// Instance with given signature not found.
    #[error("instance not found: {0}")]
    InstanceNotFound(String),

    /// Wayland connection failed.
    #[cfg(feature = "wayland")]
    #[error("wayland connect error: {0}")]
    WaylandConnect(String),

    /// Wayland event dispatch error.
    #[cfg(feature = "wayland")]
    #[error("wayland dispatch error: {0}")]
    WaylandDispatch(String),

    /// The compositor does not advertise a required protocol global.
    #[cfg(feature = "wayland")]
    #[error("protocol not supported: {0}")]
    ProtocolNotSupported(String),
}

/// Convenience result type for hypr-sdk.
pub type HyprResult<T> = std::result::Result<T, HyprError>;
