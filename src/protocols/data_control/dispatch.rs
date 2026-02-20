use std::io::Write;

use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle, event_created_child};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1, zwlr_data_control_manager_v1, zwlr_data_control_offer_v1,
    zwlr_data_control_source_v1,
};

use super::state::{DataControlState, OfferEntry};

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
        state: &mut Self,
        proxy: &zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
        event: zwlr_data_control_source_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_source_v1::Event::Send { mime_type, fd } => {
                if let Some(source) = state.sources.iter().find(|s| s.proxy == *proxy)
                    && let Some(payload) = source.payloads.get(&mime_type)
                {
                    let mut file = std::fs::File::from(fd);
                    let _ = file.write_all(payload);
                    let _ = file.flush();
                }
            }
            zwlr_data_control_source_v1::Event::Cancelled => {
                if state
                    .active_clipboard_source
                    .as_ref()
                    .is_some_and(|s| *s == *proxy)
                {
                    state.active_clipboard_source = None;
                }
                if state
                    .active_primary_source
                    .as_ref()
                    .is_some_and(|s| *s == *proxy)
                {
                    state.active_primary_source = None;
                }

                if let Some(idx) = state.sources.iter().position(|s| s.proxy == *proxy) {
                    let source = state.sources.swap_remove(idx);
                    source.proxy.destroy();
                } else {
                    proxy.destroy();
                }
            }
            _ => {}
        }
    }
}
