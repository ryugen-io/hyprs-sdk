//! ext-foreign-toplevel-list: read-only window listing.
//!
//! Provides [`ExtForeignToplevelListClient`] for listing open toplevel
//! windows via the `ext_foreign_toplevel_list_v1` protocol. This is a
//! simpler, read-only alternative to the wlr foreign-toplevel-management
//! protocol -- it exposes title, app_id, and a stable identifier but
//! offers no control requests.

use std::fmt;

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, event_created_child};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1, ext_foreign_toplevel_list_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Information about a toplevel window from the ext-foreign-toplevel-list protocol.
#[derive(Debug, Clone, Default)]
pub struct ExtToplevelInfo {
    /// Stable unique identifier for this toplevel.
    ///
    /// This identifier is assigned by the compositor and persists across
    /// done events. It can be used to correlate handles from different
    /// protocol instances.
    pub identifier: String,
    /// Window title.
    pub title: String,
    /// Application identifier (e.g. `"org.mozilla.firefox"`).
    pub app_id: String,
    /// Whether the toplevel has been closed.
    pub closed: bool,
}

/// Client for the `ext_foreign_toplevel_list_v1` protocol.
///
/// Provides a read-only listing of open toplevel windows with their
/// title, app_id, and a stable identifier. Unlike the wlr variant,
/// this protocol does not support control actions (maximize, minimize,
/// close, etc.).
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::ext_foreign_toplevel_list::ExtForeignToplevelListClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = ExtForeignToplevelListClient::connect(&wl).unwrap();
///
/// for toplevel in client.toplevels() {
///     println!("[{}] {}: {}", toplevel.identifier, toplevel.app_id, toplevel.title);
/// }
/// ```
pub struct ExtForeignToplevelListClient {
    state: ExtForeignToplevelListState,
    event_queue: EventQueue<ExtForeignToplevelListState>,
}

impl ExtForeignToplevelListClient {
    /// Connect to the ext-foreign-toplevel-list manager.
    ///
    /// Binds `ext_foreign_toplevel_list_v1`, discovers all open
    /// toplevel windows, and queries their properties.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `ext_foreign_toplevel_list_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("ext_foreign_toplevel_list_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "ext_foreign_toplevel_list_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<ExtForeignToplevelListState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = ExtForeignToplevelListState::new();

        // Wayland events arrive asynchronously; roundtrip ensures the manager
        // global is bound before we use it.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "ext_foreign_toplevel_list_v1".into(),
            ));
        }

        // The manager sends toplevel events asynchronously after binding; a second
        // roundtrip is needed to receive the handle objects for all open windows.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Property events (title, app_id, identifier) arrive on handles created in
        // the previous roundtrip; a third roundtrip collects them all.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All currently known (non-closed) toplevels.
    #[must_use]
    pub fn toplevels(&self) -> Vec<ExtToplevelInfo> {
        self.state
            .handles
            .iter()
            .filter(|h| !h.closed)
            .map(|h| ExtToplevelInfo {
                identifier: h.identifier.clone(),
                title: h.title.clone(),
                app_id: h.app_id.clone(),
                closed: false,
            })
            .collect()
    }

    /// Re-dispatch events to update toplevel state.
    ///
    /// Call this to pick up new toplevels, closed toplevels, or
    /// property changes since the last call.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for ExtForeignToplevelListClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtForeignToplevelListClient")
            .field("toplevels", &self.state.handles.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks toplevel handles and their properties across roundtrips.

struct ExtForeignToplevelListState {
    manager: Option<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1>,
    handles: Vec<HandleEntry>,
    finished: bool,
}

struct HandleEntry {
    handle: ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    identifier: String,
    title: String,
    app_id: String,
    closed: bool,
}

impl ExtForeignToplevelListState {
    fn new() -> Self {
        Self {
            manager: None,
            handles: Vec::new(),
            finished: false,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for ExtForeignToplevelListState {
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
            && interface == "ext_foreign_toplevel_list_v1"
            && state.manager.is_none()
        {
            let mgr = registry
                .bind::<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1, (), Self>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1, ()>
    for ExtForeignToplevelListState
{
    fn event(
        state: &mut Self,
        _proxy: &ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        event: ext_foreign_toplevel_list_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } => {
                state.handles.push(HandleEntry {
                    handle: toplevel,
                    identifier: String::new(),
                    title: String::new(),
                    app_id: String::new(),
                    closed: false,
                });
            }
            ext_foreign_toplevel_list_v1::Event::Finished => {
                state.finished = true;
            }
            _ => {}
        }
    }

    event_created_child!(ExtForeignToplevelListState, ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1, [
        // wayland-client dispatches child-object creation by opcode, not name;
        // opcode 0 is the toplevel event that spawns a new handle.
        0 => (ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1, ()>
    for ExtForeignToplevelListState
{
    fn event(
        state: &mut Self,
        proxy: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
        event: ext_foreign_toplevel_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(entry) = state.handles.iter_mut().find(|h| h.handle == *proxy) {
            match event {
                ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                    entry.title = title;
                }
                ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                    entry.app_id = app_id;
                }
                ext_foreign_toplevel_handle_v1::Event::Identifier { identifier } => {
                    entry.identifier = identifier;
                }
                ext_foreign_toplevel_handle_v1::Event::Done => {
                    // The protocol batches property updates and signals completion with "done";
                    // no action needed here since we read properties after roundtrip.
                }
                ext_foreign_toplevel_handle_v1::Event::Closed => {
                    entry.closed = true;
                }
                _ => {}
            }
        }
    }
}
