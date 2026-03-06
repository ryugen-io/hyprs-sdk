#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprs_sdk::types::common::WindowAddress;

fuzz_target!(|data: &str| {
    // WindowAddress::from_str must never panic.
    let _ = data.parse::<WindowAddress>();
});
