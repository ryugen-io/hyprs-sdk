#![cfg(feature = "wayland")]
use hypr_sdk::protocols::gamma_control;

#[test]
fn gamma_table_identity() {
    let table = gamma_control::GammaTable::identity(256);
    assert_eq!(table.size, 256);
    assert_eq!(table.red.len(), 256);
    assert_eq!(table.red[0], 0);
    assert_eq!(table.red[255], u16::MAX);
}

#[test]
fn gamma_table_to_bytes() {
    let table = gamma_control::GammaTable::identity(4);
    let bytes = table.to_bytes();
    assert_eq!(bytes.len(), 24); // 3 * 4 * 2
}

#[test]
fn gamma_table_brightness() {
    let table = gamma_control::GammaTable::with_brightness(256, 0.5);
    assert!(table.red[255] < u16::MAX);
    assert!(table.red[255] > 0);
}

#[test]
fn gamma_table_gamma_correction() {
    let table = gamma_control::GammaTable::with_gamma(256, 2.2);
    // WHY: Needed for correctness and maintainability: Gamma > 1 darkens midtones
    let mid = table.red[128];
    let identity_mid = gamma_control::GammaTable::identity(256).red[128];
    assert!(mid < identity_mid);
}
