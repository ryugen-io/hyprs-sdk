#![cfg(feature = "wayland")]
use hypr_sdk::protocols::pointer_warp::*;

#[test]
fn warp_target_construction() {
    let target = WarpTarget::new(500.0, 300.0);
    assert!((target.x - 500.0).abs() < f64::EPSILON);
    assert!((target.y - 300.0).abs() < f64::EPSILON);
}
