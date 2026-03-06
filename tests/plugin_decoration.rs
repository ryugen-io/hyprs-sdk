use hyprs_sdk::plugin::*;

#[test]
fn decoration_position_policy_default() {
    let p = DecorationPositionPolicy::default();
    assert_eq!(p, DecorationPositionPolicy::Absolute);
}

#[test]
fn decoration_position_policy_values() {
    assert_eq!(DecorationPositionPolicy::Absolute as u8, 0);
    assert_eq!(DecorationPositionPolicy::Sticky as u8, 1);
}

#[test]
fn decoration_edges_none() {
    assert_eq!(DecorationEdges::NONE.0, 0);
}

#[test]
fn decoration_edges_all() {
    let all = DecorationEdges::TOP
        | DecorationEdges::BOTTOM
        | DecorationEdges::LEFT
        | DecorationEdges::RIGHT;
    assert_eq!(all, DecorationEdges::ALL);
}

#[test]
fn decoration_edges_contains() {
    let edges = DecorationEdges::TOP | DecorationEdges::LEFT;
    assert!(edges.contains(DecorationEdges::TOP));
    assert!(edges.contains(DecorationEdges::LEFT));
    assert!(!edges.contains(DecorationEdges::BOTTOM));
    assert!(!edges.contains(DecorationEdges::RIGHT));
}

#[test]
fn decoration_edges_bitand() {
    let a = DecorationEdges::TOP | DecorationEdges::BOTTOM;
    let b = DecorationEdges::TOP | DecorationEdges::LEFT;
    let c = a & b;
    assert!(c.contains(DecorationEdges::TOP));
    assert!(!c.contains(DecorationEdges::BOTTOM));
    assert!(!c.contains(DecorationEdges::LEFT));
}

#[test]
fn decoration_type_default() {
    let t = DecorationType::default();
    assert_eq!(t, DecorationType::None);
}

#[test]
fn decoration_type_values() {
    assert_eq!(DecorationType::GroupBar as i8, 0);
    assert_eq!(DecorationType::Shadow as i8, 1);
    assert_eq!(DecorationType::Border as i8, 2);
    assert_eq!(DecorationType::Custom as i8, 3);
}

#[test]
fn decoration_layer_default() {
    let l = DecorationLayer::default();
    assert_eq!(l, DecorationLayer::Bottom);
}

#[test]
fn decoration_layer_order() {
    assert!((DecorationLayer::Bottom as u8) < (DecorationLayer::Under as u8));
    assert!((DecorationLayer::Under as u8) < (DecorationLayer::Over as u8));
    assert!((DecorationLayer::Over as u8) < (DecorationLayer::Overlay as u8));
}

#[test]
fn decoration_flags_none() {
    assert_eq!(DecorationFlags::NONE.0, 0);
}

#[test]
fn decoration_flags_bitor() {
    let flags = DecorationFlags::ALLOWS_MOUSE_INPUT | DecorationFlags::NON_SOLID;
    assert!(flags.contains(DecorationFlags::ALLOWS_MOUSE_INPUT));
    assert!(flags.contains(DecorationFlags::NON_SOLID));
    assert!(!flags.contains(DecorationFlags::PART_OF_MAIN_WINDOW));
}

#[test]
fn decoration_flags_bitand() {
    let a = DecorationFlags::ALLOWS_MOUSE_INPUT | DecorationFlags::NON_SOLID;
    let b = DecorationFlags::ALLOWS_MOUSE_INPUT | DecorationFlags::PART_OF_MAIN_WINDOW;
    let c = a & b;
    assert!(c.contains(DecorationFlags::ALLOWS_MOUSE_INPUT));
    assert!(!c.contains(DecorationFlags::NON_SOLID));
}

#[test]
fn decoration_positioning_info_default() {
    let info = DecorationPositioningInfo::default();
    assert_eq!(info.policy, DecorationPositionPolicy::Absolute);
    assert_eq!(info.edges, DecorationEdges::NONE);
    assert_eq!(info.priority, 0);
    assert!(!info.reserved);
}

#[test]
fn decoration_handle_null() {
    let h = DecorationHandle::NULL;
    assert!(h.is_null());
}

#[test]
fn decoration_handle_non_null() {
    let mut dummy: u8 = 0;
    let h = DecorationHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!h.is_null());
}

/// Verify WindowDecoration trait is object-safe.
#[test]
fn window_decoration_trait_object_safety() {
    fn _assert_object_safe(_: &dyn WindowDecoration) {}
}
