//! hyprland-global-shortcuts: register global keyboard shortcuts.
//!
//! Provides [`GlobalShortcutsClient`] for registering global shortcuts that
//! work even when the app is not focused, via the
//! `hyprland_global_shortcuts_manager_v1` protocol.
//!
//! # Example
//!
//! ```no_run
//! use hyprs_sdk::protocols::connection::WaylandConnection;
//! use hyprs_sdk::protocols::global_shortcuts::GlobalShortcutsClient;
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = GlobalShortcutsClient::connect(&wl).unwrap();
//!
//! client.register("screenshot", "my-app", "Take screenshot", "SUPER+P").unwrap();
//!
//! loop {
//!     for event in client.poll().unwrap() {
//!         println!("{}: {:?}", event.id, event.kind);
//!     }
//! }
//! ```

use std::fmt;

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_hyprland::global_shortcuts::v1::client::{
    hyprland_global_shortcut_v1, hyprland_global_shortcuts_manager_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Whether a shortcut was pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutEventKind {
    /// The shortcut key combination was pressed.
    Pressed,
    /// The shortcut key combination was released.
    Released,
}

/// Event when a global shortcut is triggered.
#[derive(Debug, Clone)]
pub struct ShortcutEvent {
    /// The shortcut ID that was triggered.
    pub id: String,
    /// Whether the shortcut was pressed or released.
    pub kind: ShortcutEventKind,
    /// Timestamp in nanoseconds (from epoch).
    pub timestamp_ns: u64,
}

/// Information about a registered shortcut.
#[derive(Debug, Clone)]
pub struct ShortcutInfo {
    /// Unique identifier for the shortcut.
    pub id: String,
    /// Application ID.
    pub app_id: String,
    /// Human-readable description.
    pub description: String,
    /// Preferred trigger key binding (e.g. `"SUPER+P"`). Advisory only.
    pub trigger_description: String,
}

/// Client for the `hyprland_global_shortcuts_manager_v1` protocol.
///
/// Registers global keyboard shortcuts that fire even when the app
/// is not focused. Use [`poll`](Self::poll) to receive press/release events.
pub struct GlobalShortcutsClient {
    state: GlobalShortcutsState,
    event_queue: EventQueue<GlobalShortcutsState>,
    qh: QueueHandle<GlobalShortcutsState>,
}

impl GlobalShortcutsClient {
    /// Connect to the global shortcuts manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `hyprland_global_shortcuts_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("hyprland_global_shortcuts_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_global_shortcuts_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<GlobalShortcutsState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = GlobalShortcutsState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_global_shortcuts_manager_v1".into(),
            ));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Register a global shortcut.
    ///
    /// The `id` must be unique per application. The `trigger_description`
    /// is advisory (e.g. `"SUPER+P"`) -- the compositor decides the
    /// actual binding.
    ///
    /// # Errors
    ///
    /// Returns an error if the manager is unavailable or dispatch fails.
    pub fn register(
        &mut self,
        id: &str,
        app_id: &str,
        description: &str,
        trigger_description: &str,
    ) -> HyprResult<()> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("hyprland_global_shortcuts_manager_v1".into())
        })?;

        let handle = manager.register_shortcut(
            id.to_string(),
            app_id.to_string(),
            description.to_string(),
            trigger_description.to_string(),
            qh,
            id.to_string(),
        );

        state.shortcuts.push(ShortcutEntry {
            info: ShortcutInfo {
                id: id.to_string(),
                app_id: app_id.to_string(),
                description: description.to_string(),
                trigger_description: trigger_description.to_string(),
            },
            _handle: handle,
        });

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Poll for shortcut events.
    ///
    /// Dispatches pending Wayland events and returns any shortcut
    /// press/release events that occurred since the last poll.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn poll(&mut self) -> HyprResult<Vec<ShortcutEvent>> {
        let Self {
            state, event_queue, ..
        } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(state.events.drain(..).collect())
    }

    /// All registered shortcuts.
    #[must_use]
    pub fn shortcuts(&self) -> Vec<ShortcutInfo> {
        self.state
            .shortcuts
            .iter()
            .map(|s| s.info.clone())
            .collect()
    }
}

impl fmt::Debug for GlobalShortcutsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlobalShortcutsClient")
            .field("shortcuts", &self.state.shortcuts.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks registered shortcuts and accumulates press/release events between polls.

struct GlobalShortcutsState {
    manager: Option<hyprland_global_shortcuts_manager_v1::HyprlandGlobalShortcutsManagerV1>,
    shortcuts: Vec<ShortcutEntry>,
    events: Vec<ShortcutEvent>,
}

struct ShortcutEntry {
    info: ShortcutInfo,
    _handle: hyprland_global_shortcut_v1::HyprlandGlobalShortcutV1,
}

impl GlobalShortcutsState {
    fn new() -> Self {
        Self {
            manager: None,
            shortcuts: Vec::new(),
            events: Vec::new(),
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for GlobalShortcutsState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "hyprland_global_shortcuts_manager_v1"
            && state.manager.is_none()
        {
            let mgr = registry
                .bind::<hyprland_global_shortcuts_manager_v1::HyprlandGlobalShortcutsManagerV1, (), Self>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<hyprland_global_shortcuts_manager_v1::HyprlandGlobalShortcutsManagerV1, ()>
    for GlobalShortcutsState
{
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_global_shortcuts_manager_v1::HyprlandGlobalShortcutsManagerV1,
        _event: hyprland_global_shortcuts_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<hyprland_global_shortcut_v1::HyprlandGlobalShortcutV1, String>
    for GlobalShortcutsState
{
    fn event(
        state: &mut Self,
        _proxy: &hyprland_global_shortcut_v1::HyprlandGlobalShortcutV1,
        event: hyprland_global_shortcut_v1::Event,
        data: &String,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let (kind, tv_sec_hi, tv_sec_lo, tv_nsec) = match event {
            hyprland_global_shortcut_v1::Event::Pressed {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
            } => (ShortcutEventKind::Pressed, tv_sec_hi, tv_sec_lo, tv_nsec),
            hyprland_global_shortcut_v1::Event::Released {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
            } => (ShortcutEventKind::Released, tv_sec_hi, tv_sec_lo, tv_nsec),
            _ => return,
        };

        let timestamp_ns =
            ((tv_sec_hi as u64) << 32 | tv_sec_lo as u64) * 1_000_000_000 + tv_nsec as u64;

        state.events.push(ShortcutEvent {
            id: data.clone(),
            kind,
            timestamp_ns,
        });
    }
}
