use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;

use wayland_client::{EventQueue, QueueHandle};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

use super::state::DataControlState;
use super::{ClipboardOffer, MimeType, Selection};

/// Client for the `zwlr_data_control_manager_v1` protocol.
///
/// Read and write clipboard and primary selection content.
///
/// # Example
///
/// ```no_run
/// use hyprs_sdk::protocols::connection::WaylandConnection;
/// use hyprs_sdk::protocols::data_control::{DataControlClient, Selection};
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
    qh: QueueHandle<DataControlState>,
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

        // Wayland events arrive asynchronously; roundtrip ensures the manager and
        // seat globals are bound before we create the data device.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let manager = state.manager.clone().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_data_control_manager_v1".into())
        })?;
        let seat = state
            .seat
            .clone()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_seat available".into()))?;

        let device = manager.get_data_device(&seat, &qh, ());
        state.device = Some(device);

        // The device sends data_offer + selection events asynchronously after creation;
        // roundtrip ensures we have the initial clipboard state.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // MIME type events arrive on the offer created in the previous roundtrip;
        // an extra roundtrip collects them so we know all available types.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Get the current clipboard/selection offer (available MIME types).
    #[must_use]
    pub fn offer(&self, selection: Selection) -> Option<ClipboardOffer> {
        let offer = self.state.offer_for(selection)?;
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
        // Dispatch pending events to ensure we read the most current offer.
        self.roundtrip()?;

        let offer_proxy = {
            let offer = self
                .state
                .offer_for(selection)
                .ok_or_else(|| HyprError::WaylandDispatch("no clipboard offer available".into()))?;

            if !offer.mime_types.iter().any(|m| m == mime_type) {
                return Err(HyprError::WaylandDispatch(format!(
                    "MIME type not offered: {mime_type}"
                )));
            }

            offer.proxy.clone()
        };

        // The protocol transfers clipboard data through a Unix socket pair: the
        // compositor writes to one end, we read from the other.
        let (mut read_end, write_end) =
            UnixStream::pair().map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        offer_proxy.receive(mime_type.to_string(), write_end.as_fd());

        // Roundtrip ensures the receive request reaches the compositor before we
        // try to read from the socket.
        self.roundtrip()?;

        // Close the write end so the read side receives EOF once the compositor
        // finishes writing the clipboard data.
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
            let offer = self.state.offer_for(selection);
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

    /// Write a single MIME payload to clipboard or primary selection.
    ///
    /// This replaces the current selection with a new source owned by this
    /// client. The source remains alive until the compositor sends
    /// `cancelled`.
    ///
    /// # Errors
    ///
    /// Returns an error if the data device is unavailable, `mime_type` is
    /// empty, or event dispatch fails.
    pub fn write(&mut self, selection: Selection, mime_type: &str, data: &[u8]) -> HyprResult<()> {
        if mime_type.is_empty() {
            return Err(HyprError::WaylandDispatch(
                "mime_type cannot be empty".into(),
            ));
        }

        let mut payloads = HashMap::new();
        payloads.insert(mime_type.to_string(), data.to_vec());
        self.write_payloads(selection, payloads)
    }

    /// Write text data to clipboard or primary selection.
    ///
    /// Offers common plain-text MIME variants for better interoperability
    /// across toolkits.
    ///
    /// # Errors
    ///
    /// Returns an error if the data device is unavailable or dispatch fails.
    pub fn write_text(&mut self, selection: Selection, text: &str) -> HyprResult<()> {
        let mut payloads = HashMap::new();
        let bytes = text.as_bytes().to_vec();

        for mime in [
            MimeType::TEXT_PLAIN_UTF8,
            MimeType::TEXT_PLAIN,
            "UTF8_STRING",
            "STRING",
        ] {
            payloads.insert(mime.to_string(), bytes.clone());
        }

        self.write_payloads(selection, payloads)
    }

    /// Clear clipboard or primary selection.
    ///
    /// # Errors
    ///
    /// Returns an error if the data device is unavailable or dispatch fails.
    pub fn clear(&mut self, selection: Selection) -> HyprResult<()> {
        let device = self
            .state
            .device
            .clone()
            .ok_or_else(|| HyprError::WaylandDispatch("no data device available".into()))?;

        self.state.clear_selection_source(selection, &device);
        self.roundtrip()
    }

    /// Re-dispatch events to update clipboard state.
    pub fn refresh(&mut self) -> HyprResult<()> {
        self.roundtrip()
    }

    fn write_payloads(
        &mut self,
        selection: Selection,
        payloads: HashMap<String, Vec<u8>>,
    ) -> HyprResult<()> {
        if payloads.is_empty() {
            return Err(HyprError::WaylandDispatch(
                "at least one payload is required".into(),
            ));
        }

        let manager = self.state.manager.clone().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_data_control_manager_v1".into())
        })?;
        let device = self
            .state
            .device
            .clone()
            .ok_or_else(|| HyprError::WaylandDispatch("no data device available".into()))?;

        let source = manager.create_data_source(&self.qh, ());
        for mime in payloads.keys() {
            source.offer(mime.clone());
        }

        self.state
            .set_selection_source(selection, &device, source, payloads);
        self.roundtrip()
    }

    fn roundtrip(&mut self) -> HyprResult<()> {
        self.event_queue
            .roundtrip(&mut self.state)
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
