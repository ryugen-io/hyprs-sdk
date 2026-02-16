use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hypr_sdk::ipc::commands::{self, Flags};

fn bench_flag_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("flag_prefix");
    group.bench_function("json_only", |b| {
        b.iter(|| commands::monitors(black_box(Flags::json())))
    });
    group.bench_function("no_flags", |b| {
        b.iter(|| commands::monitors(black_box(Flags::default())))
    });
    group.bench_function("all_flags", |b| {
        let flags = Flags {
            json: true,
            reload: true,
            all: true,
            config: true,
        };
        b.iter(|| commands::monitors(black_box(flags)))
    });
    group.finish();
}

fn bench_query_commands(c: &mut Criterion) {
    let json = Flags::json();
    let mut group = c.benchmark_group("query_commands");

    group.bench_function("workspaces", |b| {
        b.iter(|| commands::workspaces(black_box(json)))
    });
    group.bench_function("clients", |b| b.iter(|| commands::clients(black_box(json))));
    group.bench_function("active_window", |b| {
        b.iter(|| commands::active_window(black_box(json)))
    });
    group.bench_function("version", |b| b.iter(|| commands::version(black_box(json))));
    group.bench_function("devices", |b| b.iter(|| commands::devices(black_box(json))));
    group.bench_function("binds", |b| b.iter(|| commands::binds(black_box(json))));
    group.bench_function("kill", |b| b.iter(|| commands::kill()));
    group.bench_function("splash", |b| b.iter(|| commands::splash()));

    group.finish();
}

fn bench_parameterized_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("param_commands");

    group.bench_function("dispatch", |b| {
        b.iter(|| commands::dispatch(black_box("workspace"), black_box("3")))
    });
    group.bench_function("keyword", |b| {
        b.iter(|| commands::keyword(black_box("general:border_size"), black_box("2")))
    });
    group.bench_function("notify", |b| {
        b.iter(|| {
            commands::notify(
                black_box(0),
                black_box(5000),
                black_box("0xff0000ff"),
                black_box("Hello world"),
            )
        })
    });
    group.bench_function("set_cursor", |b| {
        b.iter(|| commands::set_cursor(black_box("Bibata-Modern-Classic"), black_box(24)))
    });
    group.bench_function("get_option", |b| {
        b.iter(|| commands::get_option(black_box("general:gaps_in"), black_box(Flags::json())))
    });
    group.bench_function("get_prop", |b| {
        b.iter(|| {
            commands::get_prop(
                black_box("0x55a3f2c0dead"),
                black_box("alpha"),
                black_box(Flags::json()),
            )
        })
    });

    group.finish();
}

fn bench_batch(c: &mut Criterion) {
    let cmds: Vec<String> = (1..=10)
        .map(|i| commands::dispatch("workspace", &i.to_string()))
        .collect();

    let mut group = c.benchmark_group("batch");
    group.bench_function("10_dispatches", |b| {
        b.iter(|| commands::batch(black_box(&cmds)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_flag_prefix,
    bench_query_commands,
    bench_parameterized_commands,
    bench_batch,
);
criterion_main!(benches);
