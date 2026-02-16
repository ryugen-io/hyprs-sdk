//! virtual-keyboard: synthetic keyboard input.
//!
//! Provides [`VirtualKeyboardClient`] for creating virtual keyboard devices
//! and sending synthetic key events via the `zwp_virtual_keyboard_manager_v1`
//! protocol.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::virtual_keyboard::{VirtualKeyboardClient, KeyState};
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = VirtualKeyboardClient::connect(&wl).unwrap();
//!
//! // Set a keymap first (required before sending events)
//! let keymap = std::fs::read("/path/to/keymap").unwrap();
//! client.set_keymap(&keymap).unwrap();
//!
//! // Send a key press (key 28 = Enter on most keymaps)
//! client.key(0, 28, KeyState::Pressed).unwrap();
//! client.key(10, 28, KeyState::Released).unwrap();
//! ```

use std::fmt;
use std::io::Write;
use std::os::unix::io::AsFd;

use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Key state for virtual keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyState {
    /// Key is released.
    Released = 0,
    /// Key is pressed.
    Pressed = 1,
}

/// A virtual key event.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Time in milliseconds.
    pub time: u32,
    /// Evdev keycode.
    pub key: u32,
    /// Key state.
    pub state: KeyState,
}

/// Modifier state for the virtual keyboard.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    /// Depressed modifiers (currently held keys).
    pub mods_depressed: u32,
    /// Latched modifiers (toggled on for next key).
    pub mods_latched: u32,
    /// Locked modifiers (e.g. Caps Lock).
    pub mods_locked: u32,
    /// Active keyboard group/layout.
    pub group: u32,
}

/// Client for the `zwp_virtual_keyboard_manager_v1` protocol.
///
/// Creates virtual keyboards that send synthetic key events to the
/// compositor. A keymap must be set before sending key events.
pub struct VirtualKeyboardClient {
    state: VirtualKeyboardState,
    event_queue: EventQueue<VirtualKeyboardState>,
}

impl VirtualKeyboardClient {
    /// Connect to the virtual keyboard manager and create a virtual keyboard.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwp_virtual_keyboard_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwp_virtual_keyboard_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwp_virtual_keyboard_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<VirtualKeyboardState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = VirtualKeyboardState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwp_virtual_keyboard_manager_v1".into(),
            ));
        }

        let seat = state
            .seat
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_seat available".into()))?;
        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwp_virtual_keyboard_manager_v1".into())
        })?;

        let keyboard = manager.create_virtual_keyboard(seat, &qh, ());
        state.keyboard = Some(keyboard);

        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// Set the keymap for the virtual keyboard.
    ///
    /// The keymap must be in XKB format. This must be called before
    /// sending any key events.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyboard is unavailable or the temp file
    /// cannot be created.
    pub fn set_keymap(&mut self, keymap_data: &[u8]) -> HyprResult<()> {
        let keyboard = self
            .state
            .keyboard
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no virtual keyboard created".into()))?;

        // Write keymap to a temp file and pass the fd.
        let mut tmpfile = tempfile().map_err(|e| {
            HyprError::WaylandDispatch(format!("failed to create keymap temp file: {e}"))
        })?;
        tmpfile
            .write_all(keymap_data)
            .map_err(|e| HyprError::WaylandDispatch(format!("failed to write keymap: {e}")))?;

        // XKB_KEYMAP_FORMAT_TEXT_V1 = 1
        keyboard.keymap(1, tmpfile.as_fd(), keymap_data.len() as u32);

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Send a key event.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyboard is unavailable or dispatch fails.
    pub fn key(&mut self, time: u32, keycode: u32, key_state: KeyState) -> HyprResult<()> {
        let keyboard = self
            .state
            .keyboard
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no virtual keyboard created".into()))?;

        keyboard.key(time, keycode, key_state as u32);

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Update modifier state.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyboard is unavailable or dispatch fails.
    pub fn modifiers(&mut self, mods: &ModifierState) -> HyprResult<()> {
        let keyboard = self
            .state
            .keyboard
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no virtual keyboard created".into()))?;

        keyboard.modifiers(
            mods.mods_depressed,
            mods.mods_latched,
            mods.mods_locked,
            mods.group,
        );

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }
}

impl fmt::Debug for VirtualKeyboardClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VirtualKeyboardClient")
            .field("has_keyboard", &self.state.keyboard.is_some())
            .finish()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Create a temporary file in `$XDG_RUNTIME_DIR`.
fn tempfile() -> std::io::Result<std::fs::File> {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = format!("{dir}/hypr-sdk-keymap-XXXXXX");
    let file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    // Remove the file so only the fd remains.
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

// ── Internal state ───────────────────────────────────────────────────

struct VirtualKeyboardState {
    manager: Option<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    keyboard: Option<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1>,
}

impl VirtualKeyboardState {
    fn new() -> Self {
        Self {
            manager: None,
            seat: None,
            keyboard: None,
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for VirtualKeyboardState {
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
                "zwp_virtual_keyboard_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, (), Self>(
                            name,
                            version.min(1),
                            qh,
                            (),
                        );
                    state.manager = Some(mgr);
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

impl Dispatch<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, ()>
    for VirtualKeyboardState
{
    fn event(
        _state: &mut Self,
        _proxy: &zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
        _event: zwp_virtual_keyboard_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Manager has no events.
    }
}

impl Dispatch<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1, ()> for VirtualKeyboardState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
        _event: zwp_virtual_keyboard_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Virtual keyboard has no events.
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for VirtualKeyboardState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Seat events not needed for virtual keyboard.
    }
}
