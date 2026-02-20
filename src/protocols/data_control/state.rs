use std::collections::HashMap;

use wayland_client::protocol::wl_seat;
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1, zwlr_data_control_manager_v1, zwlr_data_control_offer_v1,
    zwlr_data_control_source_v1,
};

use super::Selection;

pub(super) struct DataControlState {
    pub(super) manager: Option<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1>,
    pub(super) seat: Option<wl_seat::WlSeat>,
    pub(super) device: Option<zwlr_data_control_device_v1::ZwlrDataControlDeviceV1>,
    /// Most recently introduced offer (accumulating MIME types).
    pub(super) pending_offer: Option<OfferEntry>,
    /// Current clipboard selection offer.
    pub(super) clipboard_offer: Option<OfferEntry>,
    /// Current primary selection offer.
    pub(super) primary_offer: Option<OfferEntry>,
    /// Active and recently-used source objects for outbound clipboard data.
    ///
    /// Entries are removed when the compositor sends `cancelled`.
    pub(super) sources: Vec<SourceEntry>,
    /// Currently active source for clipboard selection.
    pub(super) active_clipboard_source:
        Option<zwlr_data_control_source_v1::ZwlrDataControlSourceV1>,
    /// Currently active source for primary selection.
    pub(super) active_primary_source: Option<zwlr_data_control_source_v1::ZwlrDataControlSourceV1>,
}

pub(super) struct OfferEntry {
    pub(super) proxy: zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
    pub(super) mime_types: Vec<String>,
}

pub(super) struct SourceEntry {
    pub(super) proxy: zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
    pub(super) payloads: HashMap<String, Vec<u8>>,
}

impl DataControlState {
    pub(super) fn new() -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            pending_offer: None,
            clipboard_offer: None,
            primary_offer: None,
            sources: Vec::new(),
            active_clipboard_source: None,
            active_primary_source: None,
        }
    }

    pub(super) fn offer_for(&self, selection: Selection) -> Option<&OfferEntry> {
        match selection {
            Selection::Clipboard => self.clipboard_offer.as_ref(),
            Selection::Primary => self.primary_offer.as_ref(),
        }
    }

    pub(super) fn set_selection_source(
        &mut self,
        selection: Selection,
        device: &zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
        source: zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
        payloads: HashMap<String, Vec<u8>>,
    ) {
        match selection {
            Selection::Clipboard => {
                device.set_selection(Some(&source));
                self.active_clipboard_source = Some(source.clone());
            }
            Selection::Primary => {
                device.set_primary_selection(Some(&source));
                self.active_primary_source = Some(source.clone());
            }
        }

        self.sources.push(SourceEntry {
            proxy: source,
            payloads,
        });
    }

    pub(super) fn clear_selection_source(
        &mut self,
        selection: Selection,
        device: &zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
    ) {
        match selection {
            Selection::Clipboard => {
                device.set_selection(None);
                self.active_clipboard_source = None;
            }
            Selection::Primary => {
                device.set_primary_selection(None);
                self.active_primary_source = None;
            }
        }
    }
}
