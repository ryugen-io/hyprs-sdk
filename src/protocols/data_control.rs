//! wlr-data-control: clipboard access.
//!
//! Provides [`DataControlClient`] for reading and writing clipboard
//! and primary selection content via the `zwlr_data_control_manager_v1`
//! protocol.

use std::fmt;
use std::io::Read;
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;

use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, event_created_child};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1, zwlr_data_control_manager_v1, zwlr_data_control_offer_v1,
    zwlr_data_control_source_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// MIME type for clipboard content.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MimeType(pub String);

impl MimeType {
    /// Standard plain text MIME type.
    pub const TEXT_PLAIN: &str = "text/plain";
    /// Plain text with UTF-8 charset.
    pub const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";
    /// URI list MIME type.
    pub const TEXT_URI_LIST: &str = "text/uri-list";
    /// PNG image MIME type.
    pub const IMAGE_PNG: &str = "image/png";

    /// Create from a MIME type string.
    #[must_use]
    pub fn new(mime: impl Into<String>) -> Self {
        Self(mime.into())
    }

    /// Check if this is a text MIME type.
    #[must_use]
    pub fn is_text(&self) -> bool {
        self.0.starts_with("text/") || self.0 == "STRING" || self.0 == "UTF8_STRING"
    }

    /// The raw MIME type string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A clipboard offer with available MIME types.
#[derive(Debug, Clone, Default)]
pub struct ClipboardOffer {
    /// Available MIME types for this offer.
    pub mime_types: Vec<MimeType>,
}

impl ClipboardOffer {
    /// Check if text content is available.
    #[must_use]
    pub fn has_text(&self) -> bool {
        self.mime_types.iter().any(|m| m.is_text())
    }

    /// Find the best text MIME type, preferring UTF-8.
    #[must_use]
    pub fn best_text_mime(&self) -> Option<&MimeType> {
        self.mime_types
            .iter()
            .find(|m| m.0 == MimeType::TEXT_PLAIN_UTF8)
            .or_else(|| self.mime_types.iter().find(|m| m.0 == MimeType::TEXT_PLAIN))
            .or_else(|| self.mime_types.iter().find(|m| m.is_text()))
    }
}

/// Which selection to operate on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Selection {
    /// The regular clipboard (Ctrl+C/V).
    Clipboard,
    /// The primary selection (middle-click paste on X11/Wayland).
    Primary,
}

/// Client for the `zwlr_data_control_manager_v1` protocol.
///
/// Read and write clipboard and primary selection content.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::data_control::{DataControlClient, Selection};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = DataControlClient::connect(&wl).unwrap();
///
/// // Read clipboard text
/// if let Some(text) = client.read_text(Selection::Clipboard).unwrap() {
///     println!("Clipboard: {text}");
/// }
/// ```
pub struct DataControlClient {
    state: DataControlState,
    event_queue: EventQueue<DataControlState>,
}

impl DataControlClient {
    /// Connect to the data control manager.
    ///
    /// Binds `zwlr_data_control_manager_v1`, creates a data device on
    /// the default seat, and waits for the initial clipboard state.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_data_control_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_data_control_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_data_control_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<DataControlState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = DataControlState::new();

        // Registry roundtrip: bind manager + seat.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_data_control_manager_v1".into())
        })?;
        let seat = state
            .seat
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_seat available".into()))?;

        let device = manager.get_data_device(seat, &qh, ());
        state.device = Some(device);

        // Roundtrip to receive initial data_offer + selection events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Extra roundtrip for offer MIME type events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// Get the current clipboard/selection offer (available MIME types).
    #[must_use]
    pub fn offer(&self, selection: Selection) -> Option<ClipboardOffer> {
        let offer = match selection {
            Selection::Clipboard => self.state.clipboard_offer.as_ref(),
            Selection::Primary => self.state.primary_offer.as_ref(),
        }?;
        Some(ClipboardOffer {
            mime_types: offer.mime_types.iter().map(MimeType::new).collect(),
        })
    }

    /// Read data from the clipboard/selection for a specific MIME type.
    ///
    /// # Errors
    ///
    /// Returns an error if no offer is available, the MIME type isn't
    /// offered, or reading the data fails.
    pub fn read(&mut self, selection: Selection, mime_type: &str) -> HyprResult<Vec<u8>> {
        // Refresh to get latest state.
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let offer = match selection {
            Selection::Clipboard => state.clipboard_offer.as_ref(),
            Selection::Primary => state.primary_offer.as_ref(),
        }
        .ok_or_else(|| HyprError::WaylandDispatch("no clipboard offer available".into()))?;

        if !offer.mime_types.iter().any(|m| m == mime_type) {
            return Err(HyprError::WaylandDispatch(format!(
                "MIME type not offered: {mime_type}"
            )));
        }

        // Create a socket pair for data transfer.
        let (mut read_end, write_end) =
            UnixStream::pair().map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        offer
            .proxy
            .receive(mime_type.to_string(), write_end.as_fd());

        // Roundtrip to flush the receive request.
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Close write end so read gets EOF when compositor finishes.
        drop(write_end);

        let mut data = Vec::new();
        read_end
            .read_to_end(&mut data)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(data)
    }

    /// Read text from the clipboard/selection.
    ///
    /// Automatically selects the best text MIME type (preferring UTF-8).
    ///
    /// # Errors
    ///
    /// Returns an error if no text is available or reading fails.
    pub fn read_text(&mut self, selection: Selection) -> HyprResult<Option<String>> {
        let mime = {
            let offer = match selection {
                Selection::Clipboard => self.state.clipboard_offer.as_ref(),
                Selection::Primary => self.state.primary_offer.as_ref(),
            };
            match offer {
                Some(o) => {
                    let co = ClipboardOffer {
                        mime_types: o.mime_types.iter().map(MimeType::new).collect(),
                    };
                    co.best_text_mime().map(|m| m.0.clone())
                }
                None => return Ok(None),
            }
        };

        match mime {
            Some(m) => {
                let data = self.read(selection, &m)?;
                Ok(Some(String::from_utf8_lossy(&data).into_owned()))
            }
            None => Ok(None),
        }
    }

    /// Re-dispatch events to update clipboard state.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for DataControlClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataControlClient")
            .field("has_clipboard", &self.state.clipboard_offer.is_some())
            .field("has_primary", &self.state.primary_offer.is_some())
            .finish()
    }
}

// ── Internal state ───────────────────────────────────────────────────

struct DataControlState {
    manager: Option<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    device: Option<zwlr_data_control_device_v1::ZwlrDataControlDeviceV1>,
    /// Most recently introduced offer (accumulating MIME types).
    pending_offer: Option<OfferEntry>,
    /// Current clipboard selection offer.
    clipboard_offer: Option<OfferEntry>,
    /// Current primary selection offer.
    primary_offer: Option<OfferEntry>,
}

struct OfferEntry {
    proxy: zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
    mime_types: Vec<String>,
}

impl DataControlState {
    fn new() -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            pending_offer: None,
            clipboard_offer: None,
            primary_offer: None,
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for DataControlState {
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
                "zwlr_data_control_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, (), Self>(
                            name,
                            version.min(2),
                            qh,
                            (),
                        );
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

impl Dispatch<wl_seat::WlSeat, ()> for DataControlState {
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

impl Dispatch<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, ()> for DataControlState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
        _event: zwlr_data_control_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Manager has no events.
    }
}

impl Dispatch<zwlr_data_control_device_v1::ZwlrDataControlDeviceV1, ()> for DataControlState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
        event: zwlr_data_control_device_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::DataOffer { id } => {
                // New offer introduced — start accumulating MIME types.
                state.pending_offer = Some(OfferEntry {
                    proxy: id,
                    mime_types: Vec::new(),
                });
            }
            zwlr_data_control_device_v1::Event::Selection { id } => {
                // Clipboard selection changed.
                if id.is_some() {
                    state.clipboard_offer = state.pending_offer.take();
                } else {
                    state.clipboard_offer = None;
                }
            }
            zwlr_data_control_device_v1::Event::PrimarySelection { id } => {
                if id.is_some() {
                    state.primary_offer = state.pending_offer.take();
                } else {
                    state.primary_offer = None;
                }
            }
            zwlr_data_control_device_v1::Event::Finished => {
                state.device = None;
            }
            _ => {}
        }
    }

    event_created_child!(DataControlState, zwlr_data_control_device_v1::ZwlrDataControlDeviceV1, [
        // Opcode 0 = data_offer event creates a new offer object.
        0 => (zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, ()),
    ]);
}

impl Dispatch<zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, ()> for DataControlState {
    fn event(
        state: &mut Self,
        proxy: &zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
        event: zwlr_data_control_offer_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
            // Add MIME type to the pending offer.
            if let Some(ref mut offer) = state.pending_offer
                && offer.proxy == *proxy
            {
                offer.mime_types.push(mime_type);
                return;
            }
            // Also check active offers in case of late events.
            if let Some(ref mut offer) = state.clipboard_offer
                && offer.proxy == *proxy
            {
                offer.mime_types.push(mime_type);
                return;
            }
            if let Some(ref mut offer) = state.primary_offer
                && offer.proxy == *proxy
            {
                offer.mime_types.push(mime_type);
            }
        }
    }
}

impl Dispatch<zwlr_data_control_source_v1::ZwlrDataControlSourceV1, ()> for DataControlState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
        _event: zwlr_data_control_source_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Source events (send, cancelled) handled if/when we implement write.
    }
}
