use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use hyprs_sdk::dispatch::{self, Corner, Direction, DispatchCmd, ToggleState};

fn build_all_dispatchers() -> Vec<DispatchCmd> {
    vec![
        dispatch::exec::exec("kitty"),
        dispatch::exec::execr("echo hi"),
        dispatch::exec::exit(),
        dispatch::window::kill_active(),
        dispatch::window::force_kill_active(),
        dispatch::window::close_window("class:kitty"),
        dispatch::window::kill_window("class:kitty"),
        dispatch::window::signal(9),
        dispatch::window::signal_window("class:kitty", 9),
        dispatch::window::toggle_floating("active"),
        dispatch::window::set_floating("active"),
        dispatch::window::set_tiled("active"),
        dispatch::window::pin("active"),
        dispatch::window::toggle_swallow(),
        dispatch::window::bring_active_to_top(),
        dispatch::window::alter_zorder("top", "active"),
        dispatch::window::center_window(),
        dispatch::window::set_prop("active", "alpha", "0.9"),
        dispatch::window::tag_window("dev,active"),
        dispatch::window::fullscreen(0),
        dispatch::window::fullscreen_state("0", "2"),
        dispatch::focus::move_focus(Direction::Left),
        dispatch::focus::focus_window("class:firefox"),
        dispatch::focus::focus_window_by_class("class:firefox"),
        dispatch::focus::focus_urgent_or_last(),
        dispatch::focus::focus_current_or_last(),
        dispatch::focus::cycle_next("next"),
        dispatch::focus::focus_monitor("DP-1"),
        dispatch::movement::move_window(Direction::Right, "active"),
        dispatch::movement::swap_window(Direction::Up),
        dispatch::movement::swap_next("prev"),
        dispatch::movement::move_active("+50", "-30"),
        dispatch::movement::resize_active("+100", "-50"),
        dispatch::movement::move_window_pixel("100", "200", "active"),
        dispatch::movement::resize_window_pixel("900", "500", "active"),
        dispatch::movement::move_to_workspace("3"),
        dispatch::movement::move_to_workspace_window("3", "class:kitty"),
        dispatch::movement::move_to_workspace_silent("special:scratchpad"),
        dispatch::movement::move_cursor(100, 200),
        dispatch::movement::move_cursor_to_corner(Corner::TopRight),
        dispatch::workspace::switch("3"),
        dispatch::workspace::rename(3, "dev"),
        dispatch::workspace::toggle_special("scratchpad"),
        dispatch::workspace::workspace_opt("allfloat"),
        dispatch::workspace::focus_on_current_monitor("2"),
        dispatch::workspace::move_current_to_monitor("DP-2"),
        dispatch::workspace::move_to_monitor("3", "DP-2"),
        dispatch::workspace::swap_active_workspaces("DP-1", "DP-2"),
        dispatch::group::toggle_group(),
        dispatch::group::change_active("f"),
        dispatch::group::move_window("b"),
        dispatch::group::lock_groups(ToggleState::On),
        dispatch::group::lock_active_group(ToggleState::Off),
        dispatch::group::move_into_group(Direction::Left),
        dispatch::group::move_out_of_group("active"),
        dispatch::group::move_window_or_group(Direction::Right),
        dispatch::group::set_ignore_group_lock(ToggleState::Toggle),
        dispatch::group::deny_window_from_group(ToggleState::On),
        dispatch::layout::pseudo("active"),
        dispatch::layout::toggle_split(),
        dispatch::layout::swap_split(),
        dispatch::layout::split_ratio("+0.1"),
        dispatch::layout::layout_msg("swapwithmaster"),
        dispatch::input::mouse("1movewindow"),
        dispatch::input::pass("class:discord"),
        dispatch::input::send_shortcut("CTRL SHIFT", "t", "class:kitty"),
        dispatch::input::send_key_state("CTRL", "k", "repeat", "class:kitty"),
        dispatch::input::submap("resize"),
        dispatch::input::global("myapp:toggle"),
        dispatch::input::dpms("off", "DP-1"),
        dispatch::misc::force_renderer_reload(),
        dispatch::misc::event("perf-test"),
        dispatch::misc::force_idle("5000"),
    ]
}

fn bench_dispatch_hot_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_hot_paths");
    group.bench_function("workspace_switch", |b| {
        b.iter(|| dispatch::workspace::switch(black_box("3")))
    });
    group.bench_function("move_focus", |b| {
        b.iter(|| dispatch::focus::move_focus(black_box(Direction::Left)))
    });
    group.bench_function("move_window", |b| {
        b.iter(|| dispatch::movement::move_window(black_box(Direction::Right), black_box("active")))
    });
    group.bench_function("dispatch_exec", |b| {
        b.iter(|| dispatch::exec::exec(black_box("kitty")))
    });
    group.bench_function("set_prop", |b| {
        b.iter(|| {
            dispatch::window::set_prop(black_box("active"), black_box("alpha"), black_box("0.9"))
        })
    });
    group.finish();
}

fn bench_dispatch_bulk(c: &mut Criterion) {
    c.bench_function("build_all_dispatchers_once", |b| {
        b.iter(|| black_box(build_all_dispatchers()))
    });

    c.bench_function("build_all_dispatchers_100x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(build_all_dispatchers());
            }
        })
    });
}

criterion_group!(benches, bench_dispatch_hot_paths, bench_dispatch_bulk);
criterion_main!(benches);
