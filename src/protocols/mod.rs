//! Wayland protocol client bindings.
//!
//! Client-side bindings for Hyprland-specific and wlr protocols.
//! Requires the `wayland` feature flag.

#[cfg(feature = "wayland")]
pub mod connection;
