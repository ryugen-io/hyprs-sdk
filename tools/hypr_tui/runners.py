"""Run cargo test, bench, and fuzz commands and parse their output."""

from __future__ import annotations

import json
import subprocess
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


@dataclass
class TestSummary:
    passed: int = 0
    failed: int = 0
    ignored: int = 0
    results: list[TestResult] = field(default_factory=list)
    error: str = ""


def run_tests() -> TestSummary:
    """Run cargo test and return parsed results."""
    summary = TestSummary()
    try:
        proc = subprocess.run(
            [
                "cargo", "test", "--features", "wayland,blocking",
                "--", "--format", "json", "-Z", "unstable-options",
            ],
            capture_output=True, text=True, cwd=ROOT, timeout=120,
        )
        for line in proc.stdout.splitlines():
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            if obj.get("type") == "test" and "event" in obj and obj["event"] != "started":
                name = obj.get("name", "?")
                status = obj["event"]  # "ok" or "failed"
                summary.results.append(TestResult(name=name, status=status))
            elif obj.get("type") == "suite" and obj.get("event") in ("ok", "failed"):
                summary.passed += obj.get("passed", 0)
                summary.failed += obj.get("failed", 0)
                summary.ignored += obj.get("ignored", 0)
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
    """Run cargo bench and return parsed results."""
    try:
        subprocess.run(
            ["cargo", "bench", "--", "--quick"],
            capture_output=True, text=True, cwd=ROOT, timeout=300,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return load_benchmarks()


# -- Fuzz status -------------------------------------------------------------


@dataclass
class FuzzTarget:
    name: str
    corpus_count: int = 0
    crash_count: int = 0
    has_run: bool = False


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
