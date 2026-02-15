#![cfg(feature = "wayland")]
use hypr_sdk::protocols::toplevel_export::*;

#[test]
fn frame_format_buffer_size() {
    let fmt = ToplevelFrameFormat {
        format: 0x34325241,
        width: 800,
        height: 600,
        stride: 3200,
    };
    assert_eq!(fmt.buffer_size(), 3200 * 600);
}

#[test]
fn frame_flags() {
    assert!(ToplevelFrameFlags::empty().is_empty());
    assert!(ToplevelFrameFlags::Y_INVERT.contains(ToplevelFrameFlags::Y_INVERT));
}
