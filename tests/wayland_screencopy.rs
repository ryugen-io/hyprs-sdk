#![cfg(feature = "wayland")]
use hyprs_sdk::protocols::screencopy::*;

#[test]
fn frame_format_buffer_size() {
    let fmt = FrameFormat {
        pixel_format: PixelFormat::Argb8888,
        width: 1920,
        height: 1080,
        stride: 1920 * 4,
    };
    assert_eq!(fmt.buffer_size(), 1920 * 1080 * 4);
}

#[test]
fn pixel_format_from_raw() {
    assert_eq!(PixelFormat::from_raw(0), Some(PixelFormat::Argb8888));
    assert_eq!(PixelFormat::from_raw(1), Some(PixelFormat::Xrgb8888));
    assert_eq!(PixelFormat::from_raw(99), None);
}

#[test]
fn capture_region() {
    let r = CaptureRegion {
        x: 100,
        y: 200,
        width: 800,
        height: 600,
    };
    assert_eq!(r.width, 800);
}

#[test]
fn frame_flags() {
    assert!(FrameFlags::empty().is_empty());
    assert!(FrameFlags::Y_INVERT.contains(FrameFlags::Y_INVERT));
    assert!(!FrameFlags::empty().contains(FrameFlags::Y_INVERT));
}
