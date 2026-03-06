//! ext-session-lock: lock screen protocol.
//!
//! Provides [`SessionLockClient`] for acquiring and releasing session locks
//! via the `ext_session_lock_manager_v1` protocol.
//!
//! Note: Creating actual lock screen surfaces requires a rendering backend.
//! This client handles the lock lifecycle (lock/unlock/finished) but does
//! not create surfaces. Use `lock_handle()` to access the underlying lock
//! object for surface creation with your own renderer.

use std::fmt;

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1, ext_session_lock_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// State of the session lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockState {
    /// Lock has been acknowledged by the compositor.
    /// All outputs should now show lock surfaces.
    Locked,
    /// Lock request was finished (session ended or lock dismissed).
    Finished,
}

/// Configuration for a lock surface on a specific output.
#[derive(Debug, Clone)]
pub struct LockSurfaceConfig {
    /// Desired width (usually matches output resolution).
    pub width: u32,
    /// Desired height (usually matches output resolution).
    pub height: u32,
}

impl LockSurfaceConfig {
    /// Create a lock surface config matching output dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Client for the `ext_session_lock_manager_v1` protocol.
///
/// Manages session lock lifecycle. After acquiring a lock, the compositor
/// will blank all outputs until lock surfaces are presented.
///
/// # Example
///
/// ```no_run
/// use hyprs_sdk::protocols::connection::WaylandConnection;
/// use hyprs_sdk::protocols::session_lock::SessionLockClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = SessionLockClient::connect(&wl).unwrap();
///
/// // Acquire the session lock
/// client.lock().unwrap();
/// println!("Lock state: {:?}", client.lock_state());
///
/// // Unlock when done (or compositor sends finished)
/// client.unlock_and_destroy().unwrap();
/// ```
pub struct SessionLockClient {
    state: SessionLockState,
    event_queue: EventQueue<SessionLockState>,
    qh: QueueHandle<SessionLockState>,
}

impl SessionLockClient {
    /// Connect to the session lock manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `ext_session_lock_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("ext_session_lock_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "ext_session_lock_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<SessionLockState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = SessionLockState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "ext_session_lock_manager_v1".into(),
            ));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Acquire the session lock.
    ///
    /// After this call, the compositor will blank all outputs until
    /// lock surfaces are presented. Poll `lock_state()` to check
    /// when the lock is acknowledged.
    ///
    /// # Errors
    ///
    /// Returns an error if a lock is already held or dispatch fails.
    pub fn lock(&mut self) -> HyprResult<()> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        if state.lock.is_some() {
            return Err(HyprError::WaylandDispatch(
                "session lock already held".into(),
            ));
        }

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("ext_session_lock_manager_v1".into()))?;

        let lock = manager.lock(qh, ());
        state.lock = Some(lock);
        state.lock_state = None;

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Get the current lock state, if a lock has been acquired.
    #[must_use]
    pub fn lock_state(&self) -> Option<LockState> {
        self.state.lock_state
    }

    /// Unlock the session and destroy the lock object.
    ///
    /// This should be called when the user successfully authenticates.
    /// The compositor will resume normal display.
    ///
    /// # Errors
    ///
    /// Returns an error if no lock is held or dispatch fails.
    pub fn unlock_and_destroy(&mut self) -> HyprResult<()> {
        let Self {
            state, event_queue, ..
        } = self;

        let lock = state
            .lock
            .take()
            .ok_or_else(|| HyprError::WaylandDispatch("no session lock held".into()))?;

        lock.unlock_and_destroy();
        state.lock_state = None;

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Re-dispatch events to update lock state.
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

impl fmt::Debug for SessionLockClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionLockClient")
            .field("lock_state", &self.state.lock_state)
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks the manager, active lock object, and last-known lock state.

struct SessionLockState {
    manager: Option<ext_session_lock_manager_v1::ExtSessionLockManagerV1>,
    lock: Option<ext_session_lock_v1::ExtSessionLockV1>,
    lock_state: Option<LockState>,
}

impl SessionLockState {
    fn new() -> Self {
        Self {
            manager: None,
            lock: None,
            lock_state: None,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for SessionLockState {
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
            && interface == "ext_session_lock_manager_v1"
            && state.manager.is_none()
        {
            let mgr = registry
                .bind::<ext_session_lock_manager_v1::ExtSessionLockManagerV1, (), Self>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for SessionLockState {
    fn event(
        _state: &mut Self,
        _proxy: &ext_session_lock_manager_v1::ExtSessionLockManagerV1,
        _event: ext_session_lock_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for SessionLockState {
    fn event(
        state: &mut Self,
        _proxy: &ext_session_lock_v1::ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_session_lock_v1::Event::Locked => {
                state.lock_state = Some(LockState::Locked);
            }
            ext_session_lock_v1::Event::Finished => {
                state.lock_state = Some(LockState::Finished);
                state.lock = None;
            }
            _ => {}
        }
    }
}
