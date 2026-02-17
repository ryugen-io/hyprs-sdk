"""Run cargo test, bench, and fuzz commands and parse their output."""

from __future__ import annotations

import json
import re
import subprocess
import time
import os
from dataclasses import dataclass, field
from pathlib import Path


def _project_root() -> Path:
    """Walk up from this file to find Cargo.toml."""
    p = Path(__file__).resolve().parent.parent.parent
    if (p / "Cargo.toml").exists():
        return p
    # Fallback: CWD
    return Path.cwd()


ROOT = _project_root()


# -- Test results -----------------------------------------------------------


@dataclass
class TestResult:
    name: str
    status: str  # "ok", "failed", "ignored"
    suite: str = ""
    exec_time_s: float = 0.0


@dataclass
class SuiteSummary:
    name: str
    passed: int = 0
    failed: int = 0
    ignored: int = 0
    exec_time_s: float = 0.0
    tests: list[TestResult] = field(default_factory=list)


@dataclass
class TestSummary:
    passed: int = 0
    failed: int = 0
    ignored: int = 0
    results: list[TestResult] = field(default_factory=list)
    suites: list[SuiteSummary] = field(default_factory=list)
    error: str = ""
    total_time_s: float = 0.0


def run_tests() -> TestSummary:
    """Run cargo test and return parsed results (batch, no live updates)."""
    return run_tests_live()


def run_tests_live(
    on_test: object = None,
    on_suite: object = None,
) -> TestSummary:
    """Run cargo test with real-time callbacks per test and suite.

    ``on_test(result: TestResult)`` fires when a single test finishes.
    ``on_suite(suite: SuiteSummary)`` fires when a suite finishes.
    Returns the complete :class:`TestSummary` at the end.
    """
    import select as _select

    summary = TestSummary()

    try:
        proc = subprocess.Popen(
            [
                "cargo", "test", "--features", "wayland,blocking",
                "--", "--format", "json", "-Z", "unstable-options",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=ROOT,
            text=True,
            bufsize=1,
        )

        # Parse stderr for suite names ("Running tests/foo.rs (path)")
        # We read stderr in a background thread to avoid deadlocks
        import threading

        suite_names: list[str] = []
        stderr_lines: list[str] = []

        def _drain_stderr() -> None:
            for line in proc.stderr:
                stderr_lines.append(line)
                m = re.search(r"Running (?:unittests )?(.*?) \(", line)
                if m:
                    raw = m.group(1).strip()
                    suite_names.append(Path(raw).stem)

        t = threading.Thread(target=_drain_stderr, daemon=True)
        t.start()

        suite_idx = -1
        current_suite: SuiteSummary | None = None
        pending_tests: list[TestResult] = []

        for raw_line in proc.stdout:
            line = raw_line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue

            if obj.get("type") == "suite" and obj.get("event") == "started":
                suite_idx += 1
                sname = (
                    suite_names[suite_idx]
                    if suite_idx < len(suite_names)
                    else f"suite_{suite_idx}"
                )
                current_suite = SuiteSummary(name=sname)
                pending_tests = []

            elif obj.get("type") == "test" and "event" in obj:
                if obj["event"] == "started":
                    continue
                name = obj.get("name", "?")
                status = obj["event"]
                sname = current_suite.name if current_suite else ""
                et = obj.get("exec_time", 0.0)
                tr = TestResult(
                    name=name, status=status, suite=sname, exec_time_s=et,
                )
                pending_tests.append(tr)
                summary.results.append(tr)
                if on_test:
                    on_test(tr)

            elif obj.get("type") == "suite" and obj.get("event") in (
                "ok", "failed",
            ):
                p = obj.get("passed", 0)
                f = obj.get("failed", 0)
                ig = obj.get("ignored", 0)
                et = obj.get("exec_time", 0.0)
                summary.passed += p
                summary.failed += f
                summary.ignored += ig
                summary.total_time_s += et
                if current_suite:
                    current_suite.passed = p
                    current_suite.failed = f
                    current_suite.ignored = ig
                    current_suite.exec_time_s = et
                    current_suite.tests = pending_tests
                    if p + f + ig > 0:
                        summary.suites.append(current_suite)
                        if on_suite:
                            on_suite(current_suite)

        proc.wait(timeout=10)
        t.join(timeout=5)

    except subprocess.TimeoutExpired:
        summary.error = "Test run timed out (>120s)"
    except FileNotFoundError:
        summary.error = "cargo not found"
    return summary


# -- Benchmark results -------------------------------------------------------


@dataclass
class BenchResult:
    name: str
    mean_ns: float
    lower_ns: float
    upper_ns: float

    @property
    def mean_display(self) -> str:
        if self.mean_ns >= 1_000_000:
            return f"{self.mean_ns / 1_000_000:.2f} ms"
        if self.mean_ns >= 1_000:
            return f"{self.mean_ns / 1_000:.2f} us"
        return f"{self.mean_ns:.1f} ns"

    @property
    def ci_display(self) -> str:
        def fmt(v: float) -> str:
            if v >= 1_000_000:
                return f"{v / 1_000_000:.2f} ms"
            if v >= 1_000:
                return f"{v / 1_000:.2f} us"
            return f"{v:.1f} ns"
        return f"[{fmt(self.lower_ns)}, {fmt(self.upper_ns)}]"


def load_benchmarks() -> list[BenchResult]:
    """Read criterion results from target/criterion/."""
    criterion_dir = ROOT / "target" / "criterion"
    results: list[BenchResult] = []

    if not criterion_dir.exists():
        return results

    for bench_dir in sorted(criterion_dir.iterdir()):
        if not bench_dir.is_dir():
            continue
        # Group benchmarks have sub-directories
        estimates = bench_dir / "new" / "estimates.json"
        if estimates.exists():
            _parse_estimate(bench_dir.name, estimates, results)
        else:
            # Check for sub-benchmarks (grouped)
            for sub in sorted(bench_dir.iterdir()):
                est = sub / "new" / "estimates.json"
                if est.exists():
                    _parse_estimate(f"{bench_dir.name}/{sub.name}", est, results)

    return results


def _parse_estimate(name: str, path: Path, out: list[BenchResult]) -> None:
    try:
        data = json.loads(path.read_text())
        mean = data["mean"]["point_estimate"]
        lower = data["mean"]["confidence_interval"]["lower_bound"]
        upper = data["mean"]["confidence_interval"]["upper_bound"]
        out.append(BenchResult(name=name, mean_ns=mean, lower_ns=lower, upper_ns=upper))
    except (json.JSONDecodeError, KeyError):
        pass


def run_benchmarks() -> list[BenchResult]:
    """Run cargo bench (full run) and return parsed results."""
    try:
        subprocess.run(
            ["cargo", "bench"],
            capture_output=True, text=True, cwd=ROOT, timeout=600,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return load_benchmarks()


_BENCH_NAME_RE = re.compile(r"Benchmarking\s+(.+?)(?:\s|:)")


def run_benchmarks_live(progress_cb: object = None) -> list[BenchResult]:
    """Run cargo bench with live progress from stderr.

    ``progress_cb(current_name, completed, total_estimate)`` is called
    as criterion prints "Benchmarking <name>" lines.
    """
    import select as _select

    try:
        proc = subprocess.Popen(
            ["cargo", "bench"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            cwd=ROOT,
            text=True,
            bufsize=1,
        )
        completed = 0
        current_name = ""
        while proc.poll() is None:
            ready, _, _ = _select.select([proc.stderr], [], [], 0.3)
            if ready:
                line = proc.stderr.readline()
                if not line:
                    continue
                m = _BENCH_NAME_RE.search(line)
                if m:
                    if current_name:
                        completed += 1
                    current_name = m.group(1)
                    if progress_cb:
                        progress_cb(current_name, completed, 0)
        if current_name:
            completed += 1
        if progress_cb:
            progress_cb("", completed, completed)
    except FileNotFoundError:
        pass
    return load_benchmarks()


# -- Fuzz status -------------------------------------------------------------


@dataclass
class FuzzTarget:
    name: str
    corpus_count: int = 0
    crash_count: int = 0
    has_run: bool = False


@dataclass
class FuzzProgress:
    target: str
    elapsed: float = 0.0
    duration: float = 10.0
    iterations: int = 0
    exec_per_sec: int = 0
    coverage: int = 0
    corpus_count: int = 0


_FUZZ_ITERS_RE = re.compile(r"#(\d+)")
_FUZZ_COV_RE = re.compile(r"cov:\s*(\d+)")
_FUZZ_CORP_RE = re.compile(r"corp:\s*(\d+)")
_FUZZ_EXEC_RE = re.compile(r"exec/s:\s*(\d+)")


def _parse_fuzz_line(line: str, progress: FuzzProgress) -> None:
    """Parse a libfuzzer output line and update progress in-place."""
    m = _FUZZ_ITERS_RE.search(line)
    if m:
        progress.iterations = int(m.group(1))
    m = _FUZZ_COV_RE.search(line)
    if m:
        progress.coverage = int(m.group(1))
    m = _FUZZ_CORP_RE.search(line)
    if m:
        progress.corpus_count = int(m.group(1))
    m = _FUZZ_EXEC_RE.search(line)
    if m:
        progress.exec_per_sec = int(m.group(1))


def load_fuzz_status() -> list[FuzzTarget]:
    """Check fuzz target status from corpus/crash directories."""
    fuzz_dir = ROOT / "fuzz"
    targets: list[FuzzTarget] = []

    if not fuzz_dir.exists():
        return targets

    # Parse fuzz/Cargo.toml for target names
    cargo_toml = fuzz_dir / "Cargo.toml"
    if not cargo_toml.exists():
        return targets

    target_names: list[str] = []
    content = cargo_toml.read_text()
    in_bin_section = False
    for line in content.splitlines():
        stripped = line.strip()
        if stripped == "[[bin]]":
            in_bin_section = True
            continue
        if stripped.startswith("[") and stripped != "[[bin]]":
            in_bin_section = False
        if in_bin_section and stripped.startswith('name = "'):
            name = stripped.split('"')[1]
            target_names.append(name)

    for name in target_names:
        corpus_dir = fuzz_dir / "corpus" / name
        crash_dir = fuzz_dir / "artifacts" / name
        corpus_count = len(list(corpus_dir.iterdir())) if corpus_dir.exists() else 0
        crash_count = 0
        if crash_dir.exists():
            crash_count = sum(1 for f in crash_dir.iterdir() if f.name.startswith("crash-"))
        has_run = corpus_count > 0
        targets.append(FuzzTarget(
            name=name,
            corpus_count=corpus_count,
            crash_count=crash_count,
            has_run=has_run,
        ))

    return targets


def run_fuzz(target: str, duration: int = 10) -> FuzzTarget:
    """Run a fuzz target for a given duration."""
    try:
        subprocess.run(
            ["cargo", "fuzz", "run", target, "--", f"-max_total_time={duration}"],
            capture_output=True, text=True, cwd=ROOT, timeout=duration + 30,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass

    # Re-read status
    targets = load_fuzz_status()
    for t in targets:
        if t.name == target:
            return t
    return FuzzTarget(name=target)


def run_fuzz_live(
    target: str,
    duration: int = 10,
    progress_cb: object = None,
) -> FuzzTarget:
    """Run a fuzz target, parse libfuzzer stderr, and report live metrics.

    ``progress_cb`` receives a :class:`FuzzProgress` object on each update.
    Metrics (iterations, coverage, corpus, exec/s) come from actual fuzzer
    output — *not* from a timer.
    """
    import fcntl as _fcntl

    progress = FuzzProgress(target=target, duration=float(duration))
    try:
        proc = subprocess.Popen(
            ["cargo", "fuzz", "run", target, "--", f"-max_total_time={duration}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            cwd=ROOT,
        )
        # Non-blocking stderr so we never deadlock on partial lines
        fd = proc.stderr.fileno()
        flags = _fcntl.fcntl(fd, _fcntl.F_GETFL)
        _fcntl.fcntl(fd, _fcntl.F_SETFL, flags | os.O_NONBLOCK)

        start = time.monotonic()
        buf = b""
        while proc.poll() is None:
            progress.elapsed = time.monotonic() - start
            try:
                chunk = os.read(fd, 8192)
                if chunk:
                    buf += chunk
                    # Process complete lines
                    while b"\n" in buf:
                        line, buf = buf.split(b"\n", 1)
                        text = line.decode("utf-8", errors="replace").strip()
                        if text:
                            _parse_fuzz_line(text, progress)
            except BlockingIOError:
                pass
            if progress_cb:
                progress_cb(progress)
            time.sleep(0.2)
            if progress.elapsed > duration + 30:
                proc.kill()
                break
        # Drain remaining
        try:
            rest = proc.stderr.read()
            if rest:
                for line in (buf + rest).split(b"\n"):
                    text = line.decode("utf-8", errors="replace").strip()
                    if text:
                        _parse_fuzz_line(text, progress)
        except Exception:
            pass
    except FileNotFoundError:
        pass

    all_targets = load_fuzz_status()
    for t in all_targets:
        if t.name == target:
            return t
    return FuzzTarget(name=target)


# -- Quality gate checks ---------------------------------------------------


@dataclass
class CheckResult:
    name: str
    command: str
    passed: bool
    output: str = ""
    exec_time_s: float = 0.0


@dataclass
class GitSummary:
    branch: str = "unknown"
    files_changed: int = 0
    insertions: int = 0
    deletions: int = 0
    file_stats: list[tuple[str, int, int]] = field(default_factory=list)
    recent_commits: list[str] = field(default_factory=list)


def git_summary() -> GitSummary:
    """Collect git branch, diff stats, and recent commits."""
    summary = GitSummary()
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True, text=True, cwd=ROOT, timeout=5,
        )
        summary.branch = proc.stdout.strip() or "unknown"

        # Diff stats vs HEAD (staged + unstaged)
        proc = subprocess.run(
            ["git", "diff", "HEAD", "--numstat"],
            capture_output=True, text=True, cwd=ROOT, timeout=10,
        )
        for line in proc.stdout.strip().splitlines():
            parts = line.split("\t")
            if len(parts) >= 3:
                add = int(parts[0]) if parts[0] != "-" else 0
                rem = int(parts[1]) if parts[1] != "-" else 0
                summary.file_stats.append((parts[2], add, rem))
                summary.insertions += add
                summary.deletions += rem
        summary.files_changed = len(summary.file_stats)

        # Recent commits
        proc = subprocess.run(
            ["git", "log", "--oneline", "-8"],
            capture_output=True, text=True, cwd=ROOT, timeout=5,
        )
        summary.recent_commits = [
            line.strip()
            for line in proc.stdout.strip().splitlines()
            if line.strip()
        ]
    except Exception:
        pass
    return summary


_QUALITY_CHECKS = [
    ("cargo fmt", ["cargo", "fmt", "--check"], 5),
    (
        "cargo clippy",
        [
            "cargo", "clippy", "--features", "wayland,blocking",
            "--", "-D", "warnings",
        ],
        60,
    ),
    (
        "cargo doc",
        ["cargo", "doc", "--no-deps", "--features", "wayland,blocking"],
        30,
    ),
    (
        "cargo test --doc",
        ["cargo", "test", "--doc", "--features", "wayland,blocking"],
        30,
    ),
]


@dataclass
class SourceUpdate:
    """Result of checking the Hyprland source repo for updates."""
    current_version: str = "unknown"
    latest_version: str = "unknown"
    has_update: bool = False
    sdk_summary: str = ""  # diff --stat summary for SDK-relevant paths
    new_commands: list[str] = field(default_factory=list)
    removed_commands: list[str] = field(default_factory=list)
    new_events: list[str] = field(default_factory=list)
    removed_events: list[str] = field(default_factory=list)
    new_hooks: list[str] = field(default_factory=list)
    removed_hooks: list[str] = field(default_factory=list)
    changed_protocols: list[str] = field(default_factory=list)
    api_changes: list[str] = field(default_factory=list)
    category_stats: dict[str, str] = field(default_factory=dict)
    error: str = ""


_SDK_PATHS = [
    "src/debug/HyprCtl.cpp",
    "src/managers/EventManager.cpp",
    "src/managers/KeybindManager.cpp",
    "src/managers/HookSystemManager.hpp",
    "src/plugins/PluginAPI.hpp",
    "src/plugins/HookSystem.hpp",
    "src/plugins/PluginSystem.hpp",
    "src/config/ConfigManager.hpp",
    "src/config/ConfigValue.hpp",
    "src/desktop/view/Window.hpp",
    "src/desktop/Workspace.hpp",
    "src/helpers/Monitor.hpp",
    "src/desktop/view/LayerSurface.hpp",
    "protocols/",
    "src/protocols/",
]


def check_hyprland_updates(progress_cb: object = None) -> SourceUpdate:
    """Check .sources/Hyprland for upstream updates.

    ``progress_cb(step: str)`` is called with status messages.
    """
    result = SourceUpdate()
    hypr_dir = ROOT / ".sources" / "Hyprland"
    version_file = ROOT / ".sources" / ".version"

    if not (hypr_dir / ".git").exists():
        result.error = ".sources/Hyprland not cloned"
        return result

    # Current version
    if version_file.exists():
        result.current_version = version_file.read_text().strip()
    else:
        result.error = "No .sources/.version file"
        return result

    # Fetch tags
    if progress_cb:
        progress_cb("Fetching tags...")
    try:
        subprocess.run(
            ["git", "-C", str(hypr_dir), "fetch", "--tags", "--quiet"],
            capture_output=True, timeout=30,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        result.error = "Failed to fetch tags"
        return result

    # Latest tag
    if progress_cb:
        progress_cb("Checking latest version...")
    try:
        proc = subprocess.run(
            ["git", "-C", str(hypr_dir), "tag", "--list", "v*"],
            capture_output=True, text=True, timeout=10,
        )
        tags = [t.strip() for t in proc.stdout.splitlines() if t.strip()]
        if not tags:
            result.error = "No version tags found"
            return result
        tags.sort(key=lambda t: [
            int(x) if x.isdigit() else x
            for x in re.split(r"[.\-]", t.lstrip("v"))
        ])
        result.latest_version = tags[-1]
    except Exception as e:
        result.error = f"Failed to read tags: {e}"
        return result

    if result.current_version == result.latest_version:
        result.has_update = False
        return result

    result.has_update = True
    from_v = result.current_version
    to_v = result.latest_version

    if progress_cb:
        progress_cb(f"Diffing {from_v} -> {to_v}...")

    def _git_diff(args: list[str]) -> str:
        try:
            proc = subprocess.run(
                ["git", "-C", str(hypr_dir)] + args,
                capture_output=True, text=True, timeout=30,
            )
            return proc.stdout
        except Exception:
            return ""

    # SDK-relevant diff summary
    stat_out = _git_diff(["diff", "--stat", f"{from_v}..{to_v}", "--"] + _SDK_PATHS)
    lines = stat_out.strip().splitlines()
    result.sdk_summary = lines[-1] if lines else ""

    # Per-category stats
    categories = {
        "IPC": ["src/debug/HyprCtl.cpp", "src/managers/EventManager.cpp",
                 "src/managers/KeybindManager.cpp"],
        "Protocols": ["protocols/", "src/protocols/"],
        "Plugin API": ["src/plugins/PluginAPI.hpp", "src/plugins/HookSystem.hpp",
                        "src/plugins/PluginSystem.hpp", "src/managers/HookSystemManager.hpp"],
        "Types": ["src/desktop/", "src/helpers/Monitor.hpp"],
        "Config": ["src/config/ConfigManager.hpp", "src/config/ConfigValue.hpp"],
    }
    for cat_name, paths in categories.items():
        out = _git_diff(["diff", "--stat", f"{from_v}..{to_v}", "--"] + paths)
        cat_lines = out.strip().splitlines()
        if cat_lines:
            result.category_stats[cat_name] = cat_lines[-1].strip()

    if progress_cb:
        progress_cb("Checking IPC changes...")

    # New/removed IPC commands
    diff_hyprctl = _git_diff(["diff", f"{from_v}..{to_v}", "--", "src/debug/HyprCtl.cpp"])
    for line in diff_hyprctl.splitlines():
        if line.startswith("+") and "registerCommand" in line and not line.startswith("+++"):
            result.new_commands.append(line[1:].strip())
        elif line.startswith("-") and "registerCommand" in line and not line.startswith("---"):
            result.removed_commands.append(line[1:].strip())

    # New/removed events
    diff_events = _git_diff(["diff", f"{from_v}..{to_v}", "--", "src/managers/EventManager.cpp"])
    for line in diff_events.splitlines():
        if line.startswith("+") and "postEvent" in line and not line.startswith("+++"):
            result.new_events.append(line[1:].strip())
        elif line.startswith("-") and "postEvent" in line and not line.startswith("---"):
            result.removed_events.append(line[1:].strip())

    # New/removed hooks
    diff_hooks = _git_diff(["diff", f"{from_v}..{to_v}", "--", "src/managers/HookSystemManager.hpp"])
    for line in diff_hooks.splitlines():
        if line.startswith("+") and ("EMIT_HOOK_EVENT" in line or "HOOK_" in line) and not line.startswith("+++"):
            result.new_hooks.append(line[1:].strip())
        elif line.startswith("-") and ("EMIT_HOOK_EVENT" in line or "HOOK_" in line) and not line.startswith("---"):
            result.removed_hooks.append(line[1:].strip())

    # Changed protocol XMLs
    xml_out = _git_diff(["diff", "--name-only", f"{from_v}..{to_v}", "--", "protocols/*.xml"])
    result.changed_protocols = [l.strip() for l in xml_out.splitlines() if l.strip()]

    # Plugin API changes
    diff_api = _git_diff(["diff", f"{from_v}..{to_v}", "--", "src/plugins/PluginAPI.hpp"])
    for line in diff_api.splitlines():
        if re.match(r"^[+-].*(inline|namespace HyprlandAPI)", line) and not line.startswith("+++") and not line.startswith("---"):
            result.api_changes.append(line.strip())

    return result


def run_quality_gates_live(progress_cb: object = None) -> list[CheckResult]:
    """Run quality gate checks with progress callbacks.

    ``progress_cb(name, status, elapsed, est_time)`` is called periodically.
    *status* is ``"running"``, ``"passed"``, or ``"failed"``.
    """
    import select as _select

    results: list[CheckResult] = []
    for name, cmd, est_time in _QUALITY_CHECKS:
        if progress_cb:
            progress_cb(name, "running", 0, est_time)
        start = time.monotonic()
        try:
            proc = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                cwd=ROOT, bufsize=1, text=True,
            )
            while proc.poll() is None:
                elapsed = time.monotonic() - start
                if progress_cb:
                    progress_cb(name, "running", elapsed, est_time)
                _select.select([proc.stderr], [], [], 0.3)
                if elapsed > 120:
                    proc.kill()
                    break
            elapsed = time.monotonic() - start
            stdout = proc.stdout.read() if proc.stdout else ""
            stderr = proc.stderr.read() if proc.stderr else ""
            passed = proc.returncode == 0
            results.append(CheckResult(
                name=name, command=" ".join(cmd),
                passed=passed, output=stdout + stderr, exec_time_s=elapsed,
            ))
        except FileNotFoundError:
            elapsed = time.monotonic() - start
            results.append(CheckResult(
                name=name, command=" ".join(cmd),
                passed=False, output="cargo not found", exec_time_s=elapsed,
            ))
        if progress_cb:
            status = "passed" if results[-1].passed else "failed"
            progress_cb(name, status, results[-1].exec_time_s, est_time)
    return results
