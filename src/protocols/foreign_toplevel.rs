//! wlr-foreign-toplevel-management: list and control opened windows.
//!
//! Provides [`ForeignToplevelClient`] for listing and controlling
//! toplevel windows (for taskbars, window lists) via the
//! `zwlr_foreign_toplevel_manager_v1` protocol.

use std::fmt;

use wayland_client::protocol::{wl_output, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, event_created_child};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// State flags for a toplevel window (bitmask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ToplevelState(u32);

impl ToplevelState {
    /// The toplevel is maximized.
    pub const MAXIMIZED: Self = Self(1);
    /// The toplevel is minimized.
    pub const MINIMIZED: Self = Self(2);
    /// The toplevel is currently focused/activated.
    pub const ACTIVATED: Self = Self(4);
    /// The toplevel is fullscreen.
    pub const FULLSCREEN: Self = Self(8);

    /// Returns `true` if no state flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if the maximized flag is set.
    #[must_use]
    pub fn is_maximized(self) -> bool {
        self.contains(Self::MAXIMIZED)
    }

    /// Returns `true` if the minimized flag is set.
    #[must_use]
    pub fn is_minimized(self) -> bool {
        self.contains(Self::MINIMIZED)
    }

    /// Returns `true` if the activated flag is set.
    #[must_use]
    pub fn is_activated(self) -> bool {
        self.contains(Self::ACTIVATED)
    }

    /// Returns `true` if the fullscreen flag is set.
    #[must_use]
    pub fn is_fullscreen(self) -> bool {
        self.contains(Self::FULLSCREEN)
    }

    /// Parse a protocol state byte array into flags.
    ///
    /// The array contains native-endian `u32` enum values where
    /// 0=maximized, 1=minimized, 2=activated, 3=fullscreen.
    fn from_protocol_array(data: &[u8]) -> Self {
        let mut bits = 0u32;
        for chunk in data.chunks_exact(4) {
            if let Ok(bytes) = <[u8; 4]>::try_from(chunk) {
                let val = u32::from_ne_bytes(bytes);
                if val < 32 {
                    bits |= 1 << val;
                }
            }
        }
        Self(bits)
    }
}

impl std::ops::BitOr for ToplevelState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Information about a toplevel window.
#[derive(Debug, Clone, Default)]
pub struct ToplevelInfo {
    /// Application identifier (e.g. `"org.mozilla.firefox"`).
    pub app_id: String,
    /// Window title.
    pub title: String,
    /// Current state flags.
    pub state: ToplevelState,
}

/// Action that can be performed on a toplevel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToplevelAction {
    /// Maximize the toplevel.
    Maximize,
    /// Unmaximize the toplevel.
    Unmaximize,
    /// Minimize the toplevel.
    Minimize,
    /// Unminimize the toplevel.
    Unminimize,
    /// Activate (focus) the toplevel.
    Activate,
    /// Close the toplevel.
    Close,
    /// Make the toplevel fullscreen.
    Fullscreen,
    /// Exit fullscreen mode.
    UnFullscreen,
}

impl fmt::Display for ToplevelAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Maximize => write!(f, "maximize"),
            Self::Unmaximize => write!(f, "unmaximize"),
            Self::Minimize => write!(f, "minimize"),
            Self::Unminimize => write!(f, "unminimize"),
            Self::Activate => write!(f, "activate"),
            Self::Close => write!(f, "close"),
            Self::Fullscreen => write!(f, "fullscreen"),
            Self::UnFullscreen => write!(f, "unfullscreen"),
        }
    }
}

/// Opaque identifier for a toplevel window.
///
/// Obtained from [`ForeignToplevelEntry`] and used to perform
/// actions via [`ForeignToplevelClient::perform_action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToplevelId(u32);

/// A toplevel window discovered via the foreign toplevel protocol.
#[derive(Debug, Clone)]
pub struct ForeignToplevelEntry {
    /// Opaque identifier for this toplevel.
    pub id: ToplevelId,
    /// Application identifier (e.g. `"org.mozilla.firefox"`).
    pub app_id: String,
    /// Window title.
    pub title: String,
    /// Current state flags.
    pub state: ToplevelState,
}

/// Client for the `zwlr_foreign_toplevel_manager_v1` protocol.
///
/// Lists open windows and allows performing actions (maximize, minimize,
/// close, activate) on them. Used by taskbars and window switchers.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::foreign_toplevel::{ForeignToplevelClient, ToplevelAction};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = ForeignToplevelClient::connect(&wl).unwrap();
///
/// for toplevel in client.toplevels() {
///     println!("{}: {} ({:?})", toplevel.app_id, toplevel.title, toplevel.state);
/// }
///
/// // Close the first toplevel
/// if let Some(toplevel) = client.toplevels().first() {
///     client.perform_action(toplevel.id, ToplevelAction::Close).unwrap();
/// }
/// ```
pub struct ForeignToplevelClient {
    state: ForeignToplevelState,
    event_queue: EventQueue<ForeignToplevelState>,
}

impl ForeignToplevelClient {
    /// Connect to the foreign toplevel manager.
    ///
    /// Binds `zwlr_foreign_toplevel_manager_v1`, discovers all open
    /// toplevel windows, and queries their properties.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_foreign_toplevel_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_foreign_toplevel_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_foreign_toplevel_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<ForeignToplevelState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = ForeignToplevelState::new();

        // Wayland events arrive asynchronously; roundtrip ensures the manager and
        // seat globals are bound before we use them.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_foreign_toplevel_manager_v1".into(),
            ));
        }

        // The manager sends toplevel events asynchronously after binding; a second
        // roundtrip is needed to receive the handle objects for all open windows.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Property events (title, app_id, state) arrive on the handles created in
        // the previous roundtrip; a third roundtrip collects them all.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All currently known (non-closed) toplevels.
    #[must_use]
    pub fn toplevels(&self) -> Vec<ForeignToplevelEntry> {
        self.state
            .handles
            .iter()
            .filter(|h| !h.closed)
            .map(|h| ForeignToplevelEntry {
                id: ToplevelId(h.id),
                app_id: h.app_id.clone(),
                title: h.title.clone(),
                state: h.state,
            })
            .collect()
    }

    /// Perform an action on a toplevel by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the toplevel is not found, was closed,
    /// or if the action requires a `wl_seat` that was not bound.
    pub fn perform_action(&mut self, id: ToplevelId, action: ToplevelAction) -> HyprResult<()> {
        let entry = self
            .state
            .handles
            .iter()
            .find(|h| h.id == id.0 && !h.closed)
            .ok_or_else(|| HyprError::WaylandDispatch(format!("toplevel not found: {id:?}")))?;

        match action {
            ToplevelAction::Maximize => entry.handle.set_maximized(),
            ToplevelAction::Unmaximize => entry.handle.unset_maximized(),
            ToplevelAction::Minimize => entry.handle.set_minimized(),
            ToplevelAction::Unminimize => entry.handle.unset_minimized(),
            ToplevelAction::Activate => {
                let seat = self.state.seat.as_ref().ok_or_else(|| {
                    HyprError::WaylandDispatch("no wl_seat available for activate".into())
                })?;
                entry.handle.activate(seat);
            }
            ToplevelAction::Close => entry.handle.close(),
            ToplevelAction::Fullscreen => entry.handle.set_fullscreen(None),
            ToplevelAction::UnFullscreen => entry.handle.unset_fullscreen(),
        }

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Re-dispatch events to update toplevel state.
    ///
    /// Call this to pick up new toplevels, closed toplevels, or
    /// property changes since the last call.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for ForeignToplevelClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForeignToplevelClient")
            .field("toplevels", &self.state.handles.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks toplevel handles and their properties across roundtrips. Each handle
// accumulates title/app_id/state events until a "done" event commits them.

struct ForeignToplevelState {
    manager: Option<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    handles: Vec<HandleEntry>,
    next_id: u32,
}

struct HandleEntry {
    id: u32,
    handle: zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
    title: String,
    app_id: String,
    state: ToplevelState,
    closed: bool,
}

impl ForeignToplevelState {
    fn new() -> Self {
        Self {
            manager: None,
            seat: None,
            handles: Vec::new(),
            next_id: 0,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue, even for objects that emit no events we care about.

impl Dispatch<wl_registry::WlRegistry, ()> for ForeignToplevelState {
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
        {
            match interface.as_str() {
                "zwlr_foreign_toplevel_manager_v1" if state.manager.is_none() => {
                    let mgr = registry.bind::<
                        zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
                        (),
                        Self,
                    >(name, version.min(3), qh, ());
                    state.manager = Some(mgr);
                }
                "wl_seat" if state.seat.is_none() => {
                    let seat =
                        registry.bind::<wl_seat::WlSeat, (), Self>(name, version.min(1), qh, ());
                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for ForeignToplevelState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We only need the seat proxy for activate requests; its events are irrelevant.
    }
}

impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()>
    for ForeignToplevelState
{
    fn event(
        state: &mut Self,
        _proxy: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                let id = state.next_id;
                state.next_id += 1;
                state.handles.push(HandleEntry {
                    id,
                    handle: toplevel,
                    title: String::new(),
                    app_id: String::new(),
                    state: ToplevelState::default(),
                    closed: false,
                });
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                // The compositor is shutting down this manager instance; no further
                // toplevel events will arrive.
            }
            _ => {}
        }
    }

    event_created_child!(ForeignToplevelState, zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, [
        // wayland-client dispatches child-object creation by opcode, not name;
        // opcode 0 is the toplevel event that spawns a new handle.
        0 => (zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()>
    for ForeignToplevelState
{
    fn event(
        state: &mut Self,
        proxy: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(entry) = state.handles.iter_mut().find(|h| h.handle == *proxy) {
            match event {
                zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                    entry.title = title;
                }
                zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                    entry.app_id = app_id;
                }
                zwlr_foreign_toplevel_handle_v1::Event::State { state: data } => {
                    entry.state = ToplevelState::from_protocol_array(&data);
                }
                zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                    entry.closed = true;
                }
                // Done, OutputEnter, OutputLeave, Parent are informational events we
                // don't need for the basic list-and-control API.
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for ForeignToplevelState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client because toplevel handles can
        // reference wl_output objects; we don't need the output events themselves.
    }
}
