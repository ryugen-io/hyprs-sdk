//! idle-inhibit: prevent the compositor from going idle.
//!
//! Provides [`IdleInhibitClient`] for creating idle inhibitors via the
//! `zwp_idle_inhibit_manager_v1` protocol. While an inhibitor is active,
//! the compositor will not trigger idle timeouts (screen blanking, DPMS, etc.).

use std::fmt;

use wayland_client::protocol::{wl_compositor, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1, zwp_idle_inhibitor_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Client for the `zwp_idle_inhibit_manager_v1` protocol.
///
/// Creates idle inhibitors that prevent the compositor from going idle
/// (blanking the screen, activating DPMS, running screen saver, etc.).
///
/// The inhibitor is tied to a `wl_surface`. This client creates a dummy
/// surface automatically for convenience.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::idle_inhibit::IdleInhibitClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = IdleInhibitClient::connect(&wl).unwrap();
///
/// // Inhibit idle
/// client.inhibit().unwrap();
/// println!("Idle inhibited: {}", client.is_inhibited());
///
/// // Release the inhibitor
/// client.release().unwrap();
/// ```
pub struct IdleInhibitClient {
    state: IdleInhibitState,
    event_queue: EventQueue<IdleInhibitState>,
    qh: QueueHandle<IdleInhibitState>,
}

impl IdleInhibitClient {
    /// Connect to the idle inhibit manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwp_idle_inhibit_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwp_idle_inhibit_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwp_idle_inhibit_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<IdleInhibitState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = IdleInhibitState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwp_idle_inhibit_manager_v1".into(),
            ));
        }

        // The idle-inhibit protocol ties the inhibitor to a wl_surface; create a dummy
        // one since the caller may not have a surface of their own.
        if let Some(ref compositor) = state.compositor {
            let surface = compositor.create_surface(&qh, ());
            state.surface = Some(surface);
        }

        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Create an idle inhibitor.
    ///
    /// While the inhibitor is active, the compositor will not trigger
    /// idle timeouts.
    ///
    /// # Errors
    ///
    /// Returns an error if no surface is available, or if an inhibitor
    /// is already active.
    pub fn inhibit(&mut self) -> HyprResult<()> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        if state.inhibitor.is_some() {
            return Err(HyprError::WaylandDispatch(
                "idle inhibitor already active".into(),
            ));
        }

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwp_idle_inhibit_manager_v1".into()))?;
        let surface = state
            .surface
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_surface available".into()))?;

        let inhibitor = manager.create_inhibitor(surface, qh, ());
        state.inhibitor = Some(inhibitor);

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Release the idle inhibitor.
    ///
    /// # Errors
    ///
    /// Returns an error if no inhibitor is active or dispatch fails.
    pub fn release(&mut self) -> HyprResult<()> {
        let Self {
            state, event_queue, ..
        } = self;

        let inhibitor = state
            .inhibitor
            .take()
            .ok_or_else(|| HyprError::WaylandDispatch("no idle inhibitor active".into()))?;

        inhibitor.destroy();

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Check if an idle inhibitor is currently active.
    #[must_use]
    pub fn is_inhibited(&self) -> bool {
        self.state.inhibitor.is_some()
    }
}

impl fmt::Debug for IdleInhibitClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdleInhibitClient")
            .field("inhibited", &self.state.inhibitor.is_some())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks the manager, compositor, dummy surface, and active inhibitor.

struct IdleInhibitState {
    manager: Option<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    surface: Option<wl_surface::WlSurface>,
    inhibitor: Option<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1>,
}

impl IdleInhibitState {
    fn new() -> Self {
        Self {
            manager: None,
            compositor: None,
            surface: None,
            inhibitor: None,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue, even for objects that emit no events we care about.

impl Dispatch<wl_registry::WlRegistry, ()> for IdleInhibitState {
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
                "zwp_idle_inhibit_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, (), Self>(
                            name,
                            version.min(1),
                            qh,
                            (),
                        );
                    state.manager = Some(mgr);
                }
                "wl_compositor" if state.compositor.is_none() => {
                    let comp = registry.bind::<wl_compositor::WlCompositor, (), Self>(
                        name,
                        version.min(6),
                        qh,
                        (),
                    );
                    state.compositor = Some(comp);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for IdleInhibitState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; compositor events are unused here.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for IdleInhibitState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // The surface is a dummy anchor for the inhibitor; its events are irrelevant.
    }
}

impl Dispatch<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, ()> for IdleInhibitState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1,
        _event: zwp_idle_inhibit_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, ()> for IdleInhibitState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
        _event: zwp_idle_inhibitor_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; inhibitor is request-only.
    }
}
