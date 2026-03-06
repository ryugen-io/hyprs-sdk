use hyprs_sdk::ipc::commands::{self, Flags};

#[test]
fn flags_default_is_no_flags() {
    let f = Flags::default();
    assert!(!f.json);
    assert!(!f.reload);
    assert!(!f.all);
    assert!(!f.config);
}

#[test]
fn flags_json_shorthand() {
    let f = Flags::json();
    assert!(f.json);
    assert!(!f.reload);
}

#[test]
fn query_without_flags() {
    assert_eq!(commands::monitors(Flags::default()), "monitors");
}

#[test]
fn query_with_json_flag() {
    assert_eq!(commands::monitors(Flags::json()), "j/monitors");
}

#[test]
fn query_with_multiple_flags() {
    let f = Flags {
        json: true,
        reload: true,
        all: true,
        config: false,
    };
    assert_eq!(commands::clients(f), "jra/clients");
}

#[test]
fn system_info_with_config_flag() {
    let f = Flags {
        config: true,
        ..Flags::default()
    };
    assert_eq!(commands::system_info(f), "c/systeminfo");
}

#[test]
fn dispatch_without_args() {
    assert_eq!(commands::dispatch("killactive", ""), "dispatch killactive");
}

#[test]
fn dispatch_with_args() {
    assert_eq!(commands::dispatch("workspace", "3"), "dispatch workspace 3");
}

#[test]
fn keyword_command() {
    assert_eq!(
        commands::keyword("general:gaps_out", "10"),
        "keyword general:gaps_out 10"
    );
}

#[test]
fn batch_command() {
    let cmds = vec![
        "j/monitors".to_string(),
        "j/clients".to_string(),
        "dispatch workspace 1".to_string(),
    ];
    assert_eq!(
        commands::batch(&cmds),
        "[[BATCH]]j/monitors;j/clients;dispatch workspace 1"
    );
}

#[test]
fn kill_no_flags() {
    assert_eq!(commands::kill(), "kill");
}

#[test]
fn splash_no_flags() {
    assert_eq!(commands::splash(), "splash");
}

#[test]
fn submap_no_flags() {
    assert_eq!(commands::submap(), "submap");
}

#[test]
fn reload_shaders() {
    assert_eq!(commands::reload_shaders(), "reloadshaders");
}

#[test]
fn reload_with_args() {
    assert_eq!(commands::reload("configfile"), "reload configfile");
}

#[test]
fn reload_no_args() {
    assert_eq!(commands::reload(""), "reload");
}

#[test]
fn set_cursor_command() {
    assert_eq!(
        commands::set_cursor("Bibata-Modern", 24),
        "setcursor Bibata-Modern 24"
    );
}

#[test]
fn notify_command() {
    assert_eq!(
        commands::notify(0, 5000, "ff00ff", "hello world"),
        "notify 0 5000 ff00ff hello world"
    );
}

#[test]
fn dismiss_notify_command() {
    assert_eq!(commands::dismiss_notify(-1), "dismissnotify -1");
}

#[test]
fn get_prop_with_flags() {
    assert_eq!(
        commands::get_prop("0xabc", "title", Flags::json()),
        "j/getprop 0xabc title"
    );
}

#[test]
fn get_prop_no_flags() {
    assert_eq!(
        commands::get_prop("0xabc", "title", Flags::default()),
        "getprop 0xabc title"
    );
}

#[test]
fn get_option_with_flags() {
    assert_eq!(
        commands::get_option("general:gaps_out", Flags::json()),
        "j/getoption general:gaps_out"
    );
}

#[test]
fn switch_xkb_layout_command() {
    assert_eq!(
        commands::switch_xkb_layout("main", "next"),
        "switchxkblayout main next"
    );
}

#[test]
fn set_error_with_message() {
    assert_eq!(commands::set_error("broken"), "seterror broken");
}

#[test]
fn set_error_disable() {
    assert_eq!(commands::set_error(""), "seterror disable");
}

#[test]
fn output_command() {
    assert_eq!(
        commands::output("create headless"),
        "output create headless"
    );
}

#[test]
fn plugin_command() {
    assert_eq!(commands::plugin("list"), "plugin list");
}

#[test]
fn decorations_with_flags() {
    assert_eq!(
        commands::decorations("0xabc", Flags::json()),
        "j/decorations 0xabc"
    );
}

#[test]
fn remaining_query_commands_with_json_flag() {
    #[allow(clippy::type_complexity)]
    let cases: &[(fn(Flags) -> String, &str)] = &[
        (commands::workspaces, "workspaces"),
        (commands::workspace_rules, "workspacerules"),
        (commands::active_workspace, "activeworkspace"),
        (commands::active_window, "activewindow"),
        (commands::layers, "layers"),
        (commands::version, "version"),
        (commands::devices, "devices"),
        (commands::cursor_pos, "cursorpos"),
        (commands::binds, "binds"),
        (commands::global_shortcuts, "globalshortcuts"),
        (commands::animations, "animations"),
        (commands::rolling_log, "rollinglog"),
        (commands::layouts, "layouts"),
        (commands::config_errors, "configerrors"),
        (commands::locked, "locked"),
        (commands::descriptions, "descriptions"),
    ];

    for (builder, name) in cases {
        assert_eq!(builder(Flags::json()), format!("j/{name}"));
    }
}
