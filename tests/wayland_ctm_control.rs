#![cfg(feature = "wayland")]
use hypr_sdk::protocols::ctm_control::*;

#[test]
fn identity_matrix() {
    let m = ColorTransformMatrix::IDENTITY;
    assert!((m.elements[0] - 1.0).abs() < f64::EPSILON);
    assert!((m.elements[4] - 1.0).abs() < f64::EPSILON);
    assert!((m.elements[8] - 1.0).abs() < f64::EPSILON);
    assert!((m.elements[1] - 0.0).abs() < f64::EPSILON);
}

#[test]
fn scale_matrix() {
    let m = ColorTransformMatrix::scale(0.5, 1.0, 0.8);
    assert!((m.elements[0] - 0.5).abs() < f64::EPSILON);
    assert!((m.elements[4] - 1.0).abs() < f64::EPSILON);
    assert!((m.elements[8] - 0.8).abs() < f64::EPSILON);
}

#[test]
fn grayscale_matrix() {
    let m = ColorTransformMatrix::grayscale();
    // All rows should be identical (same luminance weights)
    assert_eq!(m.elements[0..3], m.elements[3..6]);
    assert_eq!(m.elements[3..6], m.elements[6..9]);
}

#[test]
fn default_is_identity() {
    assert_eq!(
        ColorTransformMatrix::default(),
        ColorTransformMatrix::IDENTITY
    );
}
