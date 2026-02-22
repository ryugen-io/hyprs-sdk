use hypr_sdk::plugin::*;

#[test]
fn color_new() {
    let c = Color::new(0.5, 0.6, 0.7, 0.8);
    assert_eq!(c.r, 0.5);
    assert_eq!(c.g, 0.6);
    assert_eq!(c.b, 0.7);
    assert_eq!(c.a, 0.8);
}

#[test]
fn color_rgb() {
    let c = Color::rgb(1.0, 0.0, 0.5);
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.5);
    assert_eq!(c.a, 1.0);
}

#[test]
fn color_constants() {
    assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
    assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
    assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
}

#[test]
fn color_default() {
    let c = Color::default();
    assert_eq!(c.r, 0.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.a, 0.0);
}

#[test]
fn color_debug() {
    let c = Color::RED;
    let s = format!("{c:?}");
    assert!(s.contains("Color"));
}

#[test]
fn color_clone() {
    let a = Color::rgb(0.1, 0.2, 0.3);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn hyprctl_output_format_values() {
    assert_eq!(HyprCtlOutputFormat::Normal as u8, 0);
    assert_eq!(HyprCtlOutputFormat::Json as u8, 1);
}

#[test]
fn hyprctl_output_format_default() {
    let fmt = HyprCtlOutputFormat::default();
    assert_eq!(fmt, HyprCtlOutputFormat::Normal);
}

#[test]
fn function_hook_handle_null() {
    let h = hypr_sdk::plugin::api::FunctionHookHandle::NULL;
    assert!(h.is_null());
}

#[test]
fn function_hook_handle_non_null() {
    let mut dummy: u8 = 0;
    let h = hypr_sdk::plugin::api::FunctionHookHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!h.is_null());
}

#[test]
fn hook_callback_type_check() {
    // WHY: Needed for correctness and maintainability: Verify HookCallback can hold a closure.
    let _cb: HookCallback = Box::new(|_info, _data| {});
}

#[test]
fn hyprctl_command_handler_type_check() {
    // WHY: Needed for correctness and maintainability: Verify HyprCtlCommandHandler can hold a closure.
    let _h: HyprCtlCommandHandler = Box::new(|_fmt, _args| String::new());
}

// Note: Functions that call FFI (register_hook, invoke_hyprctl,
// WHY: Needed for correctness and maintainability: register_hyprctl_command, add_notification, etc.) cannot be tested
// WHY: Needed for correctness and maintainability: outside a Hyprland plugin process because the FFI symbols are
// WHY: Needed for correctness and maintainability: only available when linked into the compositor.
