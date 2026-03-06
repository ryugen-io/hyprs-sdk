//! Hook event types for the Hyprland plugin system.
//!
//! Hyprland emits hook events at key lifecycle points. Plugins subscribe
//! via `registerCallbackDynamic` with an event name string. This module
//! provides a strongly-typed enum covering all 50 events found in the
//! Hyprland source.
//!
//! Events are either **cancellable** (input events — setting
//! `CallbackInfo::cancelled = true` stops propagation) or
//! **non-cancellable** (informational).

/// All hook events emitted by Hyprland.
///
/// Each variant documents the C++ data type passed via `std::any` to
/// the callback. In Rust FFI, these arrive as opaque pointers that must
/// be cast to the appropriate type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEvent {
    // Lifecycle hooks fire during compositor startup and config changes, letting plugins
    // initialize state after Hyprland is ready and react to configuration updates.
    /// Compositor is fully initialized and ready.
    /// Data: `nullptr`
    Ready,

    /// Animation tick (called every frame).
    /// Data: `nullptr`
    Tick,

    /// Before config file is reloaded.
    /// Data: `nullptr`
    PreConfigReload,

    /// After config file has been reloaded.
    /// Data: `nullptr`
    ConfigReloaded,

    // Monitor hooks cover the full lifecycle of display outputs, from creation through
    // removal, so plugins can track multi-monitor state and render per-output content.
    /// Before a monitor is added to the layout.
    /// Data: `PHLMONITOR`
    PreMonitorAdded,

    /// Monitor has been added and configured.
    /// Data: `PHLMONITOR`
    MonitorAdded,

    /// Before a monitor is removed.
    /// Data: `PHLMONITOR`
    PreMonitorRemoved,

    /// Monitor has been removed.
    /// Data: `PHLMONITOR`
    MonitorRemoved,

    /// Monitor layout (arrangement) changed.
    /// Data: `nullptr`
    MonitorLayoutChanged,

    /// Before a monitor frame commit.
    /// Data: `PHLMONITOR`
    PreMonitorCommit,

    /// New monitor object created (early, before full setup).
    /// Data: `PHLMONITOR`
    NewMonitor,

    /// Focused monitor changed.
    /// Data: `PHLMONITOR`
    FocusedMon,

    // Workspace hooks let plugins react to virtual desktop changes (creation, destruction,
    // movement between monitors) for workspace-aware UI elements like bars and overviews.
    /// Active workspace changed.
    /// Data: `PHLWORKSPACE`
    Workspace,

    /// Workspace created.
    /// Data: `CWorkspace*`
    CreateWorkspace,

    /// Workspace destroyed.
    /// Data: `CWorkspace*`
    DestroyWorkspace,

    /// Workspace moved to a different monitor.
    /// Data: `std::vector<std::any>{PHLWORKSPACE, PHLMONITOR}`
    MoveWorkspace,

    // Window hooks span the full window lifecycle and state changes. Plugins use these
    // for window management, decoration updates, and focus tracking.
    /// Window is about to be mapped (early, before rules applied).
    /// Data: `PHLWINDOW`
    OpenWindowEarly,

    /// Window has been fully mapped.
    /// Data: `PHLWINDOW`
    OpenWindow,

    /// Window is being unmapped.
    /// Data: `PHLWINDOW`
    CloseWindow,

    /// Window object destroyed.
    /// Data: `PHLWINDOW`
    DestroyWindow,

    /// Window moved to a different workspace.
    /// Data: `std::vector<std::any>{PHLWINDOW, PHLWORKSPACE}`
    MoveWindow,

    /// Window title changed.
    /// Data: `PHLWINDOW`
    WindowTitle,

    /// Active (focused) window changed. Data is `nullptr` when no window is focused.
    /// Data: `PHLWINDOW` or `PHLWINDOW{nullptr}`
    ActiveWindow,

    /// Window marked as urgent.
    /// Data: `PHLWINDOW`
    Urgent,

    /// Window class changed.
    /// Data: `PHLWINDOW`
    WindowClass,

    /// Window pinned/unpinned.
    /// Data: `PHLWINDOW`
    Pin,

    /// Window fullscreen state changed.
    /// Data: `PHLWINDOW`
    Fullscreen,

    /// Window rules re-evaluated.
    /// Data: `PHLWINDOW`
    WindowUpdateRules,

    // Layer surface hooks fire for panels, overlays, and other wlr-layer-shell surfaces,
    // enabling plugins to react to status bars, lock screens, and notification popups.
    /// Layer surface (panel/overlay) opened.
    /// Data: `PHLLS`
    OpenLayer,

    /// Layer surface closed.
    /// Data: `PHLLS`
    CloseLayer,

    // Non-cancellable focus and input hooks are informational only -- plugins observe
    // keyboard focus, layout, and submap changes but cannot block them.
    /// Keyboard focus surface changed. Data is `nullptr` when focus lost.
    /// Data: `SP<CWLSurfaceResource>` or `nullptr`
    KeyboardFocus,

    /// Keyboard layout changed.
    /// Data: `std::vector<std::any>{IKeyboard*, std::string}`
    ActiveLayout,

    /// Keybind submap changed.
    /// Data: `std::string` (submap name)
    Submap,

    // Rendering hooks fire during the draw loop, allowing plugins to inject custom
    // OpenGL draw calls at specific render stages (e.g. overlays, effects).
    /// Render stage event. Use `RenderStage` to determine which phase.
    /// Data: `eRenderStage`
    Render,

    /// Before rendering a monitor frame.
    /// Data: `PHLMONITOR`
    PreRender,

    // Screencopy hooks notify plugins when screen recording or screenshot clients
    // connect/disconnect, so plugins can adjust rendering (e.g. hide sensitive content).
    /// Screencopy/toplevel-export state changed.
    /// Data: `std::vector<uint64_t>{active, frame_count, client_id}`
    Screencast,

    // Cancellable input events allow plugins to intercept and consume input before
    // Hyprland processes it -- setting `cancelled = true` blocks further propagation.
    /// Key pressed. Set `cancelled = true` to consume.
    /// Data: `std::unordered_map<std::string, std::any>`
    KeyPress,

    /// Mouse moved. Set `cancelled = true` to consume.
    /// Data: `Vector2D` (floored coordinates)
    MouseMove,

    /// Mouse button pressed/released. Set `cancelled = true` to consume.
    /// Data: `IPointer::SButtonEvent`
    MouseButton,

    /// Mouse scroll/axis event. Set `cancelled = true` to consume.
    /// Data: `std::unordered_map<std::string, std::any>`
    MouseAxis,

    /// Touch down event. Set `cancelled = true` to consume.
    /// Data: `ITouch::SDownEvent`
    TouchDown,

    /// Touch up event. Set `cancelled = true` to consume.
    /// Data: `ITouch::SUpEvent`
    TouchUp,

    /// Touch move event. Set `cancelled = true` to consume.
    /// Data: `ITouch::SMotionEvent`
    TouchMove,

    /// Tablet pen tip event. Set `cancelled = true` to consume.
    /// Data: `CTablet::STipEvent`
    TabletTip,

    /// Tablet axis event. Set `cancelled = true` to consume.
    /// Data: `CTablet::SAxisEvent`
    TabletAxis,

    /// Tablet button event. Set `cancelled = true` to consume.
    /// Data: `CTablet::SButtonEvent`
    TabletButton,

    /// Tablet proximity event. Set `cancelled = true` to consume.
    /// Data: `CTablet::SProximityEvent`
    TabletProximity,

    /// Touch cancel event. Set `cancelled = true` to consume.
    /// Data: `ITouch::SCancelEvent`
    TouchCancel,

    /// Touchpad swipe gesture started. Set `cancelled = true` to consume.
    /// Data: `IPointer::SSwipeBeginEvent`
    SwipeBegin,

    /// Touchpad swipe gesture updated. Set `cancelled = true` to consume.
    /// Data: `IPointer::SSwipeUpdateEvent`
    SwipeUpdate,

    /// Touchpad swipe gesture ended. Set `cancelled = true` to consume.
    /// Data: `IPointer::SSwipeEndEvent`
    SwipeEnd,

    /// Touchpad pinch gesture started. Set `cancelled = true` to consume.
    /// Data: `IPointer::SPinchBeginEvent`
    PinchBegin,

    /// Touchpad pinch gesture updated. Set `cancelled = true` to consume.
    /// Data: `IPointer::SPinchUpdateEvent`
    PinchUpdate,

    /// Touchpad pinch gesture ended. Set `cancelled = true` to consume.
    /// Data: `IPointer::SPinchEndEvent`
    PinchEnd,
}

impl HookEvent {
    /// The C string name used with `registerCallbackDynamic`.
    #[must_use]
    pub fn event_name(&self) -> &'static str {
        match self {
            // Lifecycle & Config
            Self::Ready => "ready",
            Self::Tick => "tick",
            Self::PreConfigReload => "preConfigReload",
            Self::ConfigReloaded => "configReloaded",

            // Monitor
            Self::PreMonitorAdded => "preMonitorAdded",
            Self::MonitorAdded => "monitorAdded",
            Self::PreMonitorRemoved => "preMonitorRemoved",
            Self::MonitorRemoved => "monitorRemoved",
            Self::MonitorLayoutChanged => "monitorLayoutChanged",
            Self::PreMonitorCommit => "preMonitorCommit",
            Self::NewMonitor => "newMonitor",
            Self::FocusedMon => "focusedMon",

            // Workspace
            Self::Workspace => "workspace",
            Self::CreateWorkspace => "createWorkspace",
            Self::DestroyWorkspace => "destroyWorkspace",
            Self::MoveWorkspace => "moveWorkspace",

            // Window
            Self::OpenWindowEarly => "openWindowEarly",
            Self::OpenWindow => "openWindow",
            Self::CloseWindow => "closeWindow",
            Self::DestroyWindow => "destroyWindow",
            Self::MoveWindow => "moveWindow",
            Self::WindowTitle => "windowTitle",
            Self::ActiveWindow => "activeWindow",
            Self::Urgent => "urgent",
            Self::WindowClass => "windowClass",
            Self::Pin => "pin",
            Self::Fullscreen => "fullscreen",
            Self::WindowUpdateRules => "windowUpdateRules",

            // Layer
            Self::OpenLayer => "openLayer",
            Self::CloseLayer => "closeLayer",

            // Focus & Input
            Self::KeyboardFocus => "keyboardFocus",
            Self::ActiveLayout => "activeLayout",
            Self::Submap => "submap",

            // Rendering
            Self::Render => "render",
            Self::PreRender => "preRender",

            // Screencopy
            Self::Screencast => "screencast",

            // Cancellable input
            Self::KeyPress => "keyPress",
            Self::MouseMove => "mouseMove",
            Self::MouseButton => "mouseButton",
            Self::MouseAxis => "mouseAxis",
            Self::TouchDown => "touchDown",
            Self::TouchUp => "touchUp",
            Self::TouchMove => "touchMove",
            Self::TabletTip => "tabletTip",
            Self::TabletAxis => "tabletAxis",
            Self::TabletButton => "tabletButton",
            Self::TabletProximity => "tabletProximity",
            Self::TouchCancel => "touchCancel",
            Self::SwipeBegin => "swipeBegin",
            Self::SwipeUpdate => "swipeUpdate",
            Self::SwipeEnd => "swipeEnd",
            Self::PinchBegin => "pinchBegin",
            Self::PinchUpdate => "pinchUpdate",
            Self::PinchEnd => "pinchEnd",
        }
    }

    /// Whether this event is cancellable.
    ///
    /// Cancellable events allow callbacks to set `CallbackInfo::cancelled = true`
    /// to prevent the event from being processed further.
    #[must_use]
    pub fn is_cancellable(&self) -> bool {
        matches!(
            self,
            Self::KeyPress
                | Self::MouseMove
                | Self::MouseButton
                | Self::MouseAxis
                | Self::TouchDown
                | Self::TouchUp
                | Self::TouchMove
                | Self::TouchCancel
                | Self::TabletTip
                | Self::TabletAxis
                | Self::TabletButton
                | Self::TabletProximity
                | Self::SwipeBegin
                | Self::SwipeUpdate
                | Self::SwipeEnd
                | Self::PinchBegin
                | Self::PinchUpdate
                | Self::PinchEnd
        )
    }

    /// Parse from an event name string.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ready" => Some(Self::Ready),
            "tick" => Some(Self::Tick),
            "preConfigReload" => Some(Self::PreConfigReload),
            "configReloaded" => Some(Self::ConfigReloaded),
            "preMonitorAdded" => Some(Self::PreMonitorAdded),
            "monitorAdded" => Some(Self::MonitorAdded),
            "preMonitorRemoved" => Some(Self::PreMonitorRemoved),
            "monitorRemoved" => Some(Self::MonitorRemoved),
            "monitorLayoutChanged" => Some(Self::MonitorLayoutChanged),
            "preMonitorCommit" => Some(Self::PreMonitorCommit),
            "newMonitor" => Some(Self::NewMonitor),
            "focusedMon" => Some(Self::FocusedMon),
            "workspace" => Some(Self::Workspace),
            "createWorkspace" => Some(Self::CreateWorkspace),
            "destroyWorkspace" => Some(Self::DestroyWorkspace),
            "moveWorkspace" => Some(Self::MoveWorkspace),
            "openWindowEarly" => Some(Self::OpenWindowEarly),
            "openWindow" => Some(Self::OpenWindow),
            "closeWindow" => Some(Self::CloseWindow),
            "destroyWindow" => Some(Self::DestroyWindow),
            "moveWindow" => Some(Self::MoveWindow),
            "windowTitle" => Some(Self::WindowTitle),
            "activeWindow" => Some(Self::ActiveWindow),
            "urgent" => Some(Self::Urgent),
            "windowClass" => Some(Self::WindowClass),
            "pin" => Some(Self::Pin),
            "fullscreen" => Some(Self::Fullscreen),
            "windowUpdateRules" => Some(Self::WindowUpdateRules),
            "openLayer" => Some(Self::OpenLayer),
            "closeLayer" => Some(Self::CloseLayer),
            "keyboardFocus" => Some(Self::KeyboardFocus),
            "activeLayout" => Some(Self::ActiveLayout),
            "submap" => Some(Self::Submap),
            "render" => Some(Self::Render),
            "preRender" => Some(Self::PreRender),
            "screencast" => Some(Self::Screencast),
            "keyPress" => Some(Self::KeyPress),
            "mouseMove" => Some(Self::MouseMove),
            "mouseButton" => Some(Self::MouseButton),
            "mouseAxis" => Some(Self::MouseAxis),
            "touchDown" => Some(Self::TouchDown),
            "touchUp" => Some(Self::TouchUp),
            "touchMove" => Some(Self::TouchMove),
            "tabletTip" => Some(Self::TabletTip),
            "tabletAxis" => Some(Self::TabletAxis),
            "tabletButton" => Some(Self::TabletButton),
            "tabletProximity" => Some(Self::TabletProximity),
            "touchCancel" => Some(Self::TouchCancel),
            "swipeBegin" => Some(Self::SwipeBegin),
            "swipeUpdate" => Some(Self::SwipeUpdate),
            "swipeEnd" => Some(Self::SwipeEnd),
            "pinchBegin" => Some(Self::PinchBegin),
            "pinchUpdate" => Some(Self::PinchUpdate),
            "pinchEnd" => Some(Self::PinchEnd),
            _ => None,
        }
    }

    /// All known hook events.
    pub const ALL: &[HookEvent] = &[
        Self::Ready,
        Self::Tick,
        Self::PreConfigReload,
        Self::ConfigReloaded,
        Self::PreMonitorAdded,
        Self::MonitorAdded,
        Self::PreMonitorRemoved,
        Self::MonitorRemoved,
        Self::MonitorLayoutChanged,
        Self::PreMonitorCommit,
        Self::NewMonitor,
        Self::FocusedMon,
        Self::Workspace,
        Self::CreateWorkspace,
        Self::DestroyWorkspace,
        Self::MoveWorkspace,
        Self::OpenWindowEarly,
        Self::OpenWindow,
        Self::CloseWindow,
        Self::DestroyWindow,
        Self::MoveWindow,
        Self::WindowTitle,
        Self::ActiveWindow,
        Self::Urgent,
        Self::WindowClass,
        Self::Pin,
        Self::Fullscreen,
        Self::WindowUpdateRules,
        Self::OpenLayer,
        Self::CloseLayer,
        Self::KeyboardFocus,
        Self::ActiveLayout,
        Self::Submap,
        Self::Render,
        Self::PreRender,
        Self::Screencast,
        Self::KeyPress,
        Self::MouseMove,
        Self::MouseButton,
        Self::MouseAxis,
        Self::TouchDown,
        Self::TouchUp,
        Self::TouchMove,
        Self::TabletTip,
        Self::TabletAxis,
        Self::TabletButton,
        Self::TabletProximity,
        Self::TouchCancel,
        Self::SwipeBegin,
        Self::SwipeUpdate,
        Self::SwipeEnd,
        Self::PinchBegin,
        Self::PinchUpdate,
        Self::PinchEnd,
    ];

    /// Total number of known hook events.
    pub const COUNT: usize = Self::ALL.len();
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event_name())
    }
}
