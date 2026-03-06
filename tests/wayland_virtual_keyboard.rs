#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::virtual_keyboard::*;

#[test]
fn key_state_variants() {
    assert_eq!(KeyState::Released as u32, 0);
    assert_eq!(KeyState::Pressed as u32, 1);
}

#[test]
fn key_event_construction() {
    let ev = KeyEvent {
        time: 500,
        key: 28,
        state: KeyState::Pressed,
    }; // KEY_ENTER
    assert_eq!(ev.key, 28);
    assert_eq!(ev.state, KeyState::Pressed);
}

#[test]
fn modifier_state_defaults() {
    let mods = ModifierState::default();
    assert_eq!(mods.mods_depressed, 0);
    assert_eq!(mods.mods_latched, 0);
    assert_eq!(mods.mods_locked, 0);
    assert_eq!(mods.group, 0);
}
