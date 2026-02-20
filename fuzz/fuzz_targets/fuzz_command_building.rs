#![no_main]
use libfuzzer_sys::fuzz_target;
use hypr_sdk::dispatch::{self, Corner, Direction, ToggleState};
use hypr_sdk::ipc::commands::{self, Flags};

fn assert_dispatch(cmd: dispatch::DispatchCmd) {
    // Dispatch command names are always static, non-empty dispatcher IDs.
    assert!(!cmd.name.is_empty());
    let _ = cmd.args.len();
}

fuzz_target!(|data: &str| {
    // IPC command builders must never panic on any input.
    let flags = [
        Flags::default(),
        Flags::json(),
        Flags {
            json: true,
            reload: true,
            all: true,
            config: true,
        },
    ];

    for f in flags {
        let _ = commands::workspaces(f);
        let _ = commands::workspace_rules(f);
        let _ = commands::active_workspace(f);
        let _ = commands::clients(f);
        let _ = commands::active_window(f);
        let _ = commands::layers(f);
        let _ = commands::version(f);
        let _ = commands::devices(f);
        let _ = commands::cursor_pos(f);
        let _ = commands::binds(f);
        let _ = commands::global_shortcuts(f);
        let _ = commands::system_info(f);
        let _ = commands::animations(f);
        let _ = commands::rolling_log(f);
        let _ = commands::layouts(f);
        let _ = commands::config_errors(f);
        let _ = commands::locked(f);
        let _ = commands::descriptions(f);
        let _ = commands::monitors(f);
        let _ = commands::get_option(data, f);
        let _ = commands::get_prop(data, data, f);
        let _ = commands::decorations(data, f);
    }

    let _ = commands::kill();
    let _ = commands::splash();
    let _ = commands::submap();
    let _ = commands::reload_shaders();
    let _ = commands::reload(data);
    let _ = commands::plugin(data);
    let _ = commands::notify(0, 5000, data, data);
    let _ = commands::dismiss_notify(-1);
    let _ = commands::set_error(data);
    let _ = commands::switch_xkb_layout(data, data);
    let _ = commands::output(data);
    let _ = commands::dispatch(data, data);
    let _ = commands::keyword(data, data);
    let _ = commands::set_cursor(data, 24);

    let batch = commands::batch(&[
        commands::dispatch("workspace", data),
        commands::keyword("general:gaps_in", data),
        commands::output(data),
    ]);
    assert!(batch.starts_with("[[BATCH]]"));

    // Typed dispatch builders must never panic on any input.
    let dirs = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ];
    for dir in dirs {
        assert_dispatch(dispatch::focus::move_focus(dir));
        assert_dispatch(dispatch::movement::move_window(dir, data));
        assert_dispatch(dispatch::movement::swap_window(dir));
        assert_dispatch(dispatch::group::move_into_group(dir));
        assert_dispatch(dispatch::group::move_window_or_group(dir));
    }

    let toggles = [ToggleState::Toggle, ToggleState::On, ToggleState::Off];
    for state in toggles {
        assert_dispatch(dispatch::group::lock_groups(state));
        assert_dispatch(dispatch::group::lock_active_group(state));
        assert_dispatch(dispatch::group::set_ignore_group_lock(state));
        assert_dispatch(dispatch::group::deny_window_from_group(state));
    }

    let corners = [
        Corner::BottomLeft,
        Corner::BottomRight,
        Corner::TopRight,
        Corner::TopLeft,
    ];
    for corner in corners {
        assert_dispatch(dispatch::movement::move_cursor_to_corner(corner));
    }

    let sig = (data.len() % 31) as u8;
    let pos = (data.len() % 4000) as i32;

    assert_dispatch(dispatch::exec::exec(data));
    assert_dispatch(dispatch::exec::execr(data));
    assert_dispatch(dispatch::exec::exit());

    assert_dispatch(dispatch::window::kill_active());
    assert_dispatch(dispatch::window::force_kill_active());
    assert_dispatch(dispatch::window::close_window(data));
    assert_dispatch(dispatch::window::kill_window(data));
    assert_dispatch(dispatch::window::signal(sig));
    assert_dispatch(dispatch::window::signal_window(data, sig));
    assert_dispatch(dispatch::window::toggle_floating(data));
    assert_dispatch(dispatch::window::set_floating(data));
    assert_dispatch(dispatch::window::set_tiled(data));
    assert_dispatch(dispatch::window::pin(data));
    assert_dispatch(dispatch::window::toggle_swallow());
    assert_dispatch(dispatch::window::bring_active_to_top());
    assert_dispatch(dispatch::window::alter_zorder(data, data));
    assert_dispatch(dispatch::window::center_window());
    assert_dispatch(dispatch::window::set_prop(data, data, data));
    assert_dispatch(dispatch::window::tag_window(data));
    assert_dispatch(dispatch::window::fullscreen(sig % 2));
    assert_dispatch(dispatch::window::fullscreen_state(data, data));

    assert_dispatch(dispatch::focus::focus_window(data));
    assert_dispatch(dispatch::focus::focus_window_by_class(data));
    assert_dispatch(dispatch::focus::focus_urgent_or_last());
    assert_dispatch(dispatch::focus::focus_current_or_last());
    assert_dispatch(dispatch::focus::cycle_next(data));
    assert_dispatch(dispatch::focus::focus_monitor(data));

    assert_dispatch(dispatch::movement::swap_next(data));
    assert_dispatch(dispatch::movement::move_active(data, data));
    assert_dispatch(dispatch::movement::resize_active(data, data));
    assert_dispatch(dispatch::movement::move_window_pixel(data, data, data));
    assert_dispatch(dispatch::movement::resize_window_pixel(data, data, data));
    assert_dispatch(dispatch::movement::move_to_workspace(data));
    assert_dispatch(dispatch::movement::move_to_workspace_window(data, data));
    assert_dispatch(dispatch::movement::move_to_workspace_silent(data));
    assert_dispatch(dispatch::movement::move_cursor(pos, -pos));

    assert_dispatch(dispatch::workspace::switch(data));
    assert_dispatch(dispatch::workspace::rename((data.len() as i64) % 100, data));
    assert_dispatch(dispatch::workspace::toggle_special(data));
    assert_dispatch(dispatch::workspace::workspace_opt(data));
    assert_dispatch(dispatch::workspace::focus_on_current_monitor(data));
    assert_dispatch(dispatch::workspace::move_current_to_monitor(data));
    assert_dispatch(dispatch::workspace::move_to_monitor(data, data));
    assert_dispatch(dispatch::workspace::swap_active_workspaces(data, data));

    assert_dispatch(dispatch::group::toggle_group());
    assert_dispatch(dispatch::group::change_active(data));
    assert_dispatch(dispatch::group::move_window(data));
    assert_dispatch(dispatch::group::move_out_of_group(data));

    assert_dispatch(dispatch::layout::pseudo(data));
    assert_dispatch(dispatch::layout::toggle_split());
    assert_dispatch(dispatch::layout::swap_split());
    assert_dispatch(dispatch::layout::split_ratio(data));
    assert_dispatch(dispatch::layout::layout_msg(data));

    assert_dispatch(dispatch::input::mouse(data));
    assert_dispatch(dispatch::input::pass(data));
    assert_dispatch(dispatch::input::send_shortcut(data, data, data));
    assert_dispatch(dispatch::input::send_key_state(data, data, data, data));
    assert_dispatch(dispatch::input::submap(data));
    assert_dispatch(dispatch::input::global(data));
    assert_dispatch(dispatch::input::dpms(data, data));

    assert_dispatch(dispatch::misc::force_renderer_reload());
    assert_dispatch(dispatch::misc::event(data));
    assert_dispatch(dispatch::misc::force_idle(data));
});
