#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprs_sdk::ipc::responses::*;
use hyprs_sdk::types::monitor::Monitor;
use hyprs_sdk::types::window::Window;
use hyprs_sdk::types::workspace::Workspace;

fuzz_target!(|data: &[u8]| {
    // All deserializers must never panic on any input.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<VersionInfo>(s);
        let _ = serde_json::from_str::<Window>(s);
        let _ = serde_json::from_str::<Monitor>(s);
        let _ = serde_json::from_str::<Workspace>(s);
        let _ = serde_json::from_str::<DevicesResponse>(s);
        let _ = serde_json::from_str::<Vec<Bind>>(s);
        let _ = serde_json::from_str::<CursorPosition>(s);
        let _ = serde_json::from_str::<OptionValue>(s);
        let _ = serde_json::from_str::<LockState>(s);
        let _ = serde_json::from_str::<Vec<WorkspaceRuleInfo>>(s);
        let _ = serde_json::from_str::<Vec<GlobalShortcutInfo>>(s);
        let _ = serde_json::from_str::<Vec<DecorationInfo>>(s);
        let _ = serde_json::from_str::<Vec<ConfigDescription>>(s);
        let _ = serde_json::from_str::<Vec<PluginInfo>>(s);
        let _ = AnimationsResponse::from_json(s);
    }
});
