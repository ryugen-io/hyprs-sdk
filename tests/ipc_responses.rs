use hyprs_sdk::ipc::responses::*;

#[test]
fn version_info_deserialize() {
    let json = r#"{
        "branch": "main",
        "commit": "abc123",
        "version": "0.53.0",
        "dirty": false,
        "commit_message": "fix: something",
        "commit_date": "2025-01-01",
        "tag": "v0.53.0",
        "commits": "42",
        "buildAquamarine": "0.4.0",
        "buildHyprlang": "0.5.0",
        "buildHyprutils": "0.2.0",
        "buildHyprcursor": "0.1.0",
        "buildHyprgraphics": "0.1.0",
        "systemAquamarine": "0.4.0",
        "systemHyprlang": "0.5.0",
        "systemHyprutils": "0.2.0",
        "systemHyprcursor": "0.1.0",
        "systemHyprgraphics": "0.1.0",
        "abiHash": "deadbeef",
        "flags": ["debug", "no xwayland"]
    }"#;
    let info: VersionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.branch, "main");
    assert_eq!(info.version, "0.53.0");
    assert!(!info.dirty);
    assert_eq!(info.build_aquamarine, "0.4.0");
    assert_eq!(info.system_hyprlang, "0.5.0");
    assert_eq!(info.abi_hash, "deadbeef");
    assert_eq!(info.flags, vec!["debug", "no xwayland"]);
}

#[test]
fn version_info_ignores_unknown_fields() {
    let json = r#"{"branch": "main", "future_field": true}"#;
    let info: VersionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.branch, "main");
}

#[test]
fn devices_deserialize() {
    let json = r#"{
        "mice": [{"address": "0x1234", "name": "Logitech", "defaultSpeed": 1.0, "scrollFactor": 1.5}],
        "keyboards": [{"address": "0x5678", "name": "AT Keyboard", "rules": "", "model": "",
                       "layout": "us", "variant": "", "options": "", "active_keymap": "English (US)",
                       "capsLock": false, "numLock": true, "main": true}],
        "tablets": [],
        "touch": [{"address": "0xaaaa", "name": "Touch Panel"}],
        "switches": []
    }"#;
    let devs: DevicesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(devs.mice.len(), 1);
    assert_eq!(devs.mice[0].name, "Logitech");
    assert_eq!(devs.mice[0].scroll_factor, 1.5);
    assert_eq!(devs.keyboards.len(), 1);
    assert_eq!(devs.keyboards[0].layout, "us");
    assert!(devs.keyboards[0].num_lock);
    assert!(devs.keyboards[0].main);
    assert_eq!(devs.touch.len(), 1);
}

#[test]
fn bind_deserialize() {
    let json = r#"[{
        "locked": false,
        "mouse": false,
        "release": false,
        "repeat": true,
        "longPress": false,
        "non_consuming": false,
        "has_description": true,
        "modmask": 64,
        "submap": "",
        "submap_universal": "",
        "key": "Return",
        "keycode": 0,
        "catch_all": false,
        "description": "Open terminal",
        "dispatcher": "exec",
        "arg": "kitty"
    }]"#;
    let binds: Vec<Bind> = serde_json::from_str(json).unwrap();
    assert_eq!(binds.len(), 1);
    assert!(binds[0].repeat);
    assert!(binds[0].has_description);
    assert_eq!(binds[0].modmask, 64);
    assert_eq!(binds[0].dispatcher, "exec");
    assert_eq!(binds[0].arg, "kitty");
    assert_eq!(binds[0].description, "Open terminal");
}

#[test]
fn cursor_position_deserialize() {
    let json = r#"{"x": 1920, "y": 540}"#;
    let pos: CursorPosition = serde_json::from_str(json).unwrap();
    assert_eq!(pos.x, 1920);
    assert_eq!(pos.y, 540);
}

#[test]
fn animations_response_from_json() {
    let json = r#"[
        [{"name": "windowsIn", "overridden": false, "bezier": "default", "enabled": true, "speed": 3.0, "style": "slide"}],
        [{"name": "default", "X0": 0.25, "Y0": 0.1, "X1": 0.25, "Y1": 1.0}]
    ]"#;
    let resp = AnimationsResponse::from_json(json).unwrap();
    assert_eq!(resp.animations.len(), 1);
    assert_eq!(resp.animations[0].name, "windowsIn");
    assert_eq!(resp.animations[0].speed, 3.0);
    assert!(resp.animations[0].enabled);
    assert_eq!(resp.beziers.len(), 1);
    assert_eq!(resp.beziers[0].name, "default");
    assert_eq!(resp.beziers[0].x0, 0.25);
    assert_eq!(resp.beziers[0].y1, 1.0);
}

#[test]
fn global_shortcut_info_deserialize() {
    let json = r#"[{"name": "myapp:toggle", "description": "Toggle app"}]"#;
    let shortcuts: Vec<GlobalShortcutInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(shortcuts.len(), 1);
    assert_eq!(shortcuts[0].name, "myapp:toggle");
    assert_eq!(shortcuts[0].description, "Toggle app");
}

#[test]
fn workspace_rule_deserialize() {
    let json = r#"[{
        "workspaceString": "1",
        "monitor": "DP-1",
        "default": true,
        "persistent": false,
        "gapsIn": [5, 5, 5, 5],
        "gapsOut": [10, 10, 10, 10],
        "borderSize": 2,
        "border": true,
        "rounding": true,
        "decorate": true,
        "shadow": false,
        "defaultName": "main"
    }]"#;
    let rules: Vec<WorkspaceRuleInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].workspace_string, "1");
    assert_eq!(rules[0].monitor, "DP-1");
    assert!(rules[0].default);
    assert_eq!(rules[0].gaps_in, Some(vec![5, 5, 5, 5]));
    assert_eq!(rules[0].border_size, Some(2));
    assert_eq!(rules[0].default_name, "main");
}

#[test]
fn workspace_rule_optional_fields() {
    let json = r#"[{"workspaceString": "2"}]"#;
    let rules: Vec<WorkspaceRuleInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(rules[0].workspace_string, "2");
    assert!(rules[0].gaps_in.is_none());
    assert!(rules[0].border_size.is_none());
    assert!(rules[0].border.is_none());
}

#[test]
fn lock_state_deserialize() {
    let json = r#"{"locked": true}"#;
    let state: LockState = serde_json::from_str(json).unwrap();
    assert!(state.locked);
}

#[test]
fn option_value_int() {
    let json = r#"{"option": "general:border_size", "int": 2, "set": true}"#;
    let opt: OptionValue = serde_json::from_str(json).unwrap();
    assert_eq!(opt.option, "general:border_size");
    assert_eq!(opt.int, Some(2));
    assert!(opt.set);
    assert!(opt.float.is_none());
}

#[test]
fn option_value_string() {
    let json = r#"{"option": "general:layout", "str": "dwindle", "set": true}"#;
    let opt: OptionValue = serde_json::from_str(json).unwrap();
    assert_eq!(opt.str, Some("dwindle".to_string()));
}

#[test]
fn option_value_vec2() {
    let json = r#"{"option": "cursor:hotspot_padding", "vec2": [0.0, 0.0], "set": false}"#;
    let opt: OptionValue = serde_json::from_str(json).unwrap();
    assert_eq!(opt.vec2, Some([0.0, 0.0]));
    assert!(!opt.set);
}

#[test]
fn decoration_info_deserialize() {
    let json = r#"[
        {"decorationName": "Border", "priority": 100},
        {"decorationName": "Shadow", "priority": 50}
    ]"#;
    let decos: Vec<DecorationInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(decos.len(), 2);
    assert_eq!(decos[0].decoration_name, "Border");
    assert_eq!(decos[0].priority, 100);
    assert_eq!(decos[1].decoration_name, "Shadow");
}

#[test]
fn layouts_deserialize() {
    let json = r#"["dwindle", "master"]"#;
    let layouts: Vec<String> = serde_json::from_str(json).unwrap();
    assert_eq!(layouts, vec!["dwindle", "master"]);
}

#[test]
fn config_errors_deserialize() {
    let json = r#"["line 42: unknown option", "line 99: syntax error"]"#;
    let errors: Vec<String> = serde_json::from_str(json).unwrap();
    assert_eq!(errors.len(), 2);
}

#[test]
fn tablet_with_parent_deserialize() {
    let json = r#"{
        "mice": [],
        "keyboards": [],
        "tablets": [
            {"address": "0x1", "type": "tabletPad", "belongsTo": {"address": "0x2", "name": "Wacom"}}
        ],
        "touch": [],
        "switches": []
    }"#;
    let devs: DevicesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(devs.tablets.len(), 1);
    assert_eq!(devs.tablets[0].tablet_type, "tabletPad");
    let parent = devs.tablets[0].belongs_to.as_ref().unwrap();
    assert_eq!(parent.name, "Wacom");
}

#[test]
fn config_description_bool_type() {
    let json = r#"[{
        "value": "general:border_size",
        "description": "Border size in pixels",
        "type": 0,
        "flags": 0,
        "data": {
            "value": true,
            "current": 1,
            "explicit": true
        }
    }]"#;
    let descs: Vec<ConfigDescription> = serde_json::from_str(json).unwrap();
    assert_eq!(descs.len(), 1);
    assert_eq!(descs[0].value, "general:border_size");
    assert_eq!(descs[0].option_type, 0);
    assert!(descs[0].data["explicit"].as_bool().unwrap());
}

#[test]
fn config_description_range_type() {
    let json = r#"[{
        "value": "general:gaps_in",
        "description": "Inner gaps",
        "type": 1,
        "flags": 0,
        "data": {
            "value": 5,
            "min": 0,
            "max": 100,
            "current": 5,
            "explicit": false
        }
    }]"#;
    let descs: Vec<ConfigDescription> = serde_json::from_str(json).unwrap();
    assert_eq!(descs[0].option_type, 1);
    assert_eq!(descs[0].data["min"], 0);
    assert_eq!(descs[0].data["max"], 100);
}

#[test]
fn config_description_ignores_unknown_fields() {
    let json = r#"[{"value": "test", "description": "d", "type": 0, "flags": 0, "data": {}, "future": true}]"#;
    let descs: Vec<ConfigDescription> = serde_json::from_str(json).unwrap();
    assert_eq!(descs[0].value, "test");
}

#[test]
fn plugin_info_deserialize() {
    let json = r#"[{
        "name": "hyprexpo",
        "author": "vaxerski",
        "handle": "5603c0ab3000",
        "version": "1.0.0",
        "description": "Expo overview plugin"
    }]"#;
    let plugins: Vec<PluginInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "hyprexpo");
    assert_eq!(plugins[0].author, "vaxerski");
    assert_eq!(plugins[0].handle, "5603c0ab3000");
    assert_eq!(plugins[0].version, "1.0.0");
}

#[test]
fn plugin_info_empty_list() {
    let json = "[]";
    let plugins: Vec<PluginInfo> = serde_json::from_str(json).unwrap();
    assert!(plugins.is_empty());
}

#[test]
fn plugin_info_ignores_unknown_fields() {
    let json = r#"[{"name": "test", "author": "a", "handle": "0", "version": "1", "description": "d", "extra": 42}]"#;
    let plugins: Vec<PluginInfo> = serde_json::from_str(json).unwrap();
    assert_eq!(plugins[0].name, "test");
}

#[test]
fn get_prop_value_minsize() {
    let json = r#"{"minSize": [100, 200]}"#;
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let arr = val["minSize"].as_array().unwrap();
    assert_eq!(arr[0], 100);
    assert_eq!(arr[1], 200);
}

#[test]
fn get_prop_value_alpha() {
    let json = r#"{"alpha": 0.85}"#;
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!((val["alpha"].as_f64().unwrap() - 0.85).abs() < f64::EPSILON);
}
