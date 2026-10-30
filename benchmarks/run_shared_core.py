#!/usr/bin/env python3
"""Deterministic initial benchmark suite for the shared scene prototype."""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import statistics
import sys
import time
import tracemalloc
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from renderer import render_scene_svg  # noqa: E402
from scene import build_scene  # noqa: E402

SEED = 20261029
ITERATIONS_MINIMUM = 20


def make_matrix(size: int, seed: int = SEED) -> list[list[float]]:
    """Create a deterministic symmetric matrix with a unit diagonal."""
    # Integer arithmetic avoids platform-dependent random-number behavior.
    state = seed & 0x7FFFFFFF
    values = [[0.0] * size for _ in range(size)]
    for row in range(size):
        values[row][row] = 1.0
        for column in range(row + 1, size):
            state = (1103515245 * state + 12345) & 0x7FFFFFFF
            value = ((state % 18001) - 9000) / 10000.0
            values[row][column] = value
            values[column][row] = value
    return values


def make_cases() -> dict[str, dict[str, Any]]:
    labels20 = [f"v{i:02d}" for i in range(20)]
    labels100 = [f"v{i:03d}" for i in range(100)]
    matrix20 = make_matrix(20)
    matrix100 = make_matrix(100)
    missing = [row[:] for row in matrix100]
    # Deterministic symmetric 10% off-diagonal missingness.
    pairs = [(i, j) for i in range(100) for j in range(i + 1, 100)]
    target = round(len(pairs) * 0.10)
    for i, j in pairs[:: max(1, len(pairs) // target)][:target]:
        missing[i][j] = None
        missing[j][i] = None
    permutation = list(reversed(range(100)))
    reordered = [[matrix100[i][j] for j in permutation] for i in permutation]
    return {
        "B01": {"labels": labels20, "matrix": matrix20},
        "B02": {"labels": labels100, "matrix": matrix100},
        "B05": {"labels": labels100, "matrix": missing},
        "B07": {
            "labels": [labels100[i] for i in permutation],
            "matrix": reordered,
            "permutation": permutation,
        },
    }


def canonical_hash(value: Any) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def validate_scene(case_id: str, config: dict[str, Any], scene: dict[str, Any]) -> None:
    labels, matrix = config["labels"], config["matrix"]
    size = len(labels)
    assert len(matrix) == size and all(len(row) == size for row in matrix)
    assert scene["schema_version"] == 1
    assert scene["kind"] == "correlation_plot"
    assert len(scene["labels"]) == size
    assert [item["text"] for item in scene["labels"]] == [str(x) for x in labels]
    assert len(scene["cells"]) == size * size
    expected_order = [(r, c) for r in range(size) for c in range(size)]
    actual_order = [(cell["row"], cell["column"]) for cell in scene["cells"]]
    assert actual_order == expected_order, f"{case_id}: cells are not row-major"
    expected_values = [value for row in matrix for value in row]
    actual_values = [cell["value"] for cell in scene["cells"]]
    assert actual_values == expected_values, f"{case_id}: values/nulls changed"
    if case_id != "B08":
        for i in range(size):
            for j in range(size):
                assert matrix[i][j] == matrix[j][i], f"{case_id}: input not symmetric"


def measure(
    operation: Callable[[], Any], iterations: int
) -> tuple[float, list[float], int]:
    first_started = time.perf_counter_ns()
    operation()
    first_ms = (time.perf_counter_ns() - first_started) / 1_000_000

    for _ in range(3):
        operation()

    samples: list[float] = []
    tracemalloc.start()
    try:
        for _ in range(iterations):
            started = time.perf_counter_ns()
            operation()
            samples.append((time.perf_counter_ns() - started) / 1_000_000)
        _, peak_bytes = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return first_ms, samples, peak_bytes


def summarize(samples: list[float]) -> dict[str, float]:
    ordered = sorted(samples)
    median = statistics.median(ordered)
    q1, q3 = statistics.quantiles(ordered, n=4, method="inclusive")[0::2]
    return {
        "median_ms": median,
        "iqr_ms": q3 - q1,
        "min_ms": min(ordered),
        "max_ms": max(ordered),
    }


def run(iterations: int) -> dict[str, Any]:
    cases = make_cases()
    results: list[dict[str, Any]] = []
    serialized_inputs: dict[str, Any] = {}

    for case_id, config in cases.items():
        operation = lambda config=config: build_scene(config)
        try:
            scene = operation()
            validate_scene(case_id, config, scene)
            if canonical_hash(scene) != canonical_hash(operation()):
                raise AssertionError(f"{case_id}: scene output is not deterministic")
        except Exception as exc:  # correctness gate is recorded, not hidden
            results.append({"workload": case_id, "status": "not_comparable",
                            "correctness_error": f"{type(exc).__name__}: {exc}"})
            continue

        serialized_inputs[case_id] = config
        first_ms, samples, peak = measure(operation, iterations)
        results.append({
            "workload": case_id,
            "status": "passed",
            "measurement": "core_only_scene_construction",
            "first_call_ms": first_ms,
            **summarize(samples),
            "samples_ms": samples,
            "peak_traced_allocation_bytes": peak,
            "memory_metric_note": "tracemalloc peak; Python allocation proxy, not process RSS",
            "scene_sha256": canonical_hash(scene),
            "scene_cell_count": len(scene["cells"]),
            "svg_bytes": len(render_scene_svg(scene).encode("utf-8")),
        })

    # B08 is deliberately isolated: the scene is constructed outside timing.
    b02 = build_scene(cases["B02"])
    try:
        svg = render_scene_svg(b02)
        if svg != render_scene_svg(b02):
            raise AssertionError("B08: SVG serialization is not deterministic")
    except Exception as exc:
        results.append({"workload": "B08", "status": "not_comparable",
                        "correctness_error": f"{type(exc).__name__}: {exc}"})
    else:
        first_ms, samples, peak = measure(lambda: render_scene_svg(b02), iterations)
        results.append({
            "workload": "B08",
            "status": "passed",
            "measurement": "svg_serialization_only",
            "first_call_ms": first_ms,
            **summarize(samples),
            "samples_ms": samples,
            "peak_traced_allocation_bytes": peak,
            "memory_metric_note": "tracemalloc peak; Python allocation proxy, not process RSS",
            "scene_sha256": canonical_hash(b02),
            "scene_cell_count": len(b02["cells"]),
            "svg_bytes": len(svg.encode("utf-8")),
        })
        serialized_inputs["B08"] = {"source_workload": "B02",
                                    "scene_sha256": canonical_hash(b02)}

    return {
        "schema_version": 1,
        "suite": "corrplot-ng-initial-shared-core",
        "seed": SEED,
        "iterations": iterations,
        "warmup_iterations": 3,
        "correctness_gate": "semantic assertions before timing",
        "environment": {
            "python": sys.version,
            "implementation": platform.python_implementation(),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "cpu_count": __import__("os").cpu_count(),
        },
        "inputs": serialized_inputs,
        "results": results,
        "scope_note": "Python prototype only; no cross-language comparison or production-language decision.",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--output", type=Path, default=Path("benchmark-results/results.json"))
    args = parser.parse_args()
    if args.iterations < ITERATIONS_MINIMUM:
        parser.error(f"--iterations must be at least {ITERATIONS_MINIMUM}")
    result = run(args.iterations)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for item in result["results"]:
        if item["status"] == "passed":
            print(f'{item["workload"]}: median={item["median_ms"]:.3f} ms '
                  f'IQR={item["iqr_ms"]:.3f} ms ({item["measurement"]})')
        else:
            print(f'{item["workload"]}: NOT COMPARABLE ({item["correctness_error"]})')
    print(f"Results written to {args.output}")


if __name__ == "__main__":
    main()
