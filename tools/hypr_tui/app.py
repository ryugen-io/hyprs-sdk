"""hypr-tui: TUI dashboard for hypr-sdk tests, benchmarks, and fuzz results."""

from __future__ import annotations

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical, VerticalScroll, Center
from textual.widgets import (
    DataTable, Footer, Header, Static, TabbedContent, TabPane,
    RichLog, Rule, Sparkline, Label, ProgressBar, Digits,
    LoadingIndicator,
)
from textual import work
from textual.reactive import reactive

from . import runners

# -- Styles ----------------------------------------------------------------

CSS = """
Screen {
    background: $surface;
}

/* ── Status cards row ──────────────────────────────────────────── */

#status-row {
    height: 5;
    layout: horizontal;
    padding: 0 1;
    margin: 0 0 0 0;
}

.status-card {
    width: 1fr;
    height: 5;
    margin: 0 1 0 0;
    padding: 0 2;
    border: tall $primary-darken-2;
    content-align: center middle;
}

.status-card:last-of-type {
    margin-right: 0;
}

.status-card.ok {
    border: tall $success;
}

.status-card.err {
    border: tall $error;
}

.status-card.pending {
    border: tall $primary-darken-2;
}

.card-icon {
    width: 4;
    height: 3;
    content-align: center middle;
    text-style: bold;
}

.card-body {
    width: 1fr;
    height: 3;
    padding: 0 0 0 1;
}

.card-title {
    text-style: bold;
    color: $text-muted;
    text-opacity: 70%;
}

.card-value {
    text-style: bold;
}

.card-value.pass { color: $success; }
.card-value.fail { color: $error; }
.card-value.warn { color: $warning; }
.card-value.info { color: $text; }

/* ── Tabs ──────────────────────────────────────────────────────── */

TabbedContent {
    height: 1fr;
}

TabPane {
    padding: 0;
}

ContentSwitcher {
    height: 1fr;
}

/* ── Tests tab ─────────────────────────────────────────────────── */

#test-table {
    height: 1fr;
}

#test-table > .datatable--header {
    text-style: bold;
    background: $primary-background;
}

/* ── Benchmarks tab ────────────────────────────────────────────── */

#bench-layout {
    height: 1fr;
}

#bench-table {
    height: 1fr;
}

#bench-sparkline-box {
    height: 8;
    border: tall $primary-darken-2;
    margin: 0 0 0 0;
    padding: 0 1;
}

#bench-sparkline-label {
    height: 1;
    color: $text-muted;
    text-style: italic;
    content-align: center middle;
}

Sparkline {
    height: 5;
    margin: 0;
}

Sparkline > .sparkline--max-color {
    color: $warning;
}

Sparkline > .sparkline--min-color {
    color: $success;
}

/* ── Fuzz tab ──────────────────────────────────────────────────── */

#fuzz-layout {
    height: 1fr;
}

#fuzz-table {
    height: auto;
    max-height: 12;
}

#fuzz-log {
    height: 1fr;
    border: tall $primary-darken-2;
    padding: 0 1;
    margin: 1 0 0 0;
}

/* ── DataTable global ──────────────────────────────────────────── */

DataTable {
    scrollbar-size: 1 1;
}

DataTable > .datatable--header {
    text-style: bold;
    background: $primary-background;
}

/* ── Loading overlay ───────────────────────────────────────────── */

#loading-overlay {
    display: none;
    layer: overlay;
    width: 100%;
    height: 100%;
    background: $surface 80%;
    content-align: center middle;
}

#loading-overlay.visible {
    display: block;
}

#loading-msg {
    width: auto;
    height: auto;
    padding: 1 3;
    background: $panel;
    border: tall $primary;
    content-align: center middle;
    text-style: bold;
}
"""


# -- Status card widget ----------------------------------------------------

class StatusCard(Static):
    """A compact status indicator card."""

    def __init__(self, icon: str, title: str, card_id: str) -> None:
        super().__init__(id=card_id, classes="status-card pending")
        self._icon = icon
        self._title = title
        self._value = "--"
        self._value_class = "info"

    def compose(self) -> ComposeResult:
        with Horizontal():
            yield Static(self._icon, classes="card-icon")
            with Vertical(classes="card-body"):
                yield Static(self._title, classes="card-title")
                yield Static(self._value, id=f"{self.id}-val", classes="card-value info")

    def set_value(self, value: str, status: str = "info") -> None:
        self._value = value
        self._value_class = status
        try:
            val_widget = self.query_one(f"#{self.id}-val", Static)
            val_widget.content = value
            val_widget.remove_class("pass", "fail", "warn", "info")
            val_widget.add_class(status)
        except Exception:
            pass

        self.remove_class("ok", "err", "pending")
        if status == "pass":
            self.add_class("ok")
        elif status == "fail":
            self.add_class("err")
        else:
            self.add_class("pending")


# -- Main app --------------------------------------------------------------

class HyprTui(App):
    """TUI dashboard for hypr-sdk quality metrics."""

    CSS = CSS
    TITLE = "hypr-sdk"
    SUB_TITLE = "quality dashboard"
    BINDINGS = [
        ("t", "run_tests", "Tests"),
        ("b", "run_benchmarks", "Bench"),
        ("f", "run_fuzz", "Fuzz (10s)"),
        ("r", "refresh_all", "Refresh"),
        ("d", "toggle_dark", "Dark/Light"),
        ("q", "quit", "Quit"),
    ]

    bench_data: reactive[list[float]] = reactive(list, recompose=False)

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)

        with Horizontal(id="status-row"):
            yield StatusCard("\u2714", "TESTS", "card-tests")
            yield StatusCard("\u26a1", "BENCHMARKS", "card-bench")
            yield StatusCard("\U0001f50d", "FUZZ", "card-fuzz")

        with TabbedContent():
            with TabPane("Tests [bold dim]t[/]", id="tab-tests"):
                yield DataTable(id="test-table")

            with TabPane("Benchmarks [bold dim]b[/]", id="tab-bench"):
                with Vertical(id="bench-layout"):
                    with Vertical(id="bench-sparkline-box"):
                        yield Static(
                            "benchmark latency distribution (ns)",
                            id="bench-sparkline-label",
                        )
                        yield Sparkline([], id="bench-sparkline")
                    yield DataTable(id="bench-table")

            with TabPane("Fuzz [bold dim]f[/]", id="tab-fuzz"):
                with Vertical(id="fuzz-layout"):
                    yield DataTable(id="fuzz-table")
                    yield RichLog(id="fuzz-log", markup=True)

        yield Footer()

    def on_mount(self) -> None:
        # Test table
        test_tbl = self.query_one("#test-table", DataTable)
        test_tbl.cursor_type = "row"
        test_tbl.zebra_stripes = True
        test_tbl.add_columns("", "Test Name", "Suite")

        # Bench table
        bench_tbl = self.query_one("#bench-table", DataTable)
        bench_tbl.cursor_type = "row"
        bench_tbl.zebra_stripes = True
        bench_tbl.add_columns("Benchmark", "Mean", "95% CI", "Bar")

        # Fuzz table
        fuzz_tbl = self.query_one("#fuzz-table", DataTable)
        fuzz_tbl.cursor_type = "row"
        fuzz_tbl.zebra_stripes = True
        fuzz_tbl.add_columns("", "Target", "Status", "Corpus", "Crashes")

        self.load_cached_data()

    def action_toggle_dark(self) -> None:
        self.theme = "textual-light" if self.theme == "textual-dark" else "textual-dark"

    # -- Actions ---------------------------------------------------------------

    def action_run_tests(self) -> None:
        card = self.query_one("#card-tests", StatusCard)
        card.set_value("running...", "warn")
        self.notify("\u23f3 Running cargo test...", timeout=3)
        self.do_run_tests()

    def action_run_benchmarks(self) -> None:
        card = self.query_one("#card-bench", StatusCard)
        card.set_value("running...", "warn")
        self.notify("\u23f3 Running cargo bench --quick...", timeout=3)
        self.do_run_benchmarks()

    def action_run_fuzz(self) -> None:
        card = self.query_one("#card-fuzz", StatusCard)
        card.set_value("running...", "warn")
        self.notify("\u23f3 Running fuzz targets (10s each)...", timeout=3)
        self.do_run_fuzz()

    def action_refresh_all(self) -> None:
        self.notify("\u21bb Refreshing cached data...", timeout=2)
        self.load_cached_data()

    # -- Workers ---------------------------------------------------------------

    @work(thread=True)
    def load_cached_data(self) -> None:
        benches = runners.load_benchmarks()
        if benches:
            self.app.call_from_thread(self._update_bench_table, benches)

        fuzz_targets = runners.load_fuzz_status()
        if fuzz_targets:
            self.app.call_from_thread(self._update_fuzz_table, fuzz_targets)

    @work(thread=True)
    def do_run_tests(self) -> None:
        summary = runners.run_tests()
        self.app.call_from_thread(self._update_test_table, summary)

    @work(thread=True)
    def do_run_benchmarks(self) -> None:
        benches = runners.run_benchmarks()
        self.app.call_from_thread(self._update_bench_table, benches)

    @work(thread=True)
    def do_run_fuzz(self) -> None:
        targets = runners.load_fuzz_status()
        log = self.query_one("#fuzz-log", RichLog)
        for t in targets:
            self.app.call_from_thread(
                log.write, f"[bold cyan]\u25b6 Fuzzing {t.name}...[/bold cyan]"
            )
            result = runners.run_fuzz(t.name, duration=10)
            icon = "[green]\u2714[/green]" if result.crash_count == 0 else "[red]\u2718[/red]"
            self.app.call_from_thread(
                log.write,
                f"  {icon} {result.name}: "
                f"[dim]{result.corpus_count} corpus[/dim], "
                f"{'[red]' if result.crash_count else ''}{result.crash_count} crashes"
                f"{'[/red]' if result.crash_count else ''}"
            )

        updated = runners.load_fuzz_status()
        self.app.call_from_thread(self._update_fuzz_table, updated)

    # -- Table updates ---------------------------------------------------------

    def _update_test_table(self, summary: runners.TestSummary) -> None:
        tbl = self.query_one("#test-table", DataTable)
        tbl.clear()
        card = self.query_one("#card-tests", StatusCard)

        if summary.error:
            tbl.add_row("\u2718", summary.error, "")
            card.set_value("ERROR", "fail")
            return

        for r in sorted(summary.results, key=lambda x: (x.status != "failed", x.name)):
            if r.status == "ok":
                icon = "[green]\u2714[/green]"
            elif r.status == "failed":
                icon = "[red]\u2718[/red]"
            else:
                icon = "[yellow]\u25cb[/yellow]"
            tbl.add_row(icon, r.name, r.suite)

        total = summary.passed + summary.failed + summary.ignored
        if summary.failed > 0:
            card.set_value(f"{summary.passed}/{total} ({summary.failed} FAILED)", "fail")
        else:
            card.set_value(f"{summary.passed}/{total} passed", "pass")

    def _update_bench_table(self, benches: list[runners.BenchResult]) -> None:
        tbl = self.query_one("#bench-table", DataTable)
        tbl.clear()
        card = self.query_one("#card-bench", StatusCard)

        if not benches:
            card.set_value("no data", "info")
            return

        max_ns = max(b.mean_ns for b in benches)
        spark_data: list[float] = []

        for b in benches:
            # Visual bar proportional to max
            bar_width = 20
            filled = int((b.mean_ns / max_ns) * bar_width) if max_ns > 0 else 0
            bar = "\u2588" * filled + "\u2591" * (bar_width - filled)

            # Color the bar based on magnitude
            if b.mean_ns < 100:
                bar = f"[green]{bar}[/green]"
            elif b.mean_ns < 1000:
                bar = f"[cyan]{bar}[/cyan]"
            elif b.mean_ns < 10000:
                bar = f"[yellow]{bar}[/yellow]"
            else:
                bar = f"[red]{bar}[/red]"

            tbl.add_row(b.name, b.mean_display, b.ci_display, bar)
            spark_data.append(b.mean_ns)

        # Update sparkline
        try:
            sparkline = self.query_one("#bench-sparkline", Sparkline)
            sparkline.data = spark_data
        except Exception:
            pass

        card.set_value(f"{len(benches)} benchmarks", "pass")

    def _update_fuzz_table(self, targets: list[runners.FuzzTarget]) -> None:
        tbl = self.query_one("#fuzz-table", DataTable)
        tbl.clear()
        card = self.query_one("#card-fuzz", StatusCard)
        crashes = 0

        for t in targets:
            if t.crash_count > 0:
                icon = "[red]\u2718[/red]"
                status = "[red bold]CRASH[/red bold]"
            elif t.has_run:
                icon = "[green]\u2714[/green]"
                status = "[green]clean[/green]"
            else:
                icon = "[dim]\u25cb[/dim]"
                status = "[dim]not run[/dim]"

            corpus_display = str(t.corpus_count) if t.corpus_count > 0 else "[dim]0[/dim]"
            crash_display = (
                f"[red bold]{t.crash_count}[/red bold]"
                if t.crash_count > 0
                else "[dim]0[/dim]"
            )

            tbl.add_row(icon, t.name, status, corpus_display, crash_display)
            crashes += t.crash_count

        if crashes > 0:
            card.set_value(f"{len(targets)} targets, {crashes} CRASHES", "fail")
        elif any(t.has_run for t in targets):
            card.set_value(f"{len(targets)} targets, all clean", "pass")
        else:
            card.set_value(f"{len(targets)} targets", "info")


def main() -> None:
    app = HyprTui()
    app.run()


if __name__ == "__main__":
    main()
