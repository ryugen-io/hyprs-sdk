//! wlr-data-control: clipboard access.
//!
//! Read and write clipboard and primary selection content.

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
