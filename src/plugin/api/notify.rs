use std::os::raw::c_char;

use crate::error::{HyprError, HyprResult};
use crate::plugin::ffi;
use crate::plugin::types::{NotificationIcon, PluginHandle};

/// RGBA color for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// Create a new color from RGBA components (0.0–1.0).
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Create a fully opaque color from RGB.
    #[must_use]
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// White.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Red.
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green.
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue.
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

/// Show a notification in Hyprland's notification area.
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the notification fails.
pub fn add_notification(
    handle: PluginHandle,
    text: &str,
    color: Color,
    time_ms: f32,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle. text is valid UTF-8.
    let result = unsafe {
        ffi::add_notification(
            handle.0,
            text.as_ptr().cast::<c_char>(),
            text.len(),
            color.r,
            color.g,
            color.b,
            color.a,
            time_ms,
        )
    };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to add notification".into()))
    }
}

/// Show a notification with an icon (v2 API).
///
/// # Errors
///
/// Returns [`HyprError::NullHandle`] if the plugin handle is null.
/// Returns [`HyprError::Plugin`] if the notification fails.
pub fn add_notification_v2(
    handle: PluginHandle,
    text: &str,
    time_ms: u64,
    color: Color,
    icon: NotificationIcon,
) -> HyprResult<()> {
    if handle.is_null() {
        return Err(HyprError::NullHandle);
    }

    // SAFETY: We validated the handle. text is valid UTF-8.
    let result = unsafe {
        ffi::add_notification_v2(
            handle.0,
            text.as_ptr().cast::<c_char>(),
            text.len(),
            time_ms,
            color.r,
            color.g,
            color.b,
            color.a,
            icon as u8,
        )
    };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to add notification".into()))
    }
}

/// Queue an asynchronous config reload.
///
/// # Errors
///
/// Returns [`HyprError::Plugin`] if the reload request fails.
pub fn reload_config() -> HyprResult<()> {
    // SAFETY: This function has no handle parameter — it's process-global.
    let result = unsafe { ffi::reload_config() };

    if result {
        Ok(())
    } else {
        Err(HyprError::Plugin("failed to reload config".into()))
    }
}
