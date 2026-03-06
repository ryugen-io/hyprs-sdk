use std::borrow::Cow;

use crate::error::{HyprError, HyprResult};

pub(crate) fn normalize_window_selector(window: &str) -> Cow<'_, str> {
    if window.starts_with("0x") {
        Cow::Owned(format!("address:{window}"))
    } else {
        Cow::Borrowed(window)
    }
}

pub(crate) fn parse_json_or_command_error(raw: String) -> HyprResult<serde_json::Value> {
    match serde_json::from_str(&raw) {
        Ok(value) => Ok(value),
        Err(err) => {
            let trimmed = raw.trim();
            if looks_like_json(trimmed) {
                Err(HyprError::Json(err))
            } else {
                Err(HyprError::Command(trimmed.to_string()))
            }
        }
    }
}

fn looks_like_json(raw: &str) -> bool {
    raw.starts_with('{')
        || raw.starts_with('[')
        || raw.starts_with('"')
        || matches!(raw, "true" | "false" | "null")
        || raw.parse::<f64>().is_ok()
}
