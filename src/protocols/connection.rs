//! Wayland display connection and registry global discovery.

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};

use crate::error::{HyprError, HyprResult};

/// Information about a global object advertised by the compositor.
#[derive(Debug, Clone)]
pub struct GlobalInfo {
    /// Server-assigned name for this global.
    pub name: u32,
    /// Interface name (e.g. `"zwlr_layer_shell_v1"`).
    pub interface: String,
    /// Maximum supported version.
    pub version: u32,
}

/// State object used during registry enumeration.
struct RegistryState {
    globals: Vec<GlobalInfo>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for RegistryState {
    fn event(
        state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            state.globals.push(GlobalInfo {
                name,
                interface,
                version,
            });
        }
    }
}

/// Connection to the Wayland display server.
///
/// Entry point for all protocol operations. Connects to the display,
/// enumerates compositor globals, and provides access to the underlying
/// `wayland_client::Connection`.
///
/// # Examples
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
///
/// let wl = WaylandConnection::connect().expect("failed to connect");
/// for global in wl.globals() {
///     println!("{}: v{}", global.interface, global.version);
/// }
/// ```
pub struct WaylandConnection {
    conn: Connection,
    globals: Vec<GlobalInfo>,
}

impl WaylandConnection {
    /// Connect to the Wayland display using `$WAYLAND_DISPLAY`.
    ///
    /// Performs a single roundtrip to enumerate all compositor globals.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::WaylandConnect`] if the display cannot be reached,
    /// or [`HyprError::WaylandDispatch`] if the initial roundtrip fails.
    pub fn connect() -> HyprResult<Self> {
        let conn =
            Connection::connect_to_env().map_err(|e| HyprError::WaylandConnect(e.to_string()))?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> HyprResult<Self> {
        let display = conn.display();
        let mut event_queue: EventQueue<RegistryState> = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());

        let mut state = RegistryState {
            globals: Vec::new(),
        };

        // Roundtrip to receive all registry.global events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            conn,
            globals: state.globals,
        })
    }

    /// All globals advertised by the compositor.
    #[must_use]
    pub fn globals(&self) -> &[GlobalInfo] {
        &self.globals
    }

    /// Find a global by interface name.
    #[must_use]
    pub fn find_global(&self, interface: &str) -> Option<&GlobalInfo> {
        self.globals.iter().find(|g| g.interface == interface)
    }

    /// Check if a protocol is supported by the compositor.
    #[must_use]
    pub fn has_protocol(&self, interface: &str) -> bool {
        self.find_global(interface).is_some()
    }

    /// Access the underlying wayland-client connection.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl std::fmt::Debug for WaylandConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaylandConnection")
            .field("globals_count", &self.globals.len())
            .finish()
    }
}
