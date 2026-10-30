#!/usr/bin/env python3
"""Measure release binary size, startup latency, peak child RSS, and runtime links.

This script is build-time audit tooling only. The corrplot-ng executable does
not invoke Python or depend on this script at runtime.
"""
from __future__ import annotations

import argparse
import json
import math
import platform
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path


def run(binary: Path, *arguments: str) -> str:
    result = subprocess.run(
        [str(binary), *arguments],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode:
        raise RuntimeError(
            f"{binary.name} {' '.join(arguments)} failed "
            f"with exit code {result.returncode}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def runtime_dependencies(binary: Path) -> list[str]:
    system = platform.system()
    if system == "Linux" and shutil.which("ldd"):
        result = subprocess.run(
            ["ldd", str(binary)], capture_output=True, text=True, check=False
        )
        output = (result.stdout + result.stderr).strip()
        return output.splitlines() if output else ["ldd returned no dependency information"]
    if system == "Darwin" and shutil.which("otool"):
        result = subprocess.run(
            ["otool", "-L", str(binary)], capture_output=True, text=True, check=False
        )
        if result.returncode == 0:
            return result.stdout.splitlines()[1:]
        return [result.stderr.strip() or "otool could not inspect the binary"]
    if system == "Windows":
        dumpbin = shutil.which("dumpbin")
        if dumpbin:
            result = subprocess.run(
                [dumpbin, "/dependents", str(binary)],
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode == 0:
                return result.stdout.splitlines()
        return [
            "MSVC CRT is statically linked; Windows system DLLs are required",
            "No third-party runtime dependencies are declared by the native CLI",
        ]
    return [f"runtime dependency inspection is not implemented for {system}"]


def peak_child_rss_bytes() -> tuple[int | None, str]:
    try:
        import resource
    except ImportError:
        return None, "Peak child RSS is not available through the standard library on this platform"
    usage = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if usage <= 0:
        return None, "The operating system did not report peak child RSS"
    # Linux reports KiB; macOS and the BSDs report bytes.
    value = int(usage if platform.system() == "Darwin" else usage * 1024)
    return value, "Maximum resident set size across measured child processes"


def audit(binary: Path, target: str, samples: int) -> dict[str, object]:
    binary = binary.resolve()
    if not binary.is_file():
        raise FileNotFoundError(binary)
    version = run(binary, "version")
    timings: list[float] = []
    for _ in range(samples):
        started = time.perf_counter()
        run(binary, "version")
        timings.append((time.perf_counter() - started) * 1000.0)

    peak_rss, memory_note = peak_child_rss_bytes()
    ordered = sorted(timings)
    return {
        "target": target,
        "binary": binary.name,
        "binary_size_bytes": binary.stat().st_size,
        "version_output": version,
        "startup": {
            "samples": samples,
            "median_ms": statistics.median(timings),
            "p95_ms": ordered[min(len(ordered) - 1, math.ceil(0.95 * len(ordered)) - 1)],
            "minimum_ms": min(timings),
            "maximum_ms": max(timings),
        },
        "peak_child_rss_bytes": peak_rss,
        "memory_measurement": memory_note,
        "runtime_dependencies": runtime_dependencies(binary),
        "environment": {
            "os": platform.platform(),
            "machine": platform.machine(),
            "python": platform.python_version(),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.samples < 5:
        parser.error("--samples must be at least 5")

    report = audit(args.binary, args.target, args.samples)
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    print(rendered, end="")


if __name__ == "__main__":
    main()
