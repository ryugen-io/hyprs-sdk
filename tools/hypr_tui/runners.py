"""Run cargo test, bench, and fuzz commands and parse their output."""

from __future__ import annotations

import json
import re
import subprocess
import time
import os
from dataclasses import dataclass, field
from pathlib import Path
from urllib import error as urlerror
from urllib import request as urlrequest


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
    sdk_update_needed: bool = False
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

_CATEGORY_PATHS = {
    "IPC": [
        "src/debug/HyprCtl.cpp",
        "src/managers/EventManager.cpp",
        "src/managers/KeybindManager.cpp",
    ],
    "Protocols": ["protocols/", "src/protocols/"],
    "Plugin API": [
        "src/plugins/PluginAPI.hpp",
        "src/plugins/HookSystem.hpp",
        "src/plugins/PluginSystem.hpp",
        "src/managers/HookSystemManager.hpp",
    ],
    "Types": ["src/desktop/", "src/helpers/Monitor.hpp"],
    "Config": ["src/config/ConfigManager.hpp", "src/config/ConfigValue.hpp"],
}

_HYPR_REPO_API = "https://api.github.com/repos/hyprwm/Hyprland"


def _semver_key(tag: str) -> list[object]:
    return [
        int(x) if x.isdigit() else x
        for x in re.split(r"[.\-]", tag.lstrip("v"))
    ]


def _normalize_tag(version: str) -> str:
    v = version.strip()
    if not v:
        return "unknown"
    return v if v.startswith("v") else f"v{v}"


def _matches_path(path: str, patterns: list[str]) -> bool:
    for p in patterns:
        if p.endswith("/") and path.startswith(p):
            return True
        if path == p:
            return True
    return False


def _format_diff_summary(files: list[dict[str, object]]) -> str:
    if not files:
        return ""
    additions = sum(int(f.get("additions", 0)) for f in files)
    deletions = sum(int(f.get("deletions", 0)) for f in files)
    file_word = "file" if len(files) == 1 else "files"
    return (
        f"{len(files)} {file_word} changed, "
        f"{additions} insertions(+), {deletions} deletions(-)"
    )


def _dedupe(items: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for item in items:
        if item not in seen:
            seen.add(item)
            out.append(item)
    return out


def _github_json(url: str) -> object:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "hypr-sdk-hypr-tui",
    }
    token = os.environ.get("GITHUB_TOKEN", "").strip()
    if token:
        headers["Authorization"] = f"Bearer {token}"

    req = urlrequest.Request(url, headers=headers)
    try:
        with urlrequest.urlopen(req, timeout=20) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urlerror.HTTPError as e:
        body = ""
        try:
            body = e.read().decode("utf-8", errors="replace")
        except Exception:
            pass
        if e.code == 403 and "rate limit" in body.lower():
            raise RuntimeError(
                "GitHub API rate limit reached (set GITHUB_TOKEN for higher limits)"
            ) from e
        raise RuntimeError(f"GitHub API error ({e.code}) for {url}") from e
    except urlerror.URLError as e:
        raise RuntimeError(f"Network error while contacting GitHub: {e.reason}") from e


def _read_target_version_from_sdk() -> str:
    lib_rs = ROOT / "src" / "lib.rs"
    if not lib_rs.exists():
        return "unknown"
    try:
        text = lib_rs.read_text(encoding="utf-8")
    except OSError:
        return "unknown"
    m = re.search(
        r'HYPRLAND_TARGET_VERSION\s*:\s*&str\s*=\s*"([^"]+)"',
        text,
    )
    if not m:
        return "unknown"
    return _normalize_tag(m.group(1))


def _resolve_current_hyprland_version(version_file: Path) -> str:
    if version_file.exists():
        try:
            v = version_file.read_text(encoding="utf-8").strip()
            if v:
                return _normalize_tag(v)
        except OSError:
            pass
    return _read_target_version_from_sdk()


def _fetch_latest_hyprland_tag() -> str:
    # Prefer latest release tag; fallback to tags list.
    release = _github_json(f"{_HYPR_REPO_API}/releases/latest")
    if isinstance(release, dict):
        tag = str(release.get("tag_name", "")).strip()
        if tag:
            return _normalize_tag(tag)

    tags = _github_json(f"{_HYPR_REPO_API}/tags?per_page=100")
    if not isinstance(tags, list):
        raise RuntimeError("Malformed GitHub tags response")
    names = [
        str(t.get("name", "")).strip()
        for t in tags
        if isinstance(t, dict)
    ]
    names = [n for n in names if n and re.match(r"^v?\d+\.\d+\.\d+", n)]
    if not names:
        raise RuntimeError("No Hyprland version tags found on GitHub")
    names.sort(key=_semver_key)
    return _normalize_tag(names[-1])


def _extract_patch_changes(
    patch: str,
    predicate: object,
) -> tuple[list[str], list[str]]:
    added: list[str] = []
    removed: list[str] = []
    for raw in patch.splitlines():
        if raw.startswith("+++") or raw.startswith("---"):
            continue
        if raw.startswith("+"):
            line = raw[1:].strip()
            if predicate(line):
                added.append(line)
        elif raw.startswith("-"):
            line = raw[1:].strip()
            if predicate(line):
                removed.append(line)
    return added, removed


def check_hyprland_updates(progress_cb: object = None) -> SourceUpdate:
    """Check upstream Hyprland updates without requiring a local clone."""
    result = SourceUpdate()
    version_file = ROOT / ".sources" / ".version"

    if progress_cb:
        progress_cb("Resolving current version...")
    result.current_version = _resolve_current_hyprland_version(version_file)
    if result.current_version == "unknown":
        result.error = (
            "No baseline version found "
            "(.sources/.version or HYPRLAND_TARGET_VERSION)"
        )
        return result

    if progress_cb:
        progress_cb("Fetching latest version...")
    try:
        result.latest_version = _fetch_latest_hyprland_tag()
    except RuntimeError as e:
        result.error = str(e)
        return result

    if result.current_version == result.latest_version:
        result.has_update = False
        return result

    result.has_update = True
    from_v = result.current_version
    to_v = result.latest_version

    if progress_cb:
        progress_cb(f"Diffing {from_v} -> {to_v}...")
    try:
        compare = _github_json(f"{_HYPR_REPO_API}/compare/{from_v}...{to_v}")
    except RuntimeError as e:
        result.error = str(e)
        return result

    if not isinstance(compare, dict):
        result.error = "Malformed GitHub compare response"
        return result

    files = compare.get("files", [])
    if not isinstance(files, list):
        result.error = "Malformed GitHub compare response: missing file list"
        return result

    sdk_files: list[dict[str, object]] = []
    for f in files:
        if not isinstance(f, dict):
            continue
        path = str(f.get("filename", ""))
        prev_path = str(f.get("previous_filename", ""))
        if _matches_path(path, _SDK_PATHS) or _matches_path(prev_path, _SDK_PATHS):
            sdk_files.append(f)
    result.sdk_update_needed = bool(sdk_files)
    result.sdk_summary = _format_diff_summary(sdk_files)

    for cat_name, paths in _CATEGORY_PATHS.items():
        cat_files = [
            f for f in files
            if isinstance(f, dict)
            and (
                _matches_path(str(f.get("filename", "")), paths)
                or _matches_path(str(f.get("previous_filename", "")), paths)
            )
        ]
        stat = _format_diff_summary(cat_files)
        if stat:
            result.category_stats[cat_name] = stat

    if progress_cb:
        progress_cb("Checking IPC changes...")

    for f in files:
        if not isinstance(f, dict):
            continue
        path = str(f.get("filename", ""))
        patch = str(f.get("patch", "") or "")

        if path.startswith("protocols/") and path.endswith(".xml"):
            result.changed_protocols.append(path)

        if not patch:
            continue

        if path == "src/debug/HyprCtl.cpp":
            add, rem = _extract_patch_changes(
                patch,
                lambda line: "registerCommand" in line,
            )
            result.new_commands.extend(add)
            result.removed_commands.extend(rem)

        if path == "src/managers/EventManager.cpp":
            add, rem = _extract_patch_changes(
                patch,
                lambda line: "postEvent" in line,
            )
            result.new_events.extend(add)
            result.removed_events.extend(rem)

        if path == "src/managers/HookSystemManager.hpp":
            add, rem = _extract_patch_changes(
                patch,
                lambda line: ("EMIT_HOOK_EVENT" in line or "HOOK_" in line),
            )
            result.new_hooks.extend(add)
            result.removed_hooks.extend(rem)

        if path == "src/plugins/PluginAPI.hpp":
            for raw in patch.splitlines():
                if raw.startswith(("+++", "---")):
                    continue
                if raw.startswith(("+", "-")) and re.search(
                    r"(inline|namespace HyprlandAPI)",
                    raw,
                ):
                    result.api_changes.append(raw.strip())

    result.changed_protocols = _dedupe(result.changed_protocols)
    result.new_commands = _dedupe(result.new_commands)
    result.removed_commands = _dedupe(result.removed_commands)
    result.new_events = _dedupe(result.new_events)
    result.removed_events = _dedupe(result.removed_events)
    result.new_hooks = _dedupe(result.new_hooks)
    result.removed_hooks = _dedupe(result.removed_hooks)
    result.api_changes = _dedupe(result.api_changes)

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
