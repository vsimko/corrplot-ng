#!/usr/bin/env python3
"""Compare native CLI startup and representative publication rendering.

Build baseline and candidate binaries from separate revisions, then run:
python benchmarks/cli_startup.py --baseline /path/to/baseline/corrplot-ng \
  --candidate /path/to/candidate/corrplot-ng
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import platform
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


def measure(binary: Path, command: list[str], samples: int) -> dict[str, float]:
    timings = []
    for _ in range(samples):
        started = time.perf_counter()
        result = subprocess.run(
            [str(binary), *command],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        elapsed = time.perf_counter() - started
        if result.returncode:
            raise RuntimeError(
                f"{binary.name} {' '.join(command)} failed: "
                f"{result.stderr.decode(errors='replace')}"
            )
        timings.append(elapsed * 1000)
    ordered = sorted(timings)
    return {
        "samples": len(timings),
        "median_ms": statistics.median(timings),
        "p95_ms": ordered[min(len(ordered) - 1, int(0.95 * len(ordered)))],
        "min_ms": min(timings),
        "max_ms": max(timings),
    }


def make_fixture(directory: Path, size: int) -> Path:
    matrix_path = directory / "matrix.csv"
    with matrix_path.open("w", newline="", encoding="utf-8") as stream:
        writer = csv.writer(stream)
        labels = [f"variable_{i:03d}" for i in range(size)]
        writer.writerow(["", *labels])
        for row, label in enumerate(labels):
            values = []
            for column in range(size):
                if row == column:
                    values.append("1.0")
                else:
                    # Deterministic symmetric correlations in a bounded range.
                    a, b = sorted((row, column))
                    values.append(f"{((a * 17 + b * 31) % 81 - 40) / 100:.2f}")
            writer.writerow([label, *values])

    config_path = directory / "benchmark.json"
    config = {
        "config_version": 1,
        "input": {
            "kind": "matrix_csv",
            "path": "matrix.csv",
            "delimiter": ",",
            "header": True,
            "row_labels": True,
        },
        "matrix": {"kind": "correlation", "method": "pearson", "missing": "preserve"},
        "plot": {
            "method": "circle",
            "display": "full",
            "ordering": "original",
            "show_values": False,
            "significance": {
                "enabled": False,
                "p_values": None,
                "thresholds": [0.001, 0.01, 0.05],
            },
        },
        "colors": {
            "negative": "#b2182b",
            "neutral": "#f7f7f7",
            "positive": "#2166ac",
            "missing": "#d9d9d9",
        },
        "dimensions": {"width": 160, "height": 145, "unit": "mm"},
        "typography": {"family": "sans-serif", "base_size": 9},
        "output": {"format": "svg", "path": "benchmark.svg"},
    }
    config_path.write_text(json.dumps(config), encoding="utf-8")
    return config_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--render-samples", type=int, default=10)
    parser.add_argument("--matrix-size", type=int, default=40)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    if args.samples < 5 or args.render_samples < 3 or args.matrix_size < 2:
        parser.error("use at least 5 startup samples, 3 render samples, and matrix size >= 2")

    with tempfile.TemporaryDirectory(prefix="corrplot-ng-cli-bench-") as temporary:
        directory = Path(temporary)
        config = make_fixture(directory, args.matrix_size)
        report = {
            "environment": {
                "platform": platform.platform(),
                "python": platform.python_version(),
                "machine": platform.machine(),
                "matrix_size": args.matrix_size,
                "startup_samples": args.samples,
                "render_samples": args.render_samples,
            },
            "results": {},
        }
        for label, binary in (("baseline", args.baseline), ("candidate", args.candidate)):
            binary = binary.resolve()
            report["results"][label] = {
                "binary": str(binary),
                "version_startup": measure(binary, ["version"], args.samples),
                "validate_config": measure(
                    binary, ["validate", "--config", str(config)], args.samples
                ),
                "render_config": measure(
                    binary,
                    ["render", "--config", str(config), "--output", str(directory / f"{label}.svg")],
                    args.render_samples,
                ),
            }

    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    print(rendered, end="")


if __name__ == "__main__":
    main()
