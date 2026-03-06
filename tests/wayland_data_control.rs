#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::data_control::*;

#[test]
fn mime_type_is_text() {
    assert!(MimeType::new("text/plain").is_text());
    assert!(MimeType::new("text/plain;charset=utf-8").is_text());
    assert!(MimeType::new("text/html").is_text());
    assert!(MimeType::new("UTF8_STRING").is_text());
    assert!(!MimeType::new("image/png").is_text());
}

#[test]
fn clipboard_offer_has_text() {
    let offer = ClipboardOffer {
        mime_types: vec![MimeType::new("image/png"), MimeType::new("text/plain")],
    };
    assert!(offer.has_text());
}

#[test]
fn clipboard_offer_no_text() {
    let offer = ClipboardOffer {
        mime_types: vec![MimeType::new("image/png")],
    };
    assert!(!offer.has_text());
}

#[test]
fn clipboard_offer_best_text_prefers_utf8() {
    let offer = ClipboardOffer {
        mime_types: vec![
            MimeType::new("text/plain"),
            MimeType::new("text/plain;charset=utf-8"),
        ],
    };
    assert_eq!(
        offer.best_text_mime().unwrap().as_str(),
        "text/plain;charset=utf-8"
    );
}

#[test]
fn selection_variants() {
    assert_ne!(Selection::Clipboard, Selection::Primary);
}
