//! Window type — desktop window representation.
//!
//! Models `CWindow` from `src/desktop/view/Window.hpp`.
//! Deserialization target for the `clients` and `activewindow` IPC queries.

use serde::Deserialize;

use super::common::{ContentType, FullscreenMode, WindowAddress, WorkspaceRef};

/// A Hyprland window (client).
///
/// Combines fields from IPC JSON responses and the internal `CWindow` struct.
/// Fields only available via the plugin API are `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    // -- Identity ------------------------------------------------------------

    /// Unique address of this window (hex pointer).
    pub address: WindowAddress,

    /// Process ID of the owning application.
    pub pid: i32,

    /// Current window class (may change at runtime).
    pub class: String,

    /// Current window title (may change at runtime).
    pub title: String,

    /// Window class at creation time.
    #[serde(rename = "initialClass")]
    pub initial_class: String,

    /// Window title at creation time.
    #[serde(rename = "initialTitle")]
    pub initial_title: String,

    // -- Geometry ------------------------------------------------------------

    /// Position `[x, y]` on the virtual desktop.
    #[serde(rename = "at")]
    pub position: [i32; 2],

    /// Size `[width, height]` in pixels.
    pub size: [i32; 2],

    // -- Workspace & monitor -------------------------------------------------

    /// Workspace this window belongs to.
    pub workspace: WorkspaceRef,

    /// Monitor ID this window is on.
    pub monitor: i32,

    // -- State flags ---------------------------------------------------------

    /// Whether the window is mapped (visible to the compositor).
    pub mapped: bool,

    /// Whether the window is hidden.
    pub hidden: bool,

    /// Whether the window is floating (not tiled).
    pub floating: bool,

    /// Whether the window is pseudo-tiled.
    pub pseudo: bool,

    /// Whether the window is pinned (sticky across workspaces).
    pub pinned: bool,

    /// Whether this is an XWayland (X11) window.
    pub xwayland: bool,

    // -- Fullscreen ----------------------------------------------------------

    /// Internal fullscreen mode (set by compositor/user).
    pub fullscreen: FullscreenMode,

    /// Client-requested fullscreen mode.
    #[serde(rename = "fullscreenClient")]
    pub fullscreen_client: FullscreenMode,

    /// Whether the window was created over a fullscreen window.
    #[serde(rename = "overFullscreen")]
    pub over_fullscreen: bool,

    // -- Groups & tags -------------------------------------------------------

    /// Addresses of windows in the same group (empty = not grouped).
    pub grouped: Vec<WindowAddress>,

    /// User-assigned tags.
    pub tags: Vec<String>,

    // -- Swallowing & focus --------------------------------------------------

    /// Address of the window being swallowed (`0x0` if none).
    pub swallowing: WindowAddress,

    /// Position in the focus history (`-1` if not in history).
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i32,

    /// Whether this window is preventing idle (e.g. video playback).
    #[serde(rename = "inhibitingIdle")]
    pub inhibiting_idle: bool,

    // -- XDG metadata --------------------------------------------------------

    /// XDG application tag.
    #[serde(rename = "xdgTag")]
    pub xdg_tag: String,

    /// XDG application description.
    #[serde(rename = "xdgDescription")]
    pub xdg_description: String,

    /// Content type hint (none, photo, video, game).
    #[serde(rename = "contentType")]
    pub content_type: ContentType,

    // -- Fields from CWindow (plugin API) ------------------------------------

    /// Urgency hint is set.
    #[serde(default)]
    pub is_urgent: bool,

    /// Window prefers tearing (immediate presentation).
    #[serde(default)]
    pub tearing_hint: bool,

    /// Window should not receive initial focus.
    #[serde(default)]
    pub no_initial_focus: bool,

    /// X11 window requests no borders.
    #[serde(default)]
    pub x11_doesnt_want_borders: bool,

    /// Window requested floating mode.
    #[serde(default)]
    pub requests_float: bool,

    /// Whether this window is the head of its group.
    #[serde(default)]
    pub group_head: bool,

    /// Whether the group this window belongs to is locked.
    #[serde(default)]
    pub group_locked: bool,
}
