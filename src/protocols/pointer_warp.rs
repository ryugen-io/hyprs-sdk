//! pointer-warp: warp the cursor to a position relative to a surface.
//!
//! Provides [`PointerWarpClient`] for warping the pointer via the
//! `wp_pointer_warp_v1` protocol.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::pointer_warp::PointerWarpClient;
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = PointerWarpClient::connect(&wl).unwrap();
//!
//! // Warp the pointer to (100.0, 200.0) relative to the client surface
//! client.warp(100.0, 200.0).unwrap();
//! ```

use std::fmt;

use wayland_client::protocol::{wl_compositor, wl_pointer, wl_registry, wl_seat, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::wp::pointer_warp::v1::client::wp_pointer_warp_v1;

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// A cursor warp target position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarpTarget {
    /// X position relative to the surface.
    pub x: f64,
    /// Y position relative to the surface.
    pub y: f64,
}

impl WarpTarget {
    /// Create a new warp target.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Client for the `wp_pointer_warp_v1` protocol.
///
/// Warps the pointer to a position relative to a surface. This client
/// creates a dummy surface and binds a pointer from the compositor's
/// seat for convenience.
pub struct PointerWarpClient {
    state: PointerWarpState,
    event_queue: EventQueue<PointerWarpState>,
}

impl PointerWarpClient {
    /// Connect to the pointer warp manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `wp_pointer_warp_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("wp_pointer_warp_v1") {
            return Err(HyprError::ProtocolNotSupported("wp_pointer_warp_v1".into()));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<PointerWarpState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = PointerWarpState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.warp_manager.is_none() {
            return Err(HyprError::ProtocolNotSupported("wp_pointer_warp_v1".into()));
        }

        // The warp protocol requires a wl_surface reference; create a dummy one
        // since the caller may not have a surface of their own.
        if let Some(ref compositor) = state.compositor {
            let surface = compositor.create_surface(&qh, ());
            state.surface = Some(surface);
        }

        // The warp protocol requires a wl_pointer; obtain one from the seat
        // so callers don't need to manage pointer objects themselves.
        if let Some(ref seat) = state.seat {
            let pointer = seat.get_pointer(&qh, ());
            state.pointer = Some(pointer);
        }

        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// Warp the pointer to a position relative to the client surface.
    ///
    /// # Errors
    ///
    /// Returns an error if the pointer, surface, or manager is unavailable.
    pub fn warp(&mut self, x: f64, y: f64) -> HyprResult<()> {
        let warp = self
            .state
            .warp_manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("wp_pointer_warp_v1".into()))?;
        let surface = self
            .state
            .surface
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_surface available".into()))?;
        let pointer = self
            .state
            .pointer
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_pointer available".into()))?;

        warp.warp_pointer(surface, pointer, x, y, 0);

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Warp the pointer to a [`WarpTarget`] position.
    ///
    /// # Errors
    ///
    /// Returns an error if the pointer, surface, or manager is unavailable.
    pub fn warp_to(&mut self, target: WarpTarget) -> HyprResult<()> {
        self.warp(target.x, target.y)
    }
}

impl fmt::Debug for PointerWarpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointerWarpClient")
            .field("has_pointer", &self.state.pointer.is_some())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks all the protocol objects the warp operation depends on: the manager,
// compositor, seat, and the dummy surface + pointer created from them.

struct PointerWarpState {
    warp_manager: Option<wp_pointer_warp_v1::WpPointerWarpV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    seat: Option<wl_seat::WlSeat>,
    surface: Option<wl_surface::WlSurface>,
    pointer: Option<wl_pointer::WlPointer>,
}

impl PointerWarpState {
    fn new() -> Self {
        Self {
            warp_manager: None,
            compositor: None,
            seat: None,
            surface: None,
            pointer: None,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue, even for objects that emit no events we care about.

impl Dispatch<wl_registry::WlRegistry, ()> for PointerWarpState {
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
                "wp_pointer_warp_v1" if state.warp_manager.is_none() => {
                    let mgr = registry.bind::<wp_pointer_warp_v1::WpPointerWarpV1, (), Self>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    );
                    state.warp_manager = Some(mgr);
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
                "wl_seat" if state.seat.is_none() => {
                    let seat =
                        registry.bind::<wl_seat::WlSeat, (), Self>(name, version.min(9), qh, ());
                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wp_pointer_warp_v1::WpPointerWarpV1, ()> for PointerWarpState {
    fn event(
        _state: &mut Self,
        _proxy: &wp_pointer_warp_v1::WpPointerWarpV1,
        _event: wp_pointer_warp_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for PointerWarpState {
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

impl Dispatch<wl_seat::WlSeat, ()> for PointerWarpState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We only need the seat to obtain a wl_pointer; its events are irrelevant.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for PointerWarpState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // The surface is a dummy target for warp coordinates; its events are irrelevant.
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for PointerWarpState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        _event: wl_pointer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We only use the pointer as a warp target; its motion/button events are irrelevant.
    }
}
