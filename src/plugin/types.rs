//! Plugin API types and enums.
//!
//! Maps to types from `SharedDefs.hpp` and `PluginAPI.hpp`.
//!
//! These are pure Rust representations of the C++ types used by the
//! Hyprland plugin API. The actual FFI boundary requires a C++ bridge
//! because Hyprland's plugin API passes C++ objects (`std::string`,
//! `std::function`, `std::any`) over `extern "C"` linkage.

use std::ffi::c_void;
use std::fmt;

/// Opaque plugin handle.
///
/// Passed to `pluginInit` by Hyprland and must be stored for all
/// subsequent API calls. Maps to `HANDLE` (`void*`) in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PluginHandle(pub *mut c_void);

// SAFETY: PluginHandle is an opaque identifier, not a real pointer we dereference.
// Hyprland only uses it to look up plugin state in its internal map.
unsafe impl Send for PluginHandle {}
unsafe impl Sync for PluginHandle {}

impl PluginHandle {
    /// Null handle (invalid).
    pub const NULL: Self = Self(std::ptr::null_mut());

    /// Whether this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Plugin metadata returned from `pluginInit`.
///
/// Maps to `PLUGIN_DESCRIPTION_INFO` in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PluginDescription {
    /// Plugin name.
    pub name: String,
    /// Short description of what the plugin does.
    pub description: String,
    /// Plugin author.
    pub author: String,
    /// Plugin version string.
    pub version: String,
}

/// Result of a custom dispatcher invocation.
///
/// Maps to `SDispatchResult` in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DispatchResult {
    /// If true, pass the event to the next handler in the chain.
    pub pass_event: bool,
    /// Whether the dispatch succeeded.
    pub success: bool,
    /// Error message (empty on success).
    pub error: String,
}

impl DispatchResult {
    /// Successful dispatch result.
    #[must_use]
    pub fn ok() -> Self {
        Self {
            pass_event: false,
            success: true,
            error: String::new(),
        }
    }

    /// Failed dispatch result with an error message.
    #[must_use]
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            pass_event: false,
            success: false,
            error: message.into(),
        }
    }

    /// Successful result that passes the event to the next handler.
    #[must_use]
    pub fn pass() -> Self {
        Self {
            pass_event: true,
            success: true,
            error: String::new(),
        }
    }
}

/// Callback info passed to hook callbacks.
///
/// For cancellable events, setting `cancelled = true` prevents further
/// processing. Maps to `SCallbackInfo` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CallbackInfo {
    /// Set to `true` to cancel the event (only for cancellable events).
    pub cancelled: bool,
}

/// Hyprland version information.
///
/// Maps to `SVersionInfo` in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VersionInfo {
    /// Git commit hash.
    pub hash: String,
    /// Version tag (e.g. "v0.53.0").
    pub tag: String,
    /// Whether the build had uncommitted changes.
    pub dirty: bool,
    /// Git branch name.
    pub branch: String,
    /// Last commit message.
    pub message: String,
    /// Number of commits (as string).
    pub commits: String,
}

/// Notification icon type.
///
/// Maps to `eIcons` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum NotificationIcon {
    Warning = 0,
    Info = 1,
    Hint = 2,
    Error = 3,
    Confused = 4,
    Ok = 5,
    #[default]
    None = 6,
}

impl NotificationIcon {
    /// Parse from the raw integer used in Hyprland source.
    #[must_use]
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Warning),
            1 => Some(Self::Info),
            2 => Some(Self::Hint),
            3 => Some(Self::Error),
            4 => Some(Self::Confused),
            5 => Some(Self::Ok),
            6 => Some(Self::None),
            _ => Option::None,
        }
    }
}

impl fmt::Display for NotificationIcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
            Self::Hint => write!(f, "hint"),
            Self::Error => write!(f, "error"),
            Self::Confused => write!(f, "confused"),
            Self::Ok => write!(f, "ok"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Render stage passed to the `render` hook event.
///
/// Maps to `eRenderStage` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RenderStage {
    /// Before binding the GL context.
    Pre = 0,
    /// Rendering begins, nothing rendered yet. Damage and render data valid.
    Begin = 1,
    /// After background layer, before bottom and overlay layers.
    PostWallpaper = 2,
    /// Before windows, after bottom and overlay layers.
    PreWindows = 3,
    /// After windows, before top/overlay layers.
    PostWindows = 4,
    /// Last moment to render with the GL context.
    LastMoment = 5,
    /// After rendering finished, GL context no longer available.
    Post = 6,
    /// After rendering a mirror.
    PostMirror = 7,
    /// Before rendering a window (any pass). Some windows have 2 passes.
    PreWindow = 8,
    /// After rendering a window (any pass).
    PostWindow = 9,
}

impl RenderStage {
    /// Parse from the raw integer used in Hyprland source.
    #[must_use]
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Pre),
            1 => Some(Self::Begin),
            2 => Some(Self::PostWallpaper),
            3 => Some(Self::PreWindows),
            4 => Some(Self::PostWindows),
            5 => Some(Self::LastMoment),
            6 => Some(Self::Post),
            7 => Some(Self::PostMirror),
            8 => Some(Self::PreWindow),
            9 => Some(Self::PostWindow),
            _ => None,
        }
    }
}

impl fmt::Display for RenderStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pre => write!(f, "RENDER_PRE"),
            Self::Begin => write!(f, "RENDER_BEGIN"),
            Self::PostWallpaper => write!(f, "RENDER_POST_WALLPAPER"),
            Self::PreWindows => write!(f, "RENDER_PRE_WINDOWS"),
            Self::PostWindows => write!(f, "RENDER_POST_WINDOWS"),
            Self::LastMoment => write!(f, "RENDER_LAST_MOMENT"),
            Self::Post => write!(f, "RENDER_POST"),
            Self::PostMirror => write!(f, "RENDER_POST_MIRROR"),
            Self::PreWindow => write!(f, "RENDER_PRE_WINDOW"),
            Self::PostWindow => write!(f, "RENDER_POST_WINDOW"),
        }
    }
}

/// Output format for hyprctl commands.
///
/// Maps to `eHyprCtlOutputFormat` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum HyprCtlOutputFormat {
    #[default]
    Normal = 0,
    Json = 1,
}

/// Input event type for decoration input handling.
///
/// Maps to `eInputType` in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InputType {
    Axis = 0,
    Button = 1,
    DragStart = 2,
    DragEnd = 3,
    Motion = 4,
}

impl InputType {
    /// Parse from the raw integer.
    #[must_use]
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Axis),
            1 => Some(Self::Button),
            2 => Some(Self::DragStart),
            3 => Some(Self::DragEnd),
            4 => Some(Self::Motion),
            _ => None,
        }
    }
}

/// The Hyprland plugin API version string.
///
/// Must be returned by `pluginAPIVersion()`. If the version doesn't
/// match what Hyprland expects, the plugin will be ejected.
pub const HYPRLAND_API_VERSION: &str = "0.1";

/// NUL-terminated API version bytes for C ABI entry points.
pub const HYPRLAND_API_VERSION_CSTR: &[u8] = b"0.1\0";

/// Function match result from `findFunctionsByName`.
///
/// Maps to `SFunctionMatch` in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FunctionMatch {
    /// Address of the matched function.
    pub address: *const c_void,
    /// Mangled C++ signature.
    pub signature: String,
    /// Demangled human-readable name.
    pub demangled: String,
}

// SAFETY: FunctionMatch addresses are stable for the plugin's lifetime
// and are only used for function hooking.
unsafe impl Send for FunctionMatch {}
unsafe impl Sync for FunctionMatch {}
