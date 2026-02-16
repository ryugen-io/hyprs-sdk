"""hypr-tui: TUI dashboard for hypr-sdk tests, benchmarks, and fuzz results."""

from __future__ import annotations

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import (
    DataTable, Footer, Header, Static, TabbedContent, TabPane, RichLog,
)
from textual import work

from . import runners

# -- Styles ----------------------------------------------------------------

CSS = """
Screen {
    background: $surface;
}

#summary-bar {
    height: 3;
    layout: horizontal;
    padding: 0 1;
    background: $primary-background;
}

.summary-cell {
    width: 1fr;
    content-align: center middle;
    text-style: bold;
}

.pass { color: $success; }
.fail { color: $error; }
.warn { color: $warning; }
.info { color: $primary; }

DataTable {
    height: 1fr;
}

#bench-table {
    height: 1fr;
}

#fuzz-table {
    height: 1fr;
}

#test-table {
    height: 1fr;
}

RichLog {
    height: 1fr;
    border: solid $primary;
    padding: 0 1;
}

TabPane {
    padding: 0;
}
"""


class HyprTui(App):
    """TUI dashboard for hypr-sdk quality metrics."""

    CSS = CSS
    TITLE = "hypr-sdk dashboard"
    BINDINGS = [
        ("t", "run_tests", "Run Tests"),
        ("b", "run_benchmarks", "Run Benchmarks"),
        ("f", "run_fuzz", "Run Fuzz (10s)"),
        ("r", "refresh_all", "Refresh All"),
        ("q", "quit", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal(id="summary-bar"):
            yield Static("Tests: --", id="sum-tests", classes="summary-cell info")
            yield Static("Benchmarks: --", id="sum-bench", classes="summary-cell info")
            yield Static("Fuzz: --", id="sum-fuzz", classes="summary-cell info")

        with TabbedContent():
            with TabPane("Tests", id="tab-tests"):
                yield DataTable(id="test-table")

            with TabPane("Benchmarks", id="tab-bench"):
                yield DataTable(id="bench-table")

            with TabPane("Fuzz", id="tab-fuzz"):
                with Vertical():
                    yield DataTable(id="fuzz-table")
                    yield RichLog(id="fuzz-log", markup=True)

        yield Footer()

    def on_mount(self) -> None:
        # Set up test table
        test_tbl = self.query_one("#test-table", DataTable)
        test_tbl.cursor_type = "row"
        test_tbl.zebra_stripes = True
        test_tbl.add_columns("Status", "Test Name")

        # Set up bench table
        bench_tbl = self.query_one("#bench-table", DataTable)
        bench_tbl.cursor_type = "row"
        bench_tbl.zebra_stripes = True
        bench_tbl.add_columns("Benchmark", "Mean", "95% CI")

        # Set up fuzz table
        fuzz_tbl = self.query_one("#fuzz-table", DataTable)
        fuzz_tbl.cursor_type = "row"
        fuzz_tbl.zebra_stripes = True
        fuzz_tbl.add_columns("Target", "Status", "Corpus", "Crashes")

        # Load cached data on startup
        self.load_cached_data()

    @work(thread=True)
    def load_cached_data(self) -> None:
        """Load already-available benchmark and fuzz data without running anything."""
        benches = runners.load_benchmarks()
        if benches:
            self.app.call_from_thread(self._update_bench_table, benches)

        fuzz_targets = runners.load_fuzz_status()
        if fuzz_targets:
            self.app.call_from_thread(self._update_fuzz_table, fuzz_targets)

    # -- Actions ---------------------------------------------------------------

    def action_run_tests(self) -> None:
        self.notify("Running tests...", timeout=2)
        self.do_run_tests()

    def action_run_benchmarks(self) -> None:
        self.notify("Running benchmarks (quick mode)...", timeout=2)
        self.do_run_benchmarks()

    def action_run_fuzz(self) -> None:
        self.notify("Running all fuzz targets (10s each)...", timeout=2)
        self.do_run_fuzz()

    def action_refresh_all(self) -> None:
        self.notify("Refreshing all...", timeout=2)
        self.load_cached_data()

    # -- Workers ---------------------------------------------------------------

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
                log.write, f"[bold]Fuzzing {t.name}...[/bold]"
            )
            result = runners.run_fuzz(t.name, duration=10)
            self.app.call_from_thread(
                log.write,
                f"  {result.name}: {result.corpus_count} corpus, {result.crash_count} crashes"
            )

        updated = runners.load_fuzz_status()
        self.app.call_from_thread(self._update_fuzz_table, updated)

    # -- Table updates ---------------------------------------------------------

    def _update_test_table(self, summary: runners.TestSummary) -> None:
        tbl = self.query_one("#test-table", DataTable)
        tbl.clear()

        if summary.error:
            tbl.add_row("ERR", summary.error)
            self.query_one("#sum-tests", Static).content = "Tests: ERROR"
            self.query_one("#sum-tests").remove_class("pass")
            self.query_one("#sum-tests").add_class("fail")
            return

        for r in sorted(summary.results, key=lambda x: (x.status != "failed", x.name)):
            icon = "[green]PASS[/green]" if r.status == "ok" else "[red]FAIL[/red]"
            if r.status == "ignored":
                icon = "[yellow]SKIP[/yellow]"
            tbl.add_row(icon, r.name)

        total = summary.passed + summary.failed + summary.ignored
        label = f"Tests: {summary.passed}/{total} passed"
        sum_widget = self.query_one("#sum-tests", Static)
        sum_widget.content = label
        sum_widget.remove_class("pass", "fail", "warn", "info")
        if summary.failed > 0:
            label += f" ({summary.failed} FAILED)"
            sum_widget.content = label
            sum_widget.add_class("fail")
        else:
            sum_widget.add_class("pass")

    def _update_bench_table(self, benches: list[runners.BenchResult]) -> None:
        tbl = self.query_one("#bench-table", DataTable)
        tbl.clear()
        for b in benches:
            tbl.add_row(b.name, b.mean_display, b.ci_display)

        self.query_one("#sum-bench", Static).content = f"Benchmarks: {len(benches)} results"
        sum_widget = self.query_one("#sum-bench", Static)
        sum_widget.remove_class("info")
        sum_widget.add_class("pass")

    def _update_fuzz_table(self, targets: list[runners.FuzzTarget]) -> None:
        tbl = self.query_one("#fuzz-table", DataTable)
        tbl.clear()
        crashes = 0
        for t in targets:
            status = "[green]OK[/green]" if t.has_run and t.crash_count == 0 else (
                "[red]CRASH[/red]" if t.crash_count > 0 else "[dim]not run[/dim]"
            )
            tbl.add_row(t.name, status, str(t.corpus_count), str(t.crash_count))
            crashes += t.crash_count

        sum_widget = self.query_one("#sum-fuzz", Static)
        sum_widget.content = f"Fuzz: {len(targets)} targets, {crashes} crashes"
        sum_widget.remove_class("pass", "fail", "info")
        if crashes > 0:
            sum_widget.add_class("fail")
        elif any(t.has_run for t in targets):
            sum_widget.add_class("pass")
        else:
            sum_widget.add_class("info")


def main() -> None:
    app = HyprTui()
    app.run()


if __name__ == "__main__":
    main()
