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
    // Identity fields derive from Hyprland's internal address scheme and process tracking.
    // Address is a hex pointer from the compositor; class/title have initial vs current
    // variants because XDG clients can change them after map.
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

    // Geometry matches X11/Wayland surface coordinates on the virtual desktop.
    // Position is absolute (not monitor-relative) so multi-monitor layouts work.
    /// Position `[x, y]` on the virtual desktop.
    #[serde(rename = "at")]
    pub position: [i32; 2],

    /// Size `[width, height]` in pixels.
    pub size: [i32; 2],

    // A window always belongs to exactly one workspace on one monitor. These fields
    // let consumers locate the window in the workspace/monitor hierarchy.
    /// Workspace this window belongs to.
    pub workspace: WorkspaceRef,

    /// Monitor ID this window is on.
    pub monitor: i32,

    // Boolean state flags control how the compositor treats the window. They affect
    // rendering, input routing, and layout decisions in the tiling engine.
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

    // Fullscreen has two independent axes: compositor-driven (user toggle) and
    // client-requested (app asks for fullscreen). Both must be tracked separately
    // because they can conflict and have different override semantics.
    /// Internal fullscreen mode (set by compositor/user).
    pub fullscreen: FullscreenMode,

    /// Client-requested fullscreen mode.
    #[serde(rename = "fullscreenClient")]
    pub fullscreen_client: FullscreenMode,

    /// Whether the window was created over a fullscreen window.
    #[serde(default, rename = "overFullscreen")]
    pub over_fullscreen: bool,

    // Groups (tabbed containers) and user tags are stored per-window so that tools
    // can display group membership and filter by custom tags.
    /// Addresses of windows in the same group (empty = not grouped).
    pub grouped: Vec<WindowAddress>,

    /// User-assigned tags.
    pub tags: Vec<String>,

    // Swallowing lets a terminal hide itself when it spawns a GUI child. Focus
    // history position is needed for alt-tab style window switching.
    /// Address of the window being swallowed (`0x0` if none).
    pub swallowing: WindowAddress,

    /// Position in the focus history (`-1` if not in history).
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i32,

    /// Whether this window is preventing idle (e.g. video playback).
    #[serde(rename = "inhibitingIdle")]
    pub inhibiting_idle: bool,

    // XDG metadata is set by the client via xdg-toplevel extensions. These fields
    // are optional in the protocol so they may be empty strings.
    /// XDG application tag.
    #[serde(rename = "xdgTag")]
    pub xdg_tag: String,

    /// XDG application description.
    #[serde(rename = "xdgDescription")]
    pub xdg_description: String,

    /// Content type hint (none, photo, video, game).
    #[serde(rename = "contentType")]
    pub content_type: ContentType,

    // Plugin API fields come from CWindow internals and are only populated when
    // accessed through the Hyprland plugin interface, not standard IPC JSON.
    // They default to false/zero so deserialization from IPC still works.
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
