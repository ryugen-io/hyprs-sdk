//! wlr-virtual-pointer: synthetic mouse/pointer input.
//!
//! Provides [`VirtualPointerClient`] for creating virtual pointer devices
//! and sending synthetic mouse events via the
//! `zwlr_virtual_pointer_manager_v1` protocol.

use std::fmt;

use wayland_client::protocol::{wl_pointer, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Virtual pointer button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ButtonState {
    /// Button is released.
    Released = 0,
    /// Button is pressed.
    Pressed = 1,
}

/// Axis source for scroll events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AxisSource {
    /// Physical scroll wheel.
    Wheel = 0,
    /// Finger on a touchpad.
    Finger = 1,
    /// Continuous scroll source.
    Continuous = 2,
    /// Tilted scroll wheel.
    WheelTilt = 3,
}

/// Scroll axis direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Axis {
    /// Vertical scroll axis.
    Vertical = 0,
    /// Horizontal scroll axis.
    Horizontal = 1,
}

/// A virtual pointer motion event.
#[derive(Debug, Clone, Copy)]
pub struct MotionEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Relative X displacement.
    pub dx: f64,
    /// Relative Y displacement.
    pub dy: f64,
}

/// An absolute motion event.
#[derive(Debug, Clone, Copy)]
pub struct MotionAbsoluteEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Absolute X position (0.0 to 1.0, normalized).
    pub x: f64,
    /// Absolute Y position (0.0 to 1.0, normalized).
    pub y: f64,
    /// Bounding box width.
    pub x_extent: u32,
    /// Bounding box height.
    pub y_extent: u32,
}

/// A button event.
#[derive(Debug, Clone, Copy)]
pub struct ButtonEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Linux input event code (e.g. BTN_LEFT = 0x110).
    pub button: u32,
    /// Button state.
    pub state: ButtonState,
}

/// Client for the `zwlr_virtual_pointer_manager_v1` protocol.
///
/// Creates virtual pointer devices and sends synthetic mouse events
/// (motion, button, scroll) to the compositor.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::virtual_pointer::{VirtualPointerClient, ButtonState};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = VirtualPointerClient::connect(&wl).unwrap();
///
/// // Move cursor 100px right, 50px down
/// client.motion(0, 100.0, 50.0);
/// client.frame();
///
/// // Click left mouse button (BTN_LEFT = 0x110)
/// client.button(0, 0x110, ButtonState::Pressed);
/// client.frame();
/// client.button(0, 0x110, ButtonState::Released);
/// client.frame();
/// ```
pub struct VirtualPointerClient {
    pointer: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
    _state: VirtualPointerState,
    event_queue: EventQueue<VirtualPointerState>,
}

impl VirtualPointerClient {
    /// Connect and create a virtual pointer device.
    ///
    /// Binds `zwlr_virtual_pointer_manager_v1` and creates a virtual
    /// pointer on the default seat.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_virtual_pointer_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_virtual_pointer_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_virtual_pointer_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<VirtualPointerState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = VirtualPointerState::new();

        // Wayland events arrive asynchronously; roundtrip ensures manager and seat
        // globals are bound before we attempt to create the virtual pointer.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_virtual_pointer_manager_v1".into())
        })?;

        // The protocol allows a seatless pointer for compositors that support it;
        // prefer the seat when available for proper input routing.
        let pointer = if let Some(ref seat) = state.seat {
            manager.create_virtual_pointer(Some(seat), &qh, ())
        } else {
            manager.create_virtual_pointer(None, &qh, ())
        };

        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            pointer,
            _state: state,
            event_queue,
        })
    }

    /// Send a relative motion event.
    pub fn motion(&self, time: u32, dx: f64, dy: f64) {
        self.pointer.motion(time, dx, dy);
    }

    /// Send an absolute motion event.
    ///
    /// Coordinates are in the range `[0, x_extent]` / `[0, y_extent]`.
    pub fn motion_absolute(&self, time: u32, x: u32, y: u32, x_extent: u32, y_extent: u32) {
        self.pointer.motion_absolute(time, x, y, x_extent, y_extent);
    }

    /// Send a button event.
    pub fn button(&self, time: u32, button: u32, state: ButtonState) {
        let wl_state = match state {
            ButtonState::Released => wl_pointer::ButtonState::Released,
            ButtonState::Pressed => wl_pointer::ButtonState::Pressed,
        };
        self.pointer.button(time, button, wl_state);
    }

    /// Send a scroll axis event.
    pub fn axis(&self, time: u32, axis: Axis, value: f64) {
        self.pointer.axis(time, to_wl_axis(axis), value);
    }

    /// Set the axis source for subsequent scroll events.
    pub fn axis_source(&self, source: AxisSource) {
        let wl_source = match source {
            AxisSource::Wheel => wl_pointer::AxisSource::Wheel,
            AxisSource::Finger => wl_pointer::AxisSource::Finger,
            AxisSource::Continuous => wl_pointer::AxisSource::Continuous,
            AxisSource::WheelTilt => wl_pointer::AxisSource::WheelTilt,
        };
        self.pointer.axis_source(wl_source);
    }

    /// Stop an axis (end of scroll gesture).
    pub fn axis_stop(&self, time: u32, axis: Axis) {
        self.pointer.axis_stop(time, to_wl_axis(axis));
    }

    /// Send a discrete axis event (wheel clicks).
    pub fn axis_discrete(&self, time: u32, axis: Axis, value: f64, discrete: i32) {
        self.pointer
            .axis_discrete(time, to_wl_axis(axis), value, discrete);
    }

    /// End the current event frame.
    ///
    /// Call this after sending one or more events to commit them
    /// as an atomic input frame.
    pub fn frame(&self) {
        self.pointer.frame();
    }

    /// Flush pending events to the compositor.
    pub fn flush(&mut self) -> HyprResult<()> {
        self.event_queue
            .roundtrip(&mut self._state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for VirtualPointerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VirtualPointerClient").finish()
    }
}

fn to_wl_axis(axis: Axis) -> wl_pointer::Axis {
    match axis {
        Axis::Vertical => wl_pointer::Axis::VerticalScroll,
        Axis::Horizontal => wl_pointer::Axis::HorizontalScroll,
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Holds protocol objects discovered during the registry roundtrip.

struct VirtualPointerState {
    manager: Option<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1>,
    seat: Option<wl_seat::WlSeat>,
}

impl VirtualPointerState {
    fn new() -> Self {
        Self {
            manager: None,
            seat: None,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue, even for objects that emit no events we care about.

impl Dispatch<wl_registry::WlRegistry, ()> for VirtualPointerState {
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
                "zwlr_virtual_pointer_manager_v1" if state.manager.is_none() => {
                    let mgr = registry.bind::<
                        zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
                        (),
                        Self,
                    >(name, version.min(2), qh, ());
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

impl Dispatch<wl_seat::WlSeat, ()> for VirtualPointerState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We only need the seat proxy for pointer creation; its events are irrelevant.
    }
}

impl Dispatch<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, ()>
    for VirtualPointerState
{
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
        _event: zwlr_virtual_pointer_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1, ()> for VirtualPointerState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        _event: zwlr_virtual_pointer_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; virtual pointer is request-only.
    }
}
