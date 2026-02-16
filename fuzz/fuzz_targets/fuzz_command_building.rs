#![no_main]
use libfuzzer_sys::fuzz_target;
use hypr_sdk::ipc::commands::{self, Flags};

fuzz_target!(|data: &str| {
    // Command builders must never panic on any input.
    let flags = Flags::json();

    let _ = commands::dispatch(data, data);
    let _ = commands::keyword(data, data);
    let _ = commands::notify(0, 5000, data, data);
    let _ = commands::set_error(data);
    let _ = commands::switch_xkb_layout(data, data);
    let _ = commands::output(data);
    let _ = commands::set_cursor(data, 24);
    let _ = commands::get_option(data, flags);
    let _ = commands::get_prop(data, data, flags);
    let _ = commands::decorations(data, flags);
    let _ = commands::reload(data);
    let _ = commands::plugin(data);
    let _ = commands::batch(&[data.to_string()]);
});
