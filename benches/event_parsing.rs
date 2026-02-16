use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hypr_sdk::ipc::events::parse_event;

// -- Realistic event lines ------------------------------------------------

const WORKSPACE: &str = "workspace>>3";
const WORKSPACE_V2: &str = "workspacev2>>3,main";
const ACTIVE_WINDOW: &str = "activewindow>>kitty,~";
const ACTIVE_WINDOW_V2: &str = "activewindowv2>>55a3f2c0dead";
const OPEN_WINDOW: &str = "openwindow>>55a3f2c0dead,3,kitty,Terminal - zsh";
const CLOSE_WINDOW: &str = "closewindow>>55a3f2c0dead";
const MOVE_WINDOW_V2: &str = "movewindowv2>>55a3f2c0dead,2,code";
const FOCUSED_MON: &str = "focusedmon>>DP-1,3";
const FULLSCREEN: &str = "fullscreen>>1";
const ACTIVE_LAYOUT: &str = "activelayout>>AT Translated Set 2 keyboard,English (US)";
const TOGGLE_GROUP: &str = "togglegroup>>1,55a3f2c0dead,55a3f2c0beef";
const CONFIG_RELOADED: &str = "configreloaded>>";
const UNKNOWN: &str = "futureevent>>some,data,here";

fn bench_single_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_parse_single");

    group.bench_function("workspace", |b| {
        b.iter(|| parse_event(black_box(WORKSPACE)))
    });
    group.bench_function("workspace_v2", |b| {
        b.iter(|| parse_event(black_box(WORKSPACE_V2)))
    });
    group.bench_function("active_window", |b| {
        b.iter(|| parse_event(black_box(ACTIVE_WINDOW)))
    });
    group.bench_function("active_window_v2", |b| {
        b.iter(|| parse_event(black_box(ACTIVE_WINDOW_V2)))
    });
    group.bench_function("open_window", |b| {
        b.iter(|| parse_event(black_box(OPEN_WINDOW)))
    });
    group.bench_function("close_window", |b| {
        b.iter(|| parse_event(black_box(CLOSE_WINDOW)))
    });
    group.bench_function("move_window_v2", |b| {
        b.iter(|| parse_event(black_box(MOVE_WINDOW_V2)))
    });
    group.bench_function("focused_mon", |b| {
        b.iter(|| parse_event(black_box(FOCUSED_MON)))
    });
    group.bench_function("fullscreen", |b| {
        b.iter(|| parse_event(black_box(FULLSCREEN)))
    });
    group.bench_function("active_layout", |b| {
        b.iter(|| parse_event(black_box(ACTIVE_LAYOUT)))
    });
    group.bench_function("toggle_group", |b| {
        b.iter(|| parse_event(black_box(TOGGLE_GROUP)))
    });
    group.bench_function("config_reloaded", |b| {
        b.iter(|| parse_event(black_box(CONFIG_RELOADED)))
    });
    group.bench_function("unknown_event", |b| {
        b.iter(|| parse_event(black_box(UNKNOWN)))
    });

    group.finish();
}

fn bench_event_stream_throughput(c: &mut Criterion) {
    // Simulate a burst of 100 events (typical compositor activity).
    let events = [
        WORKSPACE,
        WORKSPACE_V2,
        ACTIVE_WINDOW,
        ACTIVE_WINDOW_V2,
        OPEN_WINDOW,
        CLOSE_WINDOW,
        MOVE_WINDOW_V2,
        FOCUSED_MON,
        FULLSCREEN,
        ACTIVE_LAYOUT,
        TOGGLE_GROUP,
        CONFIG_RELOADED,
        UNKNOWN,
    ];
    let stream: Vec<&str> = events.iter().cycle().take(100).copied().collect();

    c.bench_function("event_stream_100", |b| {
        b.iter(|| {
            for line in &stream {
                black_box(parse_event(black_box(line)));
            }
        })
    });
}

criterion_group!(benches, bench_single_events, bench_event_stream_throughput);
criterion_main!(benches);
