use hyprs_sdk::plugin::*;

#[test]
fn dispatch_result_constructors() {
    let ok = DispatchResult::ok();
    assert!(ok.success);
    assert!(!ok.pass_event);

    let err = DispatchResult::err("bad args");
    assert!(!err.success);
    assert_eq!(err.error, "bad args");

    let pass = DispatchResult::pass();
    assert!(pass.success);
    assert!(pass.pass_event);
}

#[test]
fn dispatcher_fn_type_check() {
    // WHY: Needed for correctness and maintainability: Verify DispatcherFn is a valid type alias that can hold a closure.
    let _f: DispatcherFn = Box::new(|args| {
        let _ = args;
        DispatchResult::ok()
    });
}

#[test]
fn dispatch_result_default() {
    let d = DispatchResult::default();
    assert!(!d.success);
    assert!(!d.pass_event);
    assert!(d.error.is_empty());
}

#[test]
fn dispatch_result_debug() {
    let r = DispatchResult::ok();
    let s = format!("{r:?}");
    assert!(s.contains("DispatchResult"));
}
