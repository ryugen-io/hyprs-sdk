#![cfg(feature = "wayland")]
use hypr_sdk::protocols::virtual_pointer::*;

#[test]
fn button_state_variants() {
    assert_eq!(ButtonState::Released as u32, 0);
    assert_eq!(ButtonState::Pressed as u32, 1);
}

#[test]
fn axis_source_variants() {
    assert_eq!(AxisSource::Wheel as u32, 0);
    assert_eq!(AxisSource::WheelTilt as u32, 3);
}

#[test]
fn motion_event_construction() {
    let ev = MotionEvent {
        time: 1000,
        dx: 5.0,
        dy: -3.0,
    };
    assert_eq!(ev.time, 1000);
    assert!((ev.dx - 5.0).abs() < f64::EPSILON);
}

#[test]
fn button_event_construction() {
    let ev = ButtonEvent {
        time: 2000,
        button: 0x110,
        state: ButtonState::Pressed,
    };
    assert_eq!(ev.button, 0x110); // BTN_LEFT
    assert_eq!(ev.state, ButtonState::Pressed);
}

#[test]
fn absolute_motion_normalized() {
    let ev = MotionAbsoluteEvent {
        time: 0,
        x: 0.5,
        y: 0.5,
        x_extent: 1920,
        y_extent: 1080,
    };
    assert_eq!(ev.x_extent, 1920);
}
