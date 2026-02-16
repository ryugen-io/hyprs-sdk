//! ext-idle-notify: idle detection for user input.
//!
//! Provides [`IdleClient`] for monitoring user idle state via the
//! `ext_idle_notifier_v1` protocol.

use std::fmt;
use std::time::Duration;

use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Configuration for an idle notification.
#[derive(Debug, Clone)]
pub struct IdleNotificationConfig {
    /// How long the user must be idle before notification fires.
    pub timeout: Duration,
}

impl IdleNotificationConfig {
    /// Create with timeout in seconds.
    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(secs),
        }
    }

    /// Create with timeout in milliseconds.
    #[must_use]
    pub fn from_millis(millis: u64) -> Self {
        Self {
            timeout: Duration::from_millis(millis),
        }
    }

    /// Timeout in milliseconds (for the Wayland protocol).
    #[must_use]
    pub fn timeout_ms(&self) -> u32 {
        self.timeout.as_millis() as u32
    }
}

/// Current idle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdleState {
    /// User is active (has interacted recently).
    Active,
    /// User is idle (no interaction for the configured timeout).
    Idle,
}

/// Client for the `ext_idle_notifier_v1` protocol.
///
/// Monitors user idle state with a configurable timeout.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::idle::{IdleClient, IdleNotificationConfig};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = IdleClient::connect(&wl, IdleNotificationConfig::from_secs(300)).unwrap();
///
/// // Poll idle state
/// let state = client.poll().unwrap();
/// println!("User is {:?}", state);
/// ```
pub struct IdleClient {
    state: IdleNotifyState,
    event_queue: EventQueue<IdleNotifyState>,
}

impl IdleClient {
    /// Connect and register an idle notification.
    ///
    /// Creates a notification that fires after the configured timeout
    /// with no user input.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `ext_idle_notifier_v1`.
    pub fn connect(wl: &WaylandConnection, config: IdleNotificationConfig) -> HyprResult<Self> {
        if !wl.has_protocol("ext_idle_notifier_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "ext_idle_notifier_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<IdleNotifyState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = IdleNotifyState::new();

        // Registry roundtrip: bind notifier + seat.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let notifier = state
            .notifier
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("ext_idle_notifier_v1".into()))?;
        let seat = state
            .seat
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_seat available".into()))?;

        let _notification = notifier.get_idle_notification(config.timeout_ms(), seat, &qh, ());

        // Roundtrip to start receiving idle/resumed events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// Get the current idle state.
    #[must_use]
    pub fn idle_state(&self) -> IdleState {
        self.state.idle_state
    }

    /// Poll for idle state changes.
    ///
    /// Dispatches pending Wayland events and returns the current state.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn poll(&mut self) -> HyprResult<IdleState> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(state.idle_state)
    }
}

impl fmt::Debug for IdleClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdleClient")
            .field("idle_state", &self.state.idle_state)
            .finish()
    }
}

// ── Internal state ───────────────────────────────────────────────────

struct IdleNotifyState {
    notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    seat: Option<wl_seat::WlSeat>,
    idle_state: IdleState,
}

impl IdleNotifyState {
    fn new() -> Self {
        Self {
            notifier: None,
            seat: None,
            idle_state: IdleState::Active,
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for IdleNotifyState {
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
                "ext_idle_notifier_v1" if state.notifier.is_none() => {
                    let n = registry.bind::<ext_idle_notifier_v1::ExtIdleNotifierV1, (), Self>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    );
                    state.notifier = Some(n);
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

impl Dispatch<wl_seat::WlSeat, ()> for IdleNotifyState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Seat events not needed.
    }
}

impl Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for IdleNotifyState {
    fn event(
        _state: &mut Self,
        _proxy: &ext_idle_notifier_v1::ExtIdleNotifierV1,
        _event: ext_idle_notifier_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Notifier has no events.
    }
}

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for IdleNotifyState {
    fn event(
        state: &mut Self,
        _proxy: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                state.idle_state = IdleState::Idle;
            }
            ext_idle_notification_v1::Event::Resumed => {
                state.idle_state = IdleState::Active;
            }
            _ => {}
        }
    }
}
