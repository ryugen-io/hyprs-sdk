use hypr_sdk::dispatch::*;

// -- exec --------------------------------------------------------------------

#[test]
fn exec_command() {
    let cmd = exec::exec("kitty");
    assert_eq!(cmd.name, "exec");
    assert_eq!(cmd.args, "kitty");
}

#[test]
fn exec_with_rules() {
    let cmd = exec::exec("[float;size 800 600] kitty");
    assert_eq!(cmd.args, "[float;size 800 600] kitty");
}

#[test]
fn execr_command() {
    let cmd = exec::execr("echo hello");
    assert_eq!(cmd.name, "execr");
}

#[test]
fn exit_command() {
    let cmd = exec::exit();
    assert_eq!(cmd.name, "exit");
    assert!(cmd.args.is_empty());
}

// -- window ------------------------------------------------------------------

#[test]
fn kill_active() {
    assert_eq!(window::kill_active().name, "killactive");
}

#[test]
fn close_window() {
    let cmd = window::close_window("class:kitty");
    assert_eq!(cmd.name, "closewindow");
    assert_eq!(cmd.args, "class:kitty");
}

#[test]
fn signal_window() {
    let cmd = window::signal_window("class:firefox", 9);
    assert_eq!(cmd.args, "class:firefox,9");
}

#[test]
fn toggle_floating() {
    let cmd = window::toggle_floating("active");
    assert_eq!(cmd.name, "togglefloating");
}

#[test]
fn set_prop() {
    let cmd = window::set_prop("class:kitty", "opacity", "0.9");
    assert_eq!(cmd.name, "setprop");
    assert_eq!(cmd.args, "class:kitty opacity 0.9");
}

#[test]
fn fullscreen_modes() {
    assert_eq!(window::fullscreen(0).args, "0");
    assert_eq!(window::fullscreen(1).args, "1");
}

#[test]
fn fullscreen_state() {
    let cmd = window::fullscreen_state("0", "2");
    assert_eq!(cmd.args, "0 2");
}

#[test]
fn pin() {
    let cmd = window::pin("active");
    assert_eq!(cmd.name, "pin");
}

#[test]
fn alter_zorder() {
    let cmd = window::alter_zorder("top", "class:kitty");
    assert_eq!(cmd.args, "top,class:kitty");
}

#[test]
fn tag_window() {
    let cmd = window::tag_window("mytag,class:kitty");
    assert_eq!(cmd.name, "tagwindow");
}

// -- focus -------------------------------------------------------------------

#[test]
fn move_focus_directions() {
    assert_eq!(focus::move_focus(Direction::Left).args, "l");
    assert_eq!(focus::move_focus(Direction::Right).args, "r");
    assert_eq!(focus::move_focus(Direction::Up).args, "u");
    assert_eq!(focus::move_focus(Direction::Down).args, "d");
}

#[test]
fn focus_window() {
    let cmd = focus::focus_window("class:firefox");
    assert_eq!(cmd.name, "focuswindow");
    assert_eq!(cmd.args, "class:firefox");
}

#[test]
fn focus_monitor() {
    let cmd = focus::focus_monitor("DP-1");
    assert_eq!(cmd.name, "focusmonitor");
}

#[test]
fn cycle_next() {
    let cmd = focus::cycle_next("float");
    assert_eq!(cmd.args, "float");
}

#[test]
fn focus_urgent_or_last() {
    assert!(focus::focus_urgent_or_last().args.is_empty());
}

// -- movement ----------------------------------------------------------------

#[test]
fn move_window_direction() {
    let cmd = movement::move_window(Direction::Left, "");
    assert_eq!(cmd.args, "l");
}

#[test]
fn move_window_with_regex() {
    let cmd = movement::move_window(Direction::Right, "class:kitty");
    assert_eq!(cmd.args, "r,class:kitty");
}

#[test]
fn swap_window() {
    let cmd = movement::swap_window(Direction::Up);
    assert_eq!(cmd.name, "swapwindow");
    assert_eq!(cmd.args, "u");
}

#[test]
fn move_active() {
    let cmd = movement::move_active("+50", "-30");
    assert_eq!(cmd.args, "+50 -30");
}

#[test]
fn resize_active() {
    let cmd = movement::resize_active("+100", "+0");
    assert_eq!(cmd.args, "+100 +0");
}

#[test]
fn move_to_workspace() {
    let cmd = movement::move_to_workspace("3");
    assert_eq!(cmd.name, "movetoworkspace");
    assert_eq!(cmd.args, "3");
}

#[test]
fn move_to_workspace_silent() {
    let cmd = movement::move_to_workspace_silent("special:scratchpad");
    assert_eq!(cmd.name, "movetoworkspacesilent");
}

#[test]
fn move_cursor() {
    let cmd = movement::move_cursor(100, 200);
    assert_eq!(cmd.args, "100 200");
}

#[test]
fn move_cursor_to_corner() {
    let cmd = movement::move_cursor_to_corner(Corner::TopRight);
    assert_eq!(cmd.args, "2");
}

// -- workspace ---------------------------------------------------------------

#[test]
fn workspace_switch() {
    let cmd = workspace::switch("3");
    assert_eq!(cmd.name, "workspace");
    assert_eq!(cmd.args, "3");
}

#[test]
fn workspace_switch_relative() {
    let cmd = workspace::switch("+1");
    assert_eq!(cmd.args, "+1");
}

#[test]
fn workspace_rename() {
    let cmd = workspace::rename(1, "dev");
    assert_eq!(cmd.args, "1 dev");
}

#[test]
fn workspace_rename_clear() {
    let cmd = workspace::rename(1, "");
    assert_eq!(cmd.args, "1");
}

#[test]
fn toggle_special() {
    let cmd = workspace::toggle_special("scratchpad");
    assert_eq!(cmd.name, "togglespecialworkspace");
}

#[test]
fn move_current_to_monitor() {
    let cmd = workspace::move_current_to_monitor("DP-2");
    assert_eq!(cmd.name, "movecurrentworkspacetomonitor");
}

#[test]
fn move_workspace_to_monitor() {
    let cmd = workspace::move_to_monitor("3", "HDMI-A-1");
    assert_eq!(cmd.args, "3 HDMI-A-1");
}

#[test]
fn swap_active_workspaces() {
    let cmd = workspace::swap_active_workspaces("DP-1", "DP-2");
    assert_eq!(cmd.args, "DP-1 DP-2");
}

// -- group -------------------------------------------------------------------

#[test]
fn toggle_group() {
    assert!(group::toggle_group().args.is_empty());
}

#[test]
fn change_group_active() {
    let cmd = group::change_active("f");
    assert_eq!(cmd.name, "changegroupactive");
    assert_eq!(cmd.args, "f");
}

#[test]
fn lock_groups() {
    let cmd = group::lock_groups(ToggleState::On);
    assert_eq!(cmd.args, "on");
}

#[test]
fn move_into_group() {
    let cmd = group::move_into_group(Direction::Left);
    assert_eq!(cmd.name, "moveintogroup");
    assert_eq!(cmd.args, "l");
}

#[test]
fn move_window_or_group() {
    let cmd = group::move_window_or_group(Direction::Right);
    assert_eq!(cmd.name, "movewindoworgroup");
}

#[test]
fn deny_window_from_group() {
    let cmd = group::deny_window_from_group(ToggleState::Toggle);
    assert_eq!(cmd.args, "toggle");
}

// -- layout ------------------------------------------------------------------

#[test]
fn pseudo() {
    let cmd = layout::pseudo("active");
    assert_eq!(cmd.name, "pseudo");
}

#[test]
fn toggle_split() {
    assert!(layout::toggle_split().args.is_empty());
}

#[test]
fn split_ratio() {
    let cmd = layout::split_ratio("+0.1");
    assert_eq!(cmd.args, "+0.1");
}

#[test]
fn layout_msg() {
    let cmd = layout::layout_msg("swapwithmaster");
    assert_eq!(cmd.name, "layoutmsg");
}

// -- input -------------------------------------------------------------------

#[test]
fn submap_enter() {
    let cmd = input::submap("resize");
    assert_eq!(cmd.name, "submap");
    assert_eq!(cmd.args, "resize");
}

#[test]
fn submap_reset() {
    let cmd = input::submap("reset");
    assert_eq!(cmd.args, "reset");
}

#[test]
fn send_shortcut() {
    let cmd = input::send_shortcut("CTRL SHIFT", "t", "class:kitty");
    assert_eq!(cmd.args, "CTRL SHIFT t class:kitty");
}

#[test]
fn dpms_off() {
    let cmd = input::dpms("off", "");
    assert_eq!(cmd.args, "off");
}

#[test]
fn dpms_off_monitor() {
    let cmd = input::dpms("off", "DP-1");
    assert_eq!(cmd.args, "off DP-1");
}

#[test]
fn global_shortcut() {
    let cmd = input::global("myapp:toggle");
    assert_eq!(cmd.name, "global");
}

#[test]
fn pass_input() {
    let cmd = input::pass("class:discord");
    assert_eq!(cmd.name, "pass");
}

// -- misc --------------------------------------------------------------------

#[test]
fn force_renderer_reload() {
    assert_eq!(misc::force_renderer_reload().name, "forcerendererreload");
}

#[test]
fn custom_event() {
    let cmd = misc::event("my custom data");
    assert_eq!(cmd.name, "event");
    assert_eq!(cmd.args, "my custom data");
}

#[test]
fn force_idle() {
    let cmd = misc::force_idle("5000");
    assert_eq!(cmd.name, "forceidle");
}

// -- coverage gaps ------------------------------------------------------------

#[test]
fn force_kill_active() {
    let cmd = window::force_kill_active();
    assert_eq!(cmd.name, "forcekillactive");
    assert!(cmd.args.is_empty());
}

#[test]
fn kill_window() {
    let cmd = window::kill_window("class:steam");
    assert_eq!(cmd.name, "killwindow");
    assert_eq!(cmd.args, "class:steam");
}

#[test]
fn signal() {
    let cmd = window::signal(15);
    assert_eq!(cmd.name, "signal");
    assert_eq!(cmd.args, "15");
}

#[test]
fn set_floating() {
    let cmd = window::set_floating("class:kitty");
    assert_eq!(cmd.name, "setfloating");
    assert_eq!(cmd.args, "class:kitty");
}

#[test]
fn set_tiled() {
    let cmd = window::set_tiled("class:kitty");
    assert_eq!(cmd.name, "settiled");
    assert_eq!(cmd.args, "class:kitty");
}

#[test]
fn toggle_swallow() {
    let cmd = window::toggle_swallow();
    assert_eq!(cmd.name, "toggleswallow");
    assert!(cmd.args.is_empty());
}

#[test]
fn bring_active_to_top() {
    let cmd = window::bring_active_to_top();
    assert_eq!(cmd.name, "bringactivetotop");
    assert!(cmd.args.is_empty());
}

#[test]
fn center_window() {
    let cmd = window::center_window();
    assert_eq!(cmd.name, "centerwindow");
    assert!(cmd.args.is_empty());
}

#[test]
fn focus_window_by_class() {
    let cmd = focus::focus_window_by_class("class:firefox");
    assert_eq!(cmd.name, "focuswindowbyclass");
    assert_eq!(cmd.args, "class:firefox");
}

#[test]
fn focus_current_or_last() {
    let cmd = focus::focus_current_or_last();
    assert_eq!(cmd.name, "focuscurrentorlast");
    assert!(cmd.args.is_empty());
}

#[test]
fn swap_next() {
    let cmd = movement::swap_next("prev");
    assert_eq!(cmd.name, "swapnext");
    assert_eq!(cmd.args, "prev");
}

#[test]
fn move_window_pixel() {
    let cmd = movement::move_window_pixel("10", "20", "class:kitty");
    assert_eq!(cmd.name, "movewindowpixel");
    assert_eq!(cmd.args, "10 20,class:kitty");
}

#[test]
fn resize_window_pixel() {
    let cmd = movement::resize_window_pixel("1920", "1080", "address:0xdead");
    assert_eq!(cmd.name, "resizewindowpixel");
    assert_eq!(cmd.args, "1920 1080,address:0xdead");
}

#[test]
fn move_to_workspace_window() {
    let cmd = movement::move_to_workspace_window("3", "class:kitty");
    assert_eq!(cmd.name, "movetoworkspace");
    assert_eq!(cmd.args, "3,class:kitty");
}

#[test]
fn workspace_opt() {
    let cmd = workspace::workspace_opt("allfloat");
    assert_eq!(cmd.name, "workspaceopt");
    assert_eq!(cmd.args, "allfloat");
}

#[test]
fn focus_on_current_monitor() {
    let cmd = workspace::focus_on_current_monitor("4");
    assert_eq!(cmd.name, "focusworkspaceoncurrentmonitor");
    assert_eq!(cmd.args, "4");
}

#[test]
fn group_move_window() {
    let cmd = group::move_window("b");
    assert_eq!(cmd.name, "movegroupwindow");
    assert_eq!(cmd.args, "b");
}

#[test]
fn lock_active_group() {
    let cmd = group::lock_active_group(ToggleState::Off);
    assert_eq!(cmd.name, "lockactivegroup");
    assert_eq!(cmd.args, "off");
}

#[test]
fn move_out_of_group() {
    let cmd = group::move_out_of_group("class:kitty");
    assert_eq!(cmd.name, "moveoutofgroup");
    assert_eq!(cmd.args, "class:kitty");
}

#[test]
fn set_ignore_group_lock() {
    let cmd = group::set_ignore_group_lock(ToggleState::Toggle);
    assert_eq!(cmd.name, "setignoregrouplock");
    assert_eq!(cmd.args, "toggle");
}

#[test]
fn swap_split() {
    let cmd = layout::swap_split();
    assert_eq!(cmd.name, "swapsplit");
    assert!(cmd.args.is_empty());
}

#[test]
fn mouse() {
    let cmd = input::mouse("1movewindow");
    assert_eq!(cmd.name, "mouse");
    assert_eq!(cmd.args, "1movewindow");
}

#[test]
fn send_key_state() {
    let cmd = input::send_key_state("CTRL", "k", "repeat", "class:kitty");
    assert_eq!(cmd.name, "sendkeystate");
    assert_eq!(cmd.args, "CTRL k repeat class:kitty");
}
