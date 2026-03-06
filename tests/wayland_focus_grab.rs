#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::focus_grab::*;

#[test]
fn focus_grab_state_variants() {
    assert_ne!(FocusGrabState::Active, FocusGrabState::Cleared);
}
