//! Hyprland configuration types.
//!
//! Config option types, monitor rules, workspace rules, window rules, layer rules.

mod monitor_rule;
mod option;
mod rules;
mod workspace_rule;

pub use monitor_rule::*;
pub use option::*;
pub use rules::*;
pub use workspace_rule::*;
