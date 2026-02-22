use hypr_sdk::plugin::*;

#[test]
fn config_default_bool() {
    let val = ConfigDefault::Bool(true);
    assert_eq!(val, ConfigDefault::Bool(true));
}

#[test]
fn config_default_int() {
    let val = ConfigDefault::Int(42);
    assert_eq!(val, ConfigDefault::Int(42));
}

#[test]
fn config_default_float() {
    let val = ConfigDefault::Float(3.125);
    assert_eq!(val, ConfigDefault::Float(3.125));
}

#[test]
fn config_default_string() {
    let val = ConfigDefault::String("hello".into());
    assert_eq!(val, ConfigDefault::String("hello".into()));
}

#[test]
fn config_default_color() {
    let val = ConfigDefault::Color(0xFF0000FF);
    assert_eq!(val, ConfigDefault::Color(0xFF0000FF));
}

#[test]
fn config_default_vec2() {
    let val = ConfigDefault::Vec2(1.0, 2.0);
    assert_eq!(val, ConfigDefault::Vec2(1.0, 2.0));
}

#[test]
fn config_value_handle_null() {
    let h = ConfigValueHandle::NULL;
    assert!(h.is_null());
}

#[test]
fn config_value_handle_non_null() {
    let mut dummy: u8 = 0;
    let h = ConfigValueHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!h.is_null());
}

#[test]
fn keyword_handler_options_default() {
    let opts = KeywordHandlerOptions::default();
    assert!(!opts.allow_flags);
}

#[test]
fn keyword_handler_type_check() {
    // WHY: Needed for correctness and maintainability: Verify KeywordHandler can hold a closure.
    let _h: KeywordHandler = Box::new(|_value| Ok(()));
}

#[test]
fn config_default_clone() {
    let a = ConfigDefault::Int(99);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn config_default_debug() {
    let val = ConfigDefault::Bool(true);
    let s = format!("{val:?}");
    assert!(s.contains("Bool"));
}
