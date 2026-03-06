//! hyprland-focus-grab: grab keyboard/pointer focus.
//!
//! Provides [`FocusGrabClient`] for grabbing input focus via the
//! `hyprland_focus_grab_manager_v1` protocol. When a grab is active,
//! input is restricted to whitelisted surfaces -- clicking outside
//! dismisses the grab.
//!
//! # Example
//!
//! ```no_run
//! use hyprs_sdk::protocols::connection::WaylandConnection;
//! use hyprs_sdk::protocols::focus_grab::FocusGrabClient;
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = FocusGrabClient::connect(&wl).unwrap();
//!
//! // Create a grab (no surfaces whitelisted = immediate clear on click)
//! client.create_grab().unwrap();
//! client.commit().unwrap();
//!
//! // Poll for grab cleared events
//! if client.is_cleared() {
//!     println!("Focus grab was dismissed");
//! }
//! ```

use std::fmt;

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_hyprland::focus_grab::v1::client::{
    hyprland_focus_grab_manager_v1, hyprland_focus_grab_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// State of a focus grab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusGrabState {
    /// Grab is active, client has exclusive focus.
    Active,
    /// Grab was cleared (user clicked outside, compositor dismissed it).
    Cleared,
}

/// Client for the `hyprland_focus_grab_manager_v1` protocol.
///
/// Allows grabbing input focus so that clicking outside whitelisted
/// surfaces dismisses the grab.
pub struct FocusGrabClient {
    state: FocusGrabInternalState,
    event_queue: EventQueue<FocusGrabInternalState>,
    qh: QueueHandle<FocusGrabInternalState>,
}

impl FocusGrabClient {
    /// Connect to the focus grab manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `hyprland_focus_grab_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("hyprland_focus_grab_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_focus_grab_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<FocusGrabInternalState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = FocusGrabInternalState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_focus_grab_manager_v1".into(),
            ));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Create a focus grab.
    ///
    /// The grab is not active until [`commit`](Self::commit) is called.
    ///
    /// # Errors
    ///
    /// Returns an error if a grab is already active.
    pub fn create_grab(&mut self) -> HyprResult<()> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        if state.grab.is_some() {
            return Err(HyprError::WaylandDispatch(
                "focus grab already active".into(),
            ));
        }

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("hyprland_focus_grab_manager_v1".into())
        })?;

        let grab = manager.create_grab(qh, ());
        state.grab = Some(grab);
        state.grab_state = Some(FocusGrabState::Active);

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Commit the current grab whitelist.
    ///
    /// # Errors
    ///
    /// Returns an error if no grab is active or dispatch fails.
    pub fn commit(&mut self) -> HyprResult<()> {
        let Self {
            state, event_queue, ..
        } = self;

        let grab = state
            .grab
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no focus grab active".into()))?;

        grab.commit();

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Destroy the current grab.
    ///
    /// # Errors
    ///
    /// Returns an error if no grab is active or dispatch fails.
    pub fn destroy_grab(&mut self) -> HyprResult<()> {
        let Self {
            state, event_queue, ..
        } = self;

        let grab = state
            .grab
            .take()
            .ok_or_else(|| HyprError::WaylandDispatch("no focus grab active".into()))?;

        grab.destroy();
        state.grab_state = None;

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Current state of the focus grab.
    #[must_use]
    pub fn grab_state(&self) -> Option<FocusGrabState> {
        self.state.grab_state
    }

    /// Returns `true` if the grab has been cleared by the compositor.
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.state.grab_state == Some(FocusGrabState::Cleared)
    }

    /// Re-dispatch events to update grab state.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self {
            state, event_queue, ..
        } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for FocusGrabClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FocusGrabClient")
            .field("grab_state", &self.state.grab_state)
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Holds the bound manager, active grab, and last-known grab state so that
// the client can expose a synchronous API over the async Wayland protocol.

struct FocusGrabInternalState {
    manager: Option<hyprland_focus_grab_manager_v1::HyprlandFocusGrabManagerV1>,
    grab: Option<hyprland_focus_grab_v1::HyprlandFocusGrabV1>,
    grab_state: Option<FocusGrabState>,
}

impl FocusGrabInternalState {
    fn new() -> Self {
        Self {
            manager: None,
            grab: None,
            grab_state: None,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue, even for objects that emit no events we care about.

impl Dispatch<wl_registry::WlRegistry, ()> for FocusGrabInternalState {
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
            && interface == "hyprland_focus_grab_manager_v1"
            && state.manager.is_none()
        {
            let mgr = registry
                .bind::<hyprland_focus_grab_manager_v1::HyprlandFocusGrabManagerV1, (), Self>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<hyprland_focus_grab_manager_v1::HyprlandFocusGrabManagerV1, ()>
    for FocusGrabInternalState
{
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_focus_grab_manager_v1::HyprlandFocusGrabManagerV1,
        _event: hyprland_focus_grab_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<hyprland_focus_grab_v1::HyprlandFocusGrabV1, ()> for FocusGrabInternalState {
    fn event(
        state: &mut Self,
        _proxy: &hyprland_focus_grab_v1::HyprlandFocusGrabV1,
        event: hyprland_focus_grab_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let hyprland_focus_grab_v1::Event::Cleared = event {
            state.grab_state = Some(FocusGrabState::Cleared);
        }
    }
}
