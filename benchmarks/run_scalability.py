#!/usr/bin/env python3
"""Measure scalability of scene preparation and SVG export."""
import argparse
import hashlib
import json
import os
import platform
import statistics
import sys
import time
import tracemalloc
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from renderer import render_scene_svg
from scene import build_scene

SEED = 20261029


def make_matrix(size, seed=SEED):
    state = seed & 0x7fffffff
    matrix = [[0.0] * size for _ in range(size)]
    for row in range(size):
        matrix[row][row] = 1.0
        for column in range(row + 1, size):
            state = (1103515245 * state + 12345) & 0x7fffffff
            value = ((state % 18001) - 9000) / 10000.0
            matrix[row][column] = matrix[column][row] = value
    return matrix


def make_workloads(stress=False):
    base = make_matrix(100)
    missing = [row[:] for row in base]
    for row, column in [(r, c) for r in range(100) for c in range(r + 1, 100)][::10]:
        missing[row][column] = missing[column][row] = None
    cases = {
        "B01": {"labels": [f"v{i:02d}" for i in range(20)], "matrix": make_matrix(20)},
        "B02": {"labels": [f"v{i:03d}" for i in range(100)], "matrix": base},
        "B05": {"labels": [f"v{i:03d}" for i in range(100)], "matrix": missing},
        "B06": {"labels": [f"variable_{i:03d}_long_descriptive_label" for i in range(100)],
                "matrix": base, "annotation": "Long labels and explanatory annotation",
                "layout": {"label_collision": "truncate"}},
    }
    if stress:
        for name, size in (("B03", 500), ("B04", 1000)):
            cases[name] = {"labels": [f"v{i:04d}" for i in range(size)],
                           "matrix": make_matrix(size)}
    return cases


def validate(config):
    labels, matrix = config["labels"], config["matrix"]
    size = len(labels)
    if not size or len(matrix) != size or any(len(row) != size for row in matrix):
        raise ValueError("matrix dimensions do not match labels")
    for r in range(size):
        for c in range(size):
            value = matrix[r][c]
            if value is not None and (isinstance(value, bool) or not isinstance(value, (int, float)) or not -1 <= value <= 1):
                raise ValueError(f"invalid value at ({r}, {c})")
            if value != matrix[c][r]:
                raise ValueError(f"matrix is not symmetric at ({r}, {c})")


def digest(value):
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(data.encode("utf-8")).hexdigest()


def measure(operation, iterations):
    started = time.perf_counter_ns()
    operation()
    first = (time.perf_counter_ns() - started) / 1e6
    for _ in range(2):
        operation()
    samples = []
    tracemalloc.start()
    try:
        for _ in range(iterations):
            started = time.perf_counter_ns()
            operation()
            samples.append((time.perf_counter_ns() - started) / 1e6)
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    ordered = sorted(samples)
    q1, q3 = statistics.quantiles(ordered, n=4, method="inclusive")[0::2]
    return {"first_call_ms": first, "median_ms": statistics.median(ordered),
            "iqr_ms": q3 - q1, "samples_ms": samples,
            "peak_traced_allocation_bytes": peak,
            "memory_metric_note": "tracemalloc peak; Python allocation proxy, not process RSS"}


def run(iterations, stress):
    results = []
    for workload, config in make_workloads(stress).items():
        try:
            validate(config)
            encoded_config = json.dumps(config, separators=(",", ":"), allow_nan=False)
            loaded = json.loads(encoded_config)
            scene = build_scene(loaded)
            if digest(scene) != digest(build_scene(loaded)):
                raise AssertionError("scene output is not deterministic")
            svg = render_scene_svg(scene)
            if svg != render_scene_svg(scene):
                raise AssertionError("SVG output is not deterministic")
        except Exception as error:
            results.append({"workload": workload, "status": "not_comparable",
                            "correctness_error": f"{type(error).__name__}: {error}"})
            continue
        stages = (
            ("configuration_loading", lambda: json.loads(encoded_config)),
            ("validation_and_scene_construction", lambda: build_scene(loaded)),
            ("svg_serialization", lambda: render_scene_svg(scene)),
            ("end_to_end", lambda: render_scene_svg(build_scene(json.loads(encoded_config)))),
        )
        for stage, operation in stages:
            results.append({
                "workload": workload, "matrix_dimensions": [len(config["labels"])] * 2,
                "cell_count": len(scene["cells"]), "input_sha256": digest(config),
                "scene_sha256": digest(scene), "svg_bytes": len(svg.encode("utf-8")),
                "stage": stage, "status": "passed", **measure(operation, iterations),
            })
    return {
        "schema_version": 1, "suite": "corrplot-ng-renderer-scalability",
        "seed": SEED, "iterations": iterations,
        "profile": "stress" if stress else "regular",
        "environment": {"python": sys.version, "implementation": platform.python_implementation(),
                        "platform": platform.platform(), "machine": platform.machine(),
                        "processor": platform.processor(), "cpu_count": os.cpu_count()},
        "methodology": {"warmups": 2, "clock": "time.perf_counter_ns",
                        "memory": "tracemalloc peak, not process RSS",
                        "correctness": "input invariants and deterministic scene/SVG checked before timing"},
        "results": results,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=5)
    parser.add_argument("--include-stress", action="store_true",
                        help="include 500x500 and 1000x1000 matrices")
    parser.add_argument("--output", type=Path, default=Path("benchmark-results/scalability.json"))
    args = parser.parse_args()
    if args.iterations < 3:
        parser.error("--iterations must be at least 3")
    result = run(args.iterations, args.include_stress)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for item in result["results"]:
        if item["status"] == "passed":
            print(f'{item["workload"]} {item["stage"]}: {item["median_ms"]:.2f} ms')
        else:
            print(f'{item["workload"]}: NOT COMPARABLE ({item["correctness_error"]})')
    print(f"Results written to {args.output}")


if __name__ == "__main__":
    main()
