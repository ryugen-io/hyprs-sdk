#![cfg(feature = "wayland")]
use hypr_sdk::protocols::foreign_toplevel::*;

#[test]
fn toplevel_state_flags() {
    let state = ToplevelState::MAXIMIZED | ToplevelState::ACTIVATED;
    assert!(state.is_maximized());
    assert!(state.is_activated());
    assert!(!state.is_minimized());
    assert!(!state.is_fullscreen());
}

#[test]
fn toplevel_state_empty() {
    assert!(ToplevelState::default().is_empty());
}

#[test]
fn toplevel_info_defaults() {
    let info = ToplevelInfo::default();
    assert!(info.app_id.is_empty());
    assert!(info.title.is_empty());
    assert!(info.state.is_empty());
}

#[test]
fn toplevel_action_display() {
    assert_eq!(ToplevelAction::Close.to_string(), "close");
    assert_eq!(ToplevelAction::Maximize.to_string(), "maximize");
    assert_eq!(ToplevelAction::Activate.to_string(), "activate");
}
