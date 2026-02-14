//! Layout control dispatchers.

use super::DispatchCmd;

/// Toggle pseudo-tiling on a window.
#[must_use]
pub fn pseudo(regex: &str) -> DispatchCmd {
    DispatchCmd {
        name: "pseudo",
        args: regex.to_string(),
    }
}

/// Toggle split layout orientation.
#[must_use]
pub fn toggle_split() -> DispatchCmd {
    DispatchCmd::no_args("togglesplit")
}

/// Swap split direction.
#[must_use]
pub fn swap_split() -> DispatchCmd {
    DispatchCmd::no_args("swapsplit")
}

/// Adjust split ratio between windows.
///
/// Prefix with `+`/`-` for relative, append `exact` for absolute.
#[must_use]
pub fn split_ratio(ratio: &str) -> DispatchCmd {
    DispatchCmd {
        name: "splitratio",
        args: ratio.to_string(),
    }
}

/// Send a message to the layout engine.
#[must_use]
pub fn layout_msg(message: &str) -> DispatchCmd {
    DispatchCmd {
        name: "layoutmsg",
        args: message.to_string(),
    }
}
