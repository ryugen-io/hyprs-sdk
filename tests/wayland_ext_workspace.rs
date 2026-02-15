#![cfg(feature = "wayland")]
use hypr_sdk::protocols::ext_workspace::*;

#[test]
fn workspace_state_flags() {
    let state = WorkspaceState::ACTIVE | WorkspaceState::URGENT;
    assert!(state.is_active());
    assert!(state.is_urgent());
    assert!(!state.is_hidden());
}

#[test]
fn workspace_state_empty() {
    assert!(WorkspaceState::default().is_empty());
}

#[test]
fn workspace_group_capabilities() {
    let caps = WorkspaceGroupCapabilities::CREATE_WORKSPACE;
    assert!(caps.contains(WorkspaceGroupCapabilities::CREATE_WORKSPACE));
}

#[test]
fn workspace_capabilities() {
    let caps = WorkspaceCapabilities::ACTIVATE;
    assert!(caps.contains(WorkspaceCapabilities::ACTIVATE));
    assert!(!caps.contains(WorkspaceCapabilities::REMOVE));
}

#[test]
fn workspace_coordinates_default() {
    let coords = WorkspaceCoordinates::default();
    assert_eq!(coords.x, 0);
    assert_eq!(coords.y, 0);
}
