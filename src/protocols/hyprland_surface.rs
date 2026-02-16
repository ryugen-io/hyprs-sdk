//! hyprland-surface: per-surface opacity and visible region.
//!
//! Provides [`HyprlandSurfaceClient`] for applying Hyprland-specific surface
//! extensions via the `hyprland_surface_manager_v1` protocol. This includes
//! setting a per-surface opacity multiplier and an optional visible region
//! optimization hint.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::hyprland_surface::HyprlandSurfaceClient;
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = HyprlandSurfaceClient::connect(&wl).unwrap();
//!
//! // Create a surface with Hyprland extensions
//! let handle = client.create_surface().unwrap();
//!
//! // Set 50% opacity (takes effect on next wl_surface.commit)
//! handle.set_opacity(0.5);
//! ```

use std::fmt;

use wayland_client::protocol::{wl_compositor, wl_region, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_hyprland::surface::v1::client::{
    hyprland_surface_manager_v1, hyprland_surface_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Handle to a `wl_surface` with Hyprland surface extensions.
///
/// Created by [`HyprlandSurfaceClient::create_surface`]. Provides methods
/// for setting per-surface opacity and visible region.
pub struct SurfaceHandle {
    surface: wl_surface::WlSurface,
    hyprland_surface: hyprland_surface_v1::HyprlandSurfaceV1,
}

impl SurfaceHandle {
    /// Set the overall opacity multiplier for this surface.
    ///
    /// The value must be in the range `0.0..=1.0`. This multiplier applies
    /// to visual effects such as blur behind the surface in addition to the
    /// surface's content.
    ///
    /// Does not take effect until `wl_surface.commit` is called.
    pub fn set_opacity(&self, opacity: f64) {
        self.hyprland_surface.set_opacity(opacity);
    }

    /// Set the visible region of the surface.
    ///
    /// The visible region is an optimization hint for the compositor that
    /// lets it avoid drawing parts of the surface that are not visible
    /// (alpha == 0). The region is specified in buffer-local coordinates.
    ///
    /// Passing `None` clears the visible region (sets it to empty).
    /// The `wl_region` object can be destroyed immediately after this
    /// call (copy semantics).
    ///
    /// Does not take effect until `wl_surface.commit` is called.
    pub fn set_visible_region(&self, region: Option<&wl_region::WlRegion>) {
        self.hyprland_surface.set_visible_region(region);
    }

    /// Reference to the underlying `wl_surface`.
    #[must_use]
    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        &self.surface
    }

    /// Reference to the underlying `hyprland_surface_v1`.
    #[must_use]
    pub fn hyprland_surface(&self) -> &hyprland_surface_v1::HyprlandSurfaceV1 {
        &self.hyprland_surface
    }
}

impl fmt::Debug for SurfaceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SurfaceHandle")
            .field("surface_id", &self.surface.id())
            .field("hyprland_surface_id", &self.hyprland_surface.id())
            .finish()
    }
}

/// Client for the `hyprland_surface_manager_v1` protocol.
///
/// Creates surfaces with Hyprland-specific extensions for per-surface
/// opacity and visible region control.
pub struct HyprlandSurfaceClient {
    state: HyprlandSurfaceState,
    event_queue: EventQueue<HyprlandSurfaceState>,
    qh: QueueHandle<HyprlandSurfaceState>,
}

impl HyprlandSurfaceClient {
    /// Connect to the Hyprland surface manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `hyprland_surface_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("hyprland_surface_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_surface_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<HyprlandSurfaceState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = HyprlandSurfaceState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_surface_manager_v1".into(),
            ));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Create a new surface with Hyprland extensions.
    ///
    /// Creates a `wl_surface` via `wl_compositor` and attaches a
    /// `hyprland_surface_v1` to it via the manager.
    ///
    /// # Errors
    ///
    /// Returns an error if `wl_compositor` is not available or dispatch fails.
    pub fn create_surface(&mut self) -> HyprResult<SurfaceHandle> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let compositor = state
            .compositor
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_compositor available".into()))?;

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("hyprland_surface_manager_v1".into()))?;

        let surface = compositor.create_surface(qh, ());
        let hyprland_surface = manager.get_hyprland_surface(&surface, qh, ());

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(SurfaceHandle {
            surface,
            hyprland_surface,
        })
    }
}

impl fmt::Debug for HyprlandSurfaceClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HyprlandSurfaceClient")
            .field("has_manager", &self.state.manager.is_some())
            .field("has_compositor", &self.state.compositor.is_some())
            .finish()
    }
}

// -- Internal state -----------------------------------------------------------

struct HyprlandSurfaceState {
    manager: Option<hyprland_surface_manager_v1::HyprlandSurfaceManagerV1>,
    compositor: Option<wl_compositor::WlCompositor>,
}

impl HyprlandSurfaceState {
    fn new() -> Self {
        Self {
            manager: None,
            compositor: None,
        }
    }
}

// -- Dispatch implementations -------------------------------------------------

impl Dispatch<wl_registry::WlRegistry, ()> for HyprlandSurfaceState {
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
                "hyprland_surface_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<hyprland_surface_manager_v1::HyprlandSurfaceManagerV1, (), Self>(
                            name,
                            version.min(2),
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

impl Dispatch<hyprland_surface_manager_v1::HyprlandSurfaceManagerV1, ()> for HyprlandSurfaceState {
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_surface_manager_v1::HyprlandSurfaceManagerV1,
        _event: hyprland_surface_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Manager has no events.
    }
}

impl Dispatch<hyprland_surface_v1::HyprlandSurfaceV1, ()> for HyprlandSurfaceState {
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_surface_v1::HyprlandSurfaceV1,
        _event: hyprland_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Surface extension has no events.
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for HyprlandSurfaceState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Compositor has no events.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for HyprlandSurfaceState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Surface events not needed for Hyprland surface extensions.
    }
}
