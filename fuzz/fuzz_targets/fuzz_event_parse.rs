#![no_main]
use libfuzzer_sys::fuzz_target;
use hypr_sdk::ipc::events::parse_event;

fn clamp(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

fuzz_target!(|data: &str| {
    // Raw input path.
    let _ = parse_event(data);

    // Structured paths to exercise known parser branches.
    let mut fields = data.split([',', '|', ';', '\n']);
    let a = clamp(fields.next().unwrap_or(""), 64);
    let b = clamp(fields.next().unwrap_or(""), 64);
    let c = clamp(fields.next().unwrap_or(""), 64);
    let d = clamp(fields.next().unwrap_or(""), 64);

    let lines = [
        format!("workspace>>{a}"),
        format!("workspacev2>>42,{a}"),
        format!("createworkspacev2>>1,{a}"),
        format!("focusedmonv2>>{a},7"),
        format!("monitoraddedv2>>1,{a},{b}"),
        format!("monitorremovedv2>>2,{a},{b}"),
        format!("activespecialv2>>-99,{a},{b}"),
        format!("openwindow>>{a},1,{b},{c}"),
        format!("windowtitlev2>>{a},{b}"),
        format!("movewindowv2>>{a},3,{b}"),
        format!("togglegroup>>1,{a},{b},{c}"),
        format!("activelayout>>{a},{b}"),
        format!("screencast>>1,{a}"),
        format!("custom>>{a}"),
        format!("futureevent>>{a},{b},{c},{d}"),
    ];

    for line in &lines {
        let _ = parse_event(line);
    }
});
