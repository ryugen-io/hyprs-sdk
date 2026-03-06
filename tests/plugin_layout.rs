use hyprs_sdk::plugin::*;

#[test]
fn layout_handle_null() {
    let h = LayoutHandle::NULL;
    assert!(h.is_null());
}

#[test]
fn layout_handle_non_null() {
    let mut dummy: u8 = 0;
    let h = LayoutHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!h.is_null());
}

#[test]
fn direction_default() {
    let d = Direction::default();
    assert_eq!(d, Direction::Default);
}

#[test]
fn direction_variants() {
    assert_eq!(Direction::Up as i8, 0);
    assert_eq!(Direction::Right as i8, 1);
    assert_eq!(Direction::Down as i8, 2);
    assert_eq!(Direction::Left as i8, 3);
}

#[test]
fn rect_corner_default() {
    let c = RectCorner::default();
    assert_eq!(c, RectCorner::None);
}

#[test]
fn rect_corner_values() {
    assert_eq!(RectCorner::TopLeft as u8, 1);
    assert_eq!(RectCorner::TopRight as u8, 2);
    assert_eq!(RectCorner::BottomRight as u8, 4);
    assert_eq!(RectCorner::BottomLeft as u8, 8);
}

/// Verify Layout trait is object-safe (can be used as `dyn Layout`).
#[test]
fn layout_trait_object_safety() {
    fn _assert_object_safe(_: &dyn Layout) {}
}
