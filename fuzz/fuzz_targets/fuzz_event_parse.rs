#![no_main]
use libfuzzer_sys::fuzz_target;
use hypr_sdk::ipc::events::parse_event;

fuzz_target!(|data: &str| {
    // parse_event must never panic on any input.
    let _ = parse_event(data);
});
