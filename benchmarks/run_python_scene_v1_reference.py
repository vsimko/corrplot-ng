#!/usr/bin/env python3
"""Measure Python scene-v1 construction, JSON encoding and end-to-end cost."""
from __future__ import annotations
import argparse
import json
import platform
import statistics
import sys
import time
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from scene import build_scene

TARGET_SAMPLE_MS = 100.0
SAMPLES = 30
WARMUPS = 5


def measure(operation: Callable[[], object], stress: bool = False) -> dict[str, object]:
    sample_count = 5 if stress else SAMPLES
    warmups = 1 if stress else WARMUPS
    first_started = time.perf_counter_ns()
    operation()
    first_ms = (time.perf_counter_ns() - first_started) / 1e6
    for _ in range(warmups):
        operation()

    batch = 1
    while not stress:
        started = time.perf_counter_ns()
        for _ in range(batch):
            operation()
        elapsed_ms = (time.perf_counter_ns() - started) / 1e6
        if elapsed_ms >= TARGET_SAMPLE_MS:
            break
        batch *= 2

    samples = []
    for _ in range(sample_count):
        started = time.perf_counter_ns()
        for _ in range(batch):
            operation()
        samples.append((time.perf_counter_ns() - started) / 1e6 / batch)

    q1, q3 = statistics.quantiles(sorted(samples), n=4, method="inclusive")[0::2]
    return {
        "first_call_ms": first_ms, "median_ms": statistics.median(samples),
        "iqr_ms": q3 - q1, "min_ms": min(samples), "max_ms": max(samples),
        "samples_ms": samples, "batch_size": batch,
        "sample_target_ms": TARGET_SAMPLE_MS, "sample_count": sample_count,
        "warmup_iterations": warmups, "stress_mode": stress,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    fixture = json.loads(args.fixtures.read_text(encoding="utf-8"))
    results = []
    for case in fixture["workloads"]:
        config = {"labels": case["labels"], "matrix": case["matrix"],
                  "annotation": case["annotation"]}
        expected = case["expected_scene"]
        scene = build_scene(config)
        if scene != expected:
            raise AssertionError(f'{case["id"]}: Python oracle differs from exported expected scene')
        encoded = json.dumps(scene, separators=(",", ":"), ensure_ascii=False,
                             allow_nan=False)
        if json.loads(encoded) != expected:
            raise AssertionError(f'{case["id"]}: JSON round-trip differs from expected scene')

        operations = (
            ("scene_construction_only", lambda: build_scene(config)),
            ("json_encoding_only", lambda: json.dumps(
                scene, separators=(",", ":"), ensure_ascii=False, allow_nan=False)),
            ("scene_construction_plus_json_encoding", lambda: json.dumps(
                build_scene(config), separators=(",", ":"), ensure_ascii=False,
                allow_nan=False)),
        )
        for measurement, operation in operations:
            results.append({
                "workload": case["id"], "candidate": "Python",
                "status": "passed", "measurement": measurement,
                "scene_cells": len(expected["cells"]),
                "scene_json_bytes": len(encoded.encode("utf-8")),
                **measure(operation, stress=case["id"] in ("B03", "B04")),
            })

    payload = {
        "schema_version": 3, "suite": "corrplot-ng-cross-language-scene-v1",
        "candidate": "Python",
        "environment": {
            "python": sys.version, "platform": platform.platform(),
            "machine": platform.machine(), "processor": platform.processor(),
        },
        "timing_method": {
            "clock": "time.perf_counter_ns", "target_sample_ms": TARGET_SAMPLE_MS,
            "samples": SAMPLES, "warmups": WARMUPS,
            "stress_samples": 5, "stress_warmups": 1, "stress_batch_size": 1,
            "batching": "calibrate batch size for regular workloads; fixed batch size 1 for B03/B04",
        },
        "results": results,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
