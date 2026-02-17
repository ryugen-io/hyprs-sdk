"""hypr-tui: TUI dashboard for hypr-sdk tests, benchmarks, and fuzz results."""

from __future__ import annotations

from textual.app import App, ComposeResult
from textual.containers import Grid, Horizontal, ItemGrid, Vertical, VerticalScroll
from textual.widgets import (
    DataTable, Footer, Header, Static, TabbedContent, TabPane,
    RichLog, Sparkline, Digits,
)
from textual import work

from . import runners

# -- Fix ambiguous-width Unicode characters -----------------------------------
# Terminals (kitty, foot, etc.) render East Asian Ambiguous chars as 2 cells,
# but Rich's wcwidth table reports 1.  This causes Textual borders to misalign.
# Patch get_character_cell_size so Rich agrees with the terminal.
def _patch_cell_widths() -> None:
    import rich.cells
    from functools import lru_cache

    _orig = rich.cells.get_character_cell_size.__wrapped__
    _wide = frozenset("\u2714\u2699\u2718\u25cb\u23f3\u2605\u26a0\u2713")

    @lru_cache(maxsize=4096)
    def _get_character_cell_size(character: str, unicode_version: str = "auto") -> int:
        if character in _wide:
            return 2
        return _orig(character, unicode_version)

    rich.cells.get_character_cell_size = _get_character_cell_size
    rich.cells.cached_cell_len.cache_clear()

_patch_cell_widths()


# ── Visual helpers ───────────────────────────────────────────────


def _pass_fail_bar(passed: int, failed: int, ignored: int, width: int = 30) -> str:
    """Tri-color bar: green/red/dim."""
    total = passed + failed + ignored
    if total == 0:
        return f"[dim]{'░' * width}[/dim]"
    pw = int((passed / total) * width)
    fw = int((failed / total) * width)
    iw = width - pw - fw
    return (
        f"[green]{'█' * pw}[/green]"
        f"[red]{'█' * fw}[/red]"
        f"[dim]{'░' * iw}[/dim]"
    )


def _pacman(elapsed: float, duration: float, width: int = 36) -> str:
    """Pacman eating dots — animated progress bar."""
    if duration <= 0:
        return f"[dim]{'·' * width}[/dim]"
    ratio = min(elapsed / duration, 1.0)
    pos = int(ratio * width)
    eaten = "[green]●[/green]" * pos
    if pos < width:
        mouth = "[yellow]ᗧ[/yellow]"
        remaining = "[dim]·[/dim]" * (width - pos - 1)
    else:
        mouth = "[green]●[/green]"
        remaining = ""
    return f"{eaten}{mouth}{remaining}"


# ── Styles ───────────────────────────────────────────────────────


CSS = """
Screen {
    background: $surface;
}

/* ── Status cards ─────────────────────────────────────────── */

#status-row {
    height: 5;
    layout: grid;
    grid-size: 5;
    grid-gutter: 1;
    padding: 0 1;
}

.status-card {
    height: 5;
    border: heavy $primary-darken-2;
    padding: 0 1;
    content-align: left middle;
}

.status-card.ok  { border: heavy $success; }
.status-card.err { border: heavy $error; }

/* ── Tabs ─────────────────────────────────────────────────── */

TabbedContent { height: 1fr; }
TabPane { padding: 0; }
ContentSwitcher { height: 1fr; }

/* ── Tests tab ────────────────────────────────────────────── */

#test-layout { height: 1fr; }

#test-summary {
    height: 3;
    padding: 0 2;
    content-align: center middle;
}

#test-suites-row {
    height: auto;
    max-height: 16;
    padding: 0 1;
    grid-gutter: 1;
    overflow-y: auto;
}

.suite-card {
    height: 5;
    padding: 0 1;
    border: tall $primary-darken-2;
}

.suite-card.suite-pass { border: tall $success; }
.suite-card.suite-fail { border: tall $error; }

#test-table { height: 1fr; }

/* ── Bench tab ────────────────────────────────────────────── */

#bench-layout { height: 1fr; }
#bench-table  { height: 1fr; }

#bench-sparkline-box {
    height: 8;
    border: tall $primary-darken-2;
    padding: 0 1;
}

#bench-sparkline-label {
    height: 1;
    color: $text-muted;
    text-style: italic;
    content-align: center middle;
}

Sparkline { height: 5; margin: 0; }
Sparkline > .sparkline--max-color { color: $warning; }
Sparkline > .sparkline--min-color { color: $success; }

/* ── Fuzz tab ─────────────────────────────────────────────── */

#fuzz-layout  { height: 1fr; }

#fuzz-targets {
    height: auto;
    max-height: 20;
    padding: 0 1;
}

.fuzz-card {
    height: 3;
    margin: 0 0 1 0;
    padding: 0 2;
    border: tall $primary-darken-2;
}

.fuzz-card.fuzz-clean   { border: tall $success; }
.fuzz-card.fuzz-crash   { border: tall $error; }
.fuzz-card.fuzz-running { border: tall $warning; }

#fuzz-log {
    height: 1fr;
    border: tall $primary-darken-2;
    padding: 0 1;
    margin: 0;
}

/* ── Checks tab ───────────────────────────────────────────── */

#checks-layout { height: 1fr; }

#checks-gates {
    height: auto;
    max-height: 16;
    padding: 0 1;
}

.check-card {
    height: 3;
    margin: 0 0 1 0;
    padding: 0 2;
    border: tall $primary-darken-2;
}

.check-card.check-pass    { border: tall $success; }
.check-card.check-fail    { border: tall $error; }
.check-card.check-running { border: tall $warning; }

#checks-git {
    height: auto;
    max-height: 16;
    padding: 0 2;
    border: tall $primary-darken-2;
    margin: 0 1;
}

#checks-log {
    height: 1fr;
    border: tall $primary-darken-2;
    padding: 0 1;
    margin: 0;
}

/* ── Update tab ──────────────────────────────────────────── */

#update-layout { height: 1fr; }

#update-header {
    height: 5;
    padding: 0 2;
    border: tall $primary-darken-2;
    margin: 0 1;
    content-align: left middle;
}

#update-header.update-available { border: tall $warning; }
#update-header.update-current   { border: tall $success; }

#update-details {
    height: 1fr;
    border: tall $primary-darken-2;
    padding: 0 1;
    margin: 0 1;
}

/* ── Metrics tab ──────────────────────────────────────────── */

#metrics-layout { height: 1fr; }

#metrics-numbers {
    height: auto;
    max-height: 20;
    padding: 0 1;
    grid-gutter: 1;
}

.metric-panel {
    height: 8;
    padding: 0 1;
    border: tall $primary-darken-2;
    content-align: center middle;
}

.metric-label {
    height: 1;
    text-style: bold;
    color: $text-muted;
    text-opacity: 70%;
    content-align: center middle;
}

.metric-sublabel {
    height: 1;
    color: $text-muted;
    text-opacity: 50%;
    content-align: center middle;
    text-style: italic;
}

Digits {
    height: auto;
    content-align: center middle;
}

#metrics-charts {
    height: 1fr;
    padding: 0 1;
    grid-gutter: 1;
}

.chart-box {
    height: 1fr;
    border: tall $primary-darken-2;
    padding: 0 1;
}

.chart-label {
    height: 1;
    color: $text-muted;
    text-style: italic;
    content-align: center middle;
}

#metrics-suite-bars { height: 1fr; }

/* ── DataTable global ─────────────────────────────────────── */

DataTable { scrollbar-size: 1 1; }

DataTable > .datatable--header {
    text-style: bold;
    background: $primary-background;
}
"""


# ── Status card widget ───────────────────────────────────────────


class StatusCard(Static):
    """Compact status indicator card."""

    _COLORS = {"pass": "green", "fail": "red", "warn": "yellow", "info": ""}

    def __init__(self, icon: str, title: str, card_id: str) -> None:
        self._icon = icon
        self._title = title
        super().__init__(
            f"[dim bold]{icon} {title}[/dim bold]\n[bold]--[/bold]",
            id=card_id, classes="status-card pending",
        )

    def set_value(self, value: str, status: str = "info") -> None:
        c = self._COLORS.get(status, "")
        val = f"[{c} bold]{value}[/{c} bold]" if c else f"[bold]{value}[/bold]"
        self.update(f"[dim bold]{self._icon} {self._title}[/dim bold]\n{val}")
        self.remove_class("ok", "err", "pending")
        if status == "pass":
            self.add_class("ok")
        elif status == "fail":
            self.add_class("err")
        else:
            self.add_class("pending")


# ── Main app ─────────────────────────────────────────────────────


class HyprTui(App):
    """TUI dashboard for hypr-sdk quality metrics."""

    CSS = CSS
    TITLE = "hypr-sdk"
    SUB_TITLE = "quality dashboard"
    BINDINGS = [
        ("t", "run_tests", "Tests"),
        ("b", "run_benchmarks", "Bench"),
        ("f", "run_fuzz", "Fuzz (10s)"),
        ("c", "run_checks", "Checks"),
        ("u", "check_updates", "Update"),
        ("m", "show_metrics", "Metrics"),
        ("r", "refresh_all", "Refresh"),
        ("d", "toggle_dark", "Dark/Light"),
        ("q", "quit", "Quit"),
    ]

    def __init__(self) -> None:
        super().__init__()
        self._last_tests: runners.TestSummary | None = None
        self._last_benches: list[runners.BenchResult] = []
        self._last_fuzz: list[runners.FuzzTarget] = []
        self._last_checks: list[runners.CheckResult] = []
        self._last_git: runners.GitSummary | None = None

    # ── Compose ───────────────────────────────────────────────

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)

        with Grid(id="status-row"):
            yield StatusCard("\u2714", "TESTS", "card-tests")
            yield StatusCard("\u26a1", "BENCH", "card-bench")
            yield StatusCard("\U0001f50d", "FUZZ", "card-fuzz")
            yield StatusCard("\u2699", "CHECKS", "card-checks")
            yield StatusCard("\u21bb", "UPDATE", "card-update")

        with TabbedContent():
            # ── Tests tab ────────────────────────────────────
            with TabPane("Tests [bold dim]t[/]", id="tab-tests"):
                with Vertical(id="test-layout"):
                    yield Static(
                        "[dim]Press [bold]t[/bold] to run tests[/dim]",
                        id="test-summary",
                    )
                    yield ItemGrid(
                        min_column_width=22,
                        stretch_height=True,
                        regular=True,
                        id="test-suites-row",
                    )
                    yield DataTable(id="test-table")

            # ── Benchmarks tab ───────────────────────────────
            with TabPane("Benchmarks [bold dim]b[/]", id="tab-bench"):
                with Vertical(id="bench-layout"):
                    with Vertical(id="bench-sparkline-box"):
                        yield Static(
                            "benchmark latency distribution (ns)",
                            id="bench-sparkline-label",
                        )
                        yield Sparkline([], id="bench-sparkline")
                    yield DataTable(id="bench-table")

            # ── Fuzz tab ─────────────────────────────────────
            with TabPane("Fuzz [bold dim]f[/]", id="tab-fuzz"):
                with Vertical(id="fuzz-layout"):
                    yield VerticalScroll(id="fuzz-targets")
                    yield RichLog(id="fuzz-log", markup=True)

            # ── Checks tab ───────────────────────────────────
            with TabPane("Checks [bold dim]c[/]", id="tab-checks"):
                with Vertical(id="checks-layout"):
                    yield VerticalScroll(id="checks-gates")
                    yield Static(
                        "[dim]Press [bold]c[/bold] to run quality gates[/dim]",
                        id="checks-git",
                    )
                    yield RichLog(id="checks-log", markup=True)

            # ── Update tab ───────────────────────────────────
            with TabPane("Update [bold dim]u[/]", id="tab-update"):
                with Vertical(id="update-layout"):
                    yield Static(
                        "[dim]Press [bold]u[/bold] to check Hyprland source for updates[/dim]",
                        id="update-header",
                    )
                    yield RichLog(id="update-details", markup=True)

            # ── Metrics tab ──────────────────────────────────
            with TabPane("Metrics [bold dim]m[/]", id="tab-metrics"):
                with Vertical(id="metrics-layout"):
                    with ItemGrid(
                        min_column_width=20, regular=True,
                        id="metrics-numbers",
                    ):
                        with Vertical(classes="metric-panel"):
                            yield Static("TESTS", classes="metric-label")
                            yield Digits("--", id="m-tests")
                            yield Static("total", classes="metric-sublabel")
                        with Vertical(classes="metric-panel"):
                            yield Static("PASS RATE", classes="metric-label")
                            yield Digits("--", id="m-pass-rate")
                            yield Static("percent", classes="metric-sublabel")
                        with Vertical(classes="metric-panel"):
                            yield Static("BENCHMARKS", classes="metric-label")
                            yield Digits("--", id="m-benches")
                            yield Static("total", classes="metric-sublabel")
                        with Vertical(classes="metric-panel"):
                            yield Static("FUZZ", classes="metric-label")
                            yield Digits("--", id="m-fuzz")
                            yield Static("clean", classes="metric-sublabel")
                    with ItemGrid(
                        min_column_width=30, id="metrics-charts",
                    ):
                        with Vertical(classes="chart-box"):
                            yield Static(
                                "Benchmark Latencies (ns)",
                                classes="chart-label",
                            )
                            yield Sparkline([], id="metrics-sparkline")
                        with Vertical(classes="chart-box"):
                            yield Static(
                                "Test Results by Suite",
                                classes="chart-label",
                            )
                            yield Static(
                                "[dim]run tests to see results[/dim]",
                                id="metrics-suite-bars",
                            )

        yield Footer()

    # ── Mount ─────────────────────────────────────────────────

    def on_mount(self) -> None:
        tbl = self.query_one("#test-table", DataTable)
        tbl.cursor_type = "row"
        tbl.zebra_stripes = True
        tbl.add_columns("", "Test Name", "Suite", "Time")

        btbl = self.query_one("#bench-table", DataTable)
        btbl.cursor_type = "row"
        btbl.zebra_stripes = True
        btbl.add_columns("Benchmark", "Mean", "95% CI", "Bar")

        self.load_cached_data()

    def action_toggle_dark(self) -> None:
        self.theme = (
            "textual-light" if self.theme == "textual-dark" else "textual-dark"
        )

    # ── Actions ───────────────────────────────────────────────

    def action_run_tests(self) -> None:
        card = self.query_one("#card-tests", StatusCard)
        card.set_value("running...", "warn")
        self.query_one("#test-summary", Static).content = (
            "[yellow bold]\u23f3 Running cargo test...[/yellow bold]"
        )
        self.notify("\u23f3 Running cargo test...", timeout=3)
        self.do_run_tests()

    def action_run_benchmarks(self) -> None:
        card = self.query_one("#card-bench", StatusCard)
        card.set_value("running...", "warn")
        self.notify("\u23f3 Running cargo bench (full)...", timeout=3)
        self.do_run_benchmarks()

    def action_run_fuzz(self) -> None:
        card = self.query_one("#card-fuzz", StatusCard)
        card.set_value("running...", "warn")
        self.notify("\u23f3 Running fuzz targets (10s each)...", timeout=3)
        self.do_run_fuzz()

    def action_run_checks(self) -> None:
        card = self.query_one("#card-checks", StatusCard)
        card.set_value("running...", "warn")
        tabbed = self.query_one(TabbedContent)
        tabbed.active = "tab-checks"
        self.notify("\u23f3 Running quality gates...", timeout=3)
        self.do_run_checks()

    def action_check_updates(self) -> None:
        card = self.query_one("#card-update", StatusCard)
        card.set_value("checking...", "warn")
        tabbed = self.query_one(TabbedContent)
        tabbed.active = "tab-update"
        self.notify("\u21bb Checking Hyprland source...", timeout=3)
        self.do_check_updates()

    def action_show_metrics(self) -> None:
        tabbed = self.query_one(TabbedContent)
        tabbed.active = "tab-metrics"
        self._update_metrics()

    def action_refresh_all(self) -> None:
        self.notify("\u21bb Refreshing cached data...", timeout=2)
        self.load_cached_data()

    # ── Workers ───────────────────────────────────────────────

    @work(thread=True)
    def load_cached_data(self) -> None:
        benches = runners.load_benchmarks()
        if benches:
            self.app.call_from_thread(self._update_bench_table, benches)

        fuzz_targets = runners.load_fuzz_status()
        if fuzz_targets:
            self.app.call_from_thread(self._update_fuzz_display, fuzz_targets)

        git = runners.git_summary()
        self.app.call_from_thread(self._update_git_display, git)

        # Quick version check (no fetch, just compare local)
        version_file = runners.ROOT / ".sources" / ".version"
        if version_file.exists():
            ver = version_file.read_text().strip()
            card = self.query_one("#card-update", StatusCard)
            self.app.call_from_thread(card.set_value, ver, "info")

        self.app.call_from_thread(self._update_metrics)

    @work(thread=True)
    def do_run_tests(self) -> None:
        card = self.query_one("#card-tests", StatusCard)
        tbl = self.query_one("#test-table", DataTable)
        self.app.call_from_thread(tbl.clear)
        self._test_count = 0
        self._test_passed = 0
        self._test_failed = 0

        def _on_test(tr: runners.TestResult) -> None:
            if tr.status == "ok":
                icon = "[green]\u2714[/green]"
                self._test_passed += 1
            elif tr.status == "failed":
                icon = "[red]\u2718[/red]"
                self._test_failed += 1
            else:
                icon = "[yellow]\u25cb[/yellow]"
            t_str = (
                f"{tr.exec_time_s:.3f}s"
                if tr.exec_time_s > 0
                else "[dim]\u2014[/dim]"
            )
            self._test_count += 1
            self.app.call_from_thread(
                tbl.add_row, icon, tr.name, tr.suite, t_str,
            )
            p = self._test_passed
            f = self._test_failed
            total = self._test_count
            self.app.call_from_thread(
                card.set_value,
                f"{p}/{total}" + (f" ({f} FAIL)" if f else ""),
                "fail" if f else "warn",
            )

        def _on_suite(suite: runners.SuiteSummary) -> None:
            self.app.call_from_thread(self._mount_suite_card, suite)

        summary = runners.run_tests_live(
            on_test=_on_test, on_suite=_on_suite,
        )
        self.app.call_from_thread(self._finalize_tests, summary)

    @work(thread=True)
    def do_run_benchmarks(self) -> None:
        card = self.query_one("#card-bench", StatusCard)

        def _progress(name: str, done: int, total: int) -> None:
            if name:
                short = name[:16] + "\u2026" if len(name) > 16 else name
                self.app.call_from_thread(
                    card.set_value, f"[{done}] {short}", "warn"
                )

        benches = runners.run_benchmarks_live(progress_cb=_progress)
        self.app.call_from_thread(self._update_bench_table, benches)

    @work(thread=True)
    def do_run_fuzz(self) -> None:
        targets = runners.load_fuzz_status()
        log = self.query_one("#fuzz-log", RichLog)

        self.app.call_from_thread(self._update_fuzz_display, targets)

        for t in targets:
            self.app.call_from_thread(
                log.write,
                f"[bold cyan]\u25b6 Fuzzing {t.name}...[/bold cyan]",
            )
            self.app.call_from_thread(self._set_fuzz_card_running, t.name)

            def _progress(p: runners.FuzzProgress) -> None:
                self.app.call_from_thread(self._update_fuzz_card_progress, p)

            result = runners.run_fuzz_live(
                t.name, duration=10, progress_cb=_progress
            )

            # Update this card to final state immediately
            self.app.call_from_thread(
                self._finish_fuzz_card, result
            )

            icon = (
                "[green]\u2714[/green]"
                if result.crash_count == 0
                else "[red]\u2718[/red]"
            )
            self.app.call_from_thread(
                log.write,
                f"  {icon} {result.name}: "
                f"[dim]{result.corpus_count} corpus[/dim], "
                f"{'[red]' if result.crash_count else ''}"
                f"{result.crash_count} crashes"
                f"{'[/red]' if result.crash_count else ''}",
            )

        updated = runners.load_fuzz_status()
        self.app.call_from_thread(self._update_fuzz_display, updated)

    @work(thread=True)
    def do_run_checks(self) -> None:
        log = self.query_one("#checks-log", RichLog)

        # Git summary first
        git = runners.git_summary()
        self.app.call_from_thread(self._update_git_display, git)

        # Mount initial pending cards
        for name, _, _ in runners._QUALITY_CHECKS:
            self.app.call_from_thread(self._mount_check_card, name)

        def _progress(
            name: str, status: str, elapsed: float, est_time: float
        ) -> None:
            if status == "running":
                self.app.call_from_thread(
                    self._update_check_card_progress, name, elapsed, est_time
                )
            else:
                self.app.call_from_thread(
                    self._finish_check_card, name, status, elapsed
                )

        results = runners.run_quality_gates_live(progress_cb=_progress)
        self._last_checks = results

        # Log output for failed checks
        for r in results:
            if not r.passed:
                self.app.call_from_thread(
                    log.write,
                    f"\n[red bold]\u2718 {r.name} FAILED"
                    f" ({r.exec_time_s:.1f}s):[/red bold]",
                )
                for line in r.output.strip().splitlines()[-30:]:
                    self.app.call_from_thread(log.write, f"  [dim]{line}[/dim]")
            else:
                self.app.call_from_thread(
                    log.write,
                    f"[green]\u2714 {r.name}[/green]"
                    f"  [dim]{r.exec_time_s:.1f}s[/dim]",
                )

        self.app.call_from_thread(self._update_checks_status_card, results)
        self.app.call_from_thread(self._update_metrics)

    @work(thread=True)
    def do_check_updates(self) -> None:
        card = self.query_one("#card-update", StatusCard)
        header = self.query_one("#update-header", Static)
        log = self.query_one("#update-details", RichLog)
        self.app.call_from_thread(log.clear)

        def _progress(step: str) -> None:
            self.app.call_from_thread(
                header.__setattr__, "content",
                f"[yellow]\u23f3 {step}[/yellow]",
            )

        result = runners.check_hyprland_updates(progress_cb=_progress)
        self.app.call_from_thread(
            self._update_source_display, result,
        )

    # ── Source update display ────────────────────────────────

    def _update_source_display(self, u: runners.SourceUpdate) -> None:
        card = self.query_one("#card-update", StatusCard)
        header = self.query_one("#update-header", Static)
        log = self.query_one("#update-details", RichLog)

        header.remove_class("update-available", "update-current")

        if u.error:
            header.content = f"[red bold]ERROR: {u.error}[/red bold]"
            card.set_value("ERROR", "fail")
            return

        if not u.has_update:
            header.content = (
                f"[green bold]\u2714 Up to date[/green bold]\n"
                f"[dim]Hyprland source: {u.current_version}[/dim]"
            )
            header.add_class("update-current")
            card.set_value(u.current_version, "pass")
            return

        header.content = (
            f"[yellow bold]\u26a0 Update available[/yellow bold]\n"
            f"{u.current_version} [bold]\u2192[/bold] {u.latest_version}"
        )
        header.add_class("update-available")
        card.set_value(
            f"{u.current_version}\u2192{u.latest_version}", "warn",
        )

        if u.sdk_summary:
            log.write(f"[bold]SDK-relevant changes:[/bold] {u.sdk_summary}")
            log.write("")

        if u.category_stats:
            log.write("[bold]By category:[/bold]")
            for cat, stat in u.category_stats.items():
                log.write(f"  [cyan]{cat:<12}[/cyan] {stat}")
            log.write("")

        if u.new_commands:
            log.write("[green bold]New IPC commands:[/green bold]")
            for cmd in u.new_commands:
                log.write(f"  [green]+[/green] {cmd}")
            log.write("")

        if u.removed_commands:
            log.write("[red bold]Removed IPC commands:[/red bold]")
            for cmd in u.removed_commands:
                log.write(f"  [red]-[/red] {cmd}")
            log.write("")

        if u.new_events:
            log.write("[green bold]New events:[/green bold]")
            for ev in u.new_events:
                log.write(f"  [green]+[/green] {ev}")
            log.write("")

        if u.removed_events:
            log.write("[red bold]Removed events:[/red bold]")
            for ev in u.removed_events:
                log.write(f"  [red]-[/red] {ev}")
            log.write("")

        if u.new_hooks:
            log.write("[green bold]New hook events:[/green bold]")
            for h in u.new_hooks:
                log.write(f"  [green]+[/green] {h}")
            log.write("")

        if u.removed_hooks:
            log.write("[red bold]Removed hook events:[/red bold]")
            for h in u.removed_hooks:
                log.write(f"  [red]-[/red] {h}")
            log.write("")

        if u.changed_protocols:
            log.write("[yellow bold]Changed protocol XMLs:[/yellow bold]")
            for p in u.changed_protocols:
                log.write(f"  [yellow]\u2022[/yellow] {p}")
            log.write("")

        if u.api_changes:
            log.write("[red bold]Plugin API changes:[/red bold]")
            for a in u.api_changes:
                log.write(f"  {a}")
            log.write("")

        if not any([
            u.new_commands, u.removed_commands, u.new_events,
            u.removed_events, u.new_hooks, u.removed_hooks,
            u.changed_protocols, u.api_changes,
        ]):
            log.write("[dim]No SDK-breaking changes detected in diff.[/dim]")

        log.write("")
        log.write(
            f"[dim]Run [bold]./scripts/update-sources.sh {u.latest_version}[/bold]"
            f" to update[/dim]"
        )

    # ── Test table ────────────────────────────────────────────

    def _mount_suite_card(self, suite: runners.SuiteSummary) -> None:
        """Add or update a single suite card in real-time."""
        suites_row = self.query_one("#test-suites-row", ItemGrid)
        st = suite.passed + suite.failed + suite.ignored
        s_bar = _pass_fail_bar(suite.passed, suite.failed, suite.ignored, 14)
        state = "suite-pass" if suite.failed == 0 else "suite-fail"
        text = (
            f"[bold]{suite.name}[/bold]\n"
            f"{s_bar}\n"
            f"[dim]{suite.passed}/{st}  {suite.exec_time_s:.2f}s[/dim]"
        )
        card_id = f"suite-{suite.name}"
        try:
            existing = suites_row.query_one(f"#{card_id}", Static)
            existing.content = text
            existing.remove_class("suite-pass", "suite-fail")
            existing.add_class(state)
        except Exception:
            suites_row.mount(
                Static(text, classes=f"suite-card {state}", id=card_id)
            )

    def _finalize_tests(self, summary: runners.TestSummary) -> None:
        """Update summary bar and status card after all tests finish."""
        self._last_tests = summary
        card = self.query_one("#card-tests", StatusCard)
        summary_w = self.query_one("#test-summary", Static)

        if summary.error:
            summary_w.content = f"[red bold]ERROR: {summary.error}[/red bold]"
            card.set_value("ERROR", "fail")
            return

        total = summary.passed + summary.failed + summary.ignored
        pct = f"{summary.passed / total * 100:.0f}" if total > 0 else "0"
        bar = _pass_fail_bar(summary.passed, summary.failed, summary.ignored, 40)
        summary_w.content = (
            f"  {bar}  [bold]{summary.passed}[/bold]/{total} passed "
            f"({pct}%)  [dim]{summary.total_time_s:.2f}s[/dim]"
        )

        if summary.failed > 0:
            card.set_value(
                f"{summary.passed}/{total} ({summary.failed} FAILED)", "fail"
            )
        else:
            card.set_value(f"{summary.passed}/{total} passed", "pass")

        self._update_metrics()

    # ── Bench table ───────────────────────────────────────────

    def _update_bench_table(self, benches: list[runners.BenchResult]) -> None:
        self._last_benches = benches
        tbl = self.query_one("#bench-table", DataTable)
        tbl.clear()
        card = self.query_one("#card-bench", StatusCard)

        if not benches:
            card.set_value("no data", "info")
            return

        max_ns = max(b.mean_ns for b in benches)
        spark_data: list[float] = []

        for b in benches:
            bar_w = 20
            filled = int((b.mean_ns / max_ns) * bar_w) if max_ns > 0 else 0
            bar = "\u2588" * filled + "\u2591" * (bar_w - filled)

            if b.mean_ns < 1_000:
                bar = f"[green]{bar}[/green]"
            elif b.mean_ns < 100_000:
                bar = f"[cyan]{bar}[/cyan]"
            elif b.mean_ns < 1_000_000:
                bar = f"[yellow]{bar}[/yellow]"
            else:
                bar = f"[red]{bar}[/red]"

            tbl.add_row(b.name, b.mean_display, b.ci_display, bar)
            spark_data.append(b.mean_ns)

        try:
            self.query_one("#bench-sparkline", Sparkline).data = spark_data
        except Exception:
            pass

        card.set_value(f"{len(benches)} benchmarks", "pass")
        self._update_metrics()

    # ── Fuzz display ──────────────────────────────────────────

    def _update_fuzz_display(self, targets: list[runners.FuzzTarget]) -> None:
        self._last_fuzz = targets
        card = self.query_one("#card-fuzz", StatusCard)
        container = self.query_one("#fuzz-targets", VerticalScroll)
        crashes = 0

        for t in targets:
            if t.crash_count > 0:
                icon = "[red]\u2718[/red]"
                status = "[red bold]CRASH[/red bold]"
                bar = f"[red]{'█' * 36}[/red]"
                state_cls = "fuzz-crash"
            elif t.has_run:
                icon = "[green]\u2714[/green]"
                status = "[green]clean[/green]"
                bar = f"[green]{'●' * 36}[/green]"
                state_cls = "fuzz-clean"
            else:
                icon = "[dim]\u25cb[/dim]"
                status = "[dim]not run[/dim]"
                bar = f"[dim]{'·' * 36}[/dim]"
                state_cls = ""

            corpus = (
                str(t.corpus_count) if t.corpus_count > 0 else "[dim]0[/dim]"
            )
            crash_d = (
                f"[red bold]{t.crash_count}[/red bold]"
                if t.crash_count > 0
                else "[dim]0[/dim]"
            )

            text = (
                f"{icon} [bold]{t.name}[/bold]   {status}"
                f"   corpus: {corpus}   crashes: {crash_d}\n"
                f"{bar}"
            )

            widget_id = f"fuzz-{t.name}"
            try:
                existing = container.query_one(f"#{widget_id}", Static)
                existing.content = text
                existing.remove_class("fuzz-clean", "fuzz-crash", "fuzz-running")
                if state_cls:
                    existing.add_class(state_cls)
            except Exception:
                cls = f"fuzz-card {state_cls}".strip()
                container.mount(Static(text, classes=cls, id=widget_id))

            crashes += t.crash_count

        if crashes > 0:
            card.set_value(f"{len(targets)} targets, {crashes} CRASHES", "fail")
        elif any(t.has_run for t in targets):
            card.set_value(f"{len(targets)} targets, all clean", "pass")
        else:
            card.set_value(f"{len(targets)} targets", "info")

        self._update_metrics()

    def _set_fuzz_card_running(self, name: str) -> None:
        """Mark a fuzz target card as actively running."""
        try:
            card = self.query_one(f"#fuzz-{name}", Static)
            card.remove_class("fuzz-clean", "fuzz-crash")
            card.add_class("fuzz-running")
            bar = _pacman(0, 10, 36)
            card.content = (
                f"[yellow]\u23f3[/yellow] [bold]{name}[/bold]"
                f"   [yellow]running[/yellow]\n"
                f"{bar}  0s / 10s"
            )
        except Exception:
            pass

    def _update_fuzz_card_progress(self, p: runners.FuzzProgress) -> None:
        """Update fuzz card with real metrics from libfuzzer stderr."""
        try:
            card = self.query_one(f"#fuzz-{p.target}", Static)
            bar = _pacman(p.elapsed, p.duration, 36)
            metrics = ""
            if p.iterations > 0:
                metrics = (
                    f"  [dim]#{p.iterations:,}"
                    f"  cov:{p.coverage}"
                    f"  {p.exec_per_sec:,} exec/s[/dim]"
                )
            card.content = (
                f"[yellow]\u23f3[/yellow] [bold]{p.target}[/bold]"
                f"   [yellow]running[/yellow]{metrics}\n"
                f"{bar}  {p.elapsed:.0f}s / {p.duration:.0f}s"
            )
        except Exception:
            pass

    def _finish_fuzz_card(self, result: runners.FuzzTarget) -> None:
        """Update a fuzz card to its final state after the run completes."""
        try:
            card = self.query_one(f"#fuzz-{result.name}", Static)
            card.remove_class("fuzz-running", "fuzz-clean", "fuzz-crash")
            if result.crash_count > 0:
                card.add_class("fuzz-crash")
                icon = "[red]\u2718[/red]"
                status = "[red bold]CRASH[/red bold]"
                bar = f"[red]{'█' * 36}[/red]"
            else:
                card.add_class("fuzz-clean")
                icon = "[green]\u2714[/green]"
                status = "[green]clean[/green]"
                bar = f"[green]{'●' * 36}[/green]"
            corpus = str(result.corpus_count) if result.corpus_count > 0 else "[dim]0[/dim]"
            crash_d = (
                f"[red bold]{result.crash_count}[/red bold]"
                if result.crash_count > 0
                else "[dim]0[/dim]"
            )
            card.content = (
                f"{icon} [bold]{result.name}[/bold]   {status}"
                f"   corpus: {corpus}   crashes: {crash_d}\n"
                f"{bar}"
            )
        except Exception:
            pass

    # ── Checks display ────────────────────────────────────────

    def _mount_check_card(self, name: str) -> None:
        """Mount or reset a check card to pending state."""
        container = self.query_one("#checks-gates", VerticalScroll)
        card_id = f"chk-{name.replace(' ', '-')}"
        text = (
            f"[dim]\u25cb[/dim] [bold]{name}[/bold]"
            f"   [dim]pending[/dim]\n"
            f"[dim]{'·' * 36}[/dim]"
        )
        try:
            existing = container.query_one(f"#{card_id}", Static)
            existing.content = text
            existing.remove_class("check-pass", "check-fail", "check-running")
        except Exception:
            container.mount(Static(text, classes="check-card", id=card_id))

    def _update_check_card_progress(
        self, name: str, elapsed: float, est_time: float
    ) -> None:
        """Pacman animation on a running check."""
        card_id = f"chk-{name.replace(' ', '-')}"
        try:
            card = self.query_one(f"#{card_id}", Static)
            card.remove_class("check-pass", "check-fail")
            card.add_class("check-running")
            bar = _pacman(elapsed, est_time, 36)
            card.content = (
                f"[yellow]\u23f3[/yellow] [bold]{name}[/bold]"
                f"   [yellow]running[/yellow]\n"
                f"{bar}  {elapsed:.0f}s"
            )
        except Exception:
            pass

    def _finish_check_card(
        self, name: str, status: str, elapsed: float
    ) -> None:
        """Mark a check card as passed or failed."""
        card_id = f"chk-{name.replace(' ', '-')}"
        try:
            card = self.query_one(f"#{card_id}", Static)
            card.remove_class("check-running", "check-pass", "check-fail")
            if status == "passed":
                card.add_class("check-pass")
                icon = "[green]\u2714[/green]"
                s = "[green]passed[/green]"
                bar = f"[green]{'●' * 36}[/green]"
            else:
                card.add_class("check-fail")
                icon = "[red]\u2718[/red]"
                s = "[red bold]FAILED[/red bold]"
                bar = f"[red]{'█' * 36}[/red]"
            card.content = (
                f"{icon} [bold]{name}[/bold]   {s}   {elapsed:.1f}s\n"
                f"{bar}"
            )
        except Exception:
            pass

    def _update_checks_status_card(
        self, results: list[runners.CheckResult]
    ) -> None:
        card = self.query_one("#card-checks", StatusCard)
        passed = sum(1 for r in results if r.passed)
        total = len(results)
        if passed == total:
            card.set_value(f"{passed}/{total} passed", "pass")
        else:
            card.set_value(
                f"{passed}/{total} ({total - passed} FAILED)", "fail"
            )

    def _update_git_display(self, git: runners.GitSummary) -> None:
        self._last_git = git
        widget = self.query_one("#checks-git", Static)

        if git.files_changed == 0 and not git.recent_commits:
            widget.content = (
                f"[bold]branch:[/bold] {git.branch}   "
                f"[dim]no uncommitted changes[/dim]"
            )
            return

        lines: list[str] = [
            f"[bold]branch:[/bold] {git.branch}   "
            f"[green]+{git.insertions}[/green]  "
            f"[red]-{git.deletions}[/red]  "
            f"[dim]{git.files_changed} files[/dim]",
        ]

        if git.file_stats:
            lines.append("")
            max_ch = (
                max((a + d) for _, a, d in git.file_stats)
                if git.file_stats
                else 1
            )
            for fname, add, rem in sorted(
                git.file_stats, key=lambda x: x[1] + x[2], reverse=True
            )[:15]:
                bw = 20
                aw = int((add / max_ch) * bw) if max_ch > 0 else 0
                rw = int((rem / max_ch) * bw) if max_ch > 0 else 0
                rest = bw - aw - rw
                bar = (
                    f"[green]{'█' * aw}[/green]"
                    f"[red]{'█' * rw}[/red]"
                    f"[dim]{'░' * rest}[/dim]"
                )
                lines.append(
                    f"  {fname:<40} "
                    f"[green]+{add:<4}[/green] "
                    f"[red]-{rem:<4}[/red] "
                    f"{bar}"
                )
            if len(git.file_stats) > 15:
                lines.append(
                    f"  [dim]... and {len(git.file_stats) - 15} more[/dim]"
                )

        if git.recent_commits:
            lines.append("")
            lines.append("[bold]recent commits:[/bold]")
            for c in git.recent_commits[:6]:
                sha, _, msg = c.partition(" ")
                lines.append(f"  [cyan]{sha}[/cyan] {msg}")

        widget.content = "\n".join(lines)

    # ── Metrics ───────────────────────────────────────────────

    def _update_metrics(self) -> None:
        """Refresh the metrics tab with all cached data."""
        # Tests
        if self._last_tests:
            total = (
                self._last_tests.passed
                + self._last_tests.failed
                + self._last_tests.ignored
            )
            pct = (
                int(self._last_tests.passed / total * 100) if total > 0 else 0
            )
            self._try_update("#m-tests", str(total))
            self._try_update("#m-pass-rate", str(pct))

            lines: list[str] = []
            for s in sorted(
                self._last_tests.suites,
                key=lambda x: x.passed + x.failed + x.ignored,
                reverse=True,
            ):
                st = s.passed + s.failed + s.ignored
                bar = _pass_fail_bar(s.passed, s.failed, s.ignored, 24)
                lines.append(f"  {s.name:<16} {bar}  {s.passed}/{st}")
            try:
                self.query_one("#metrics-suite-bars", Static).content = (
                    "\n".join(lines) if lines else "[dim]no suite data[/dim]"
                )
            except Exception:
                pass
        else:
            self._try_update("#m-tests", "0")
            self._try_update("#m-pass-rate", "0")

        # Benchmarks
        if self._last_benches:
            self._try_update("#m-benches", str(len(self._last_benches)))
            try:
                self.query_one("#metrics-sparkline", Sparkline).data = [
                    b.mean_ns for b in self._last_benches
                ]
            except Exception:
                pass
        else:
            self._try_update("#m-benches", "0")

        # Fuzz
        if self._last_fuzz:
            clean = sum(
                1
                for t in self._last_fuzz
                if t.has_run and t.crash_count == 0
            )
            self._try_update("#m-fuzz", str(clean))
        else:
            self._try_update("#m-fuzz", "0")

    def _try_update(self, selector: str, value: str) -> None:
        try:
            self.query_one(selector, Digits).update(value)
        except Exception:
            pass


def main() -> None:
    app = HyprTui()
    app.run()


if __name__ == "__main__":
    main()
