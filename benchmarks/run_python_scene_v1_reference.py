#!/usr/bin/env python3
"""Measure Python scene-v1 construction plus JSON encoding on shared fixtures."""
from __future__ import annotations
import argparse
import json
import platform
import statistics
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from scene import build_scene


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--iterations", type=int, default=20)
    args = parser.parse_args()
    if args.iterations < 20:
        parser.error("--iterations must be at least 20")
    fixture = json.loads(args.fixtures.read_text(encoding="utf-8"))
    results = []
    for case in fixture["workloads"]:
        config = {"labels": case["labels"], "matrix": case["matrix"],
                  "annotation": case["annotation"]}
        expected = case["expected_scene"]
        def operation() -> str:
            return json.dumps(build_scene(config), separators=(",", ":"),
                              ensure_ascii=False, allow_nan=False)
        started = time.perf_counter_ns()
        first = operation()
        first_ms = (time.perf_counter_ns() - started) / 1e6
        if json.loads(first) != expected:
            raise AssertionError(f'{case["id"]}: Python oracle differs from exported expected scene')
        for _ in range(3):
            operation()
        batch = 100 if len(case["labels"]) == 20 else 10
        samples = []
        for _ in range(args.iterations):
            started = time.perf_counter_ns()
            for _ in range(batch):
                operation()
            samples.append((time.perf_counter_ns() - started) / 1e6 / batch)
        ordered = sorted(samples)
        q1, q3 = statistics.quantiles(ordered, n=4, method="inclusive")[0::2]
        results.append({
            "workload": case["id"], "candidate": "Python",
            "status": "passed", "measurement": "scene_construction_plus_json_encoding",
            "first_call_ms": first_ms, "median_ms": statistics.median(ordered),
            "iqr_ms": q3-q1, "min_ms": min(ordered), "max_ms": max(ordered),
            "samples_ms": samples, "batch_size": batch,
            "scene_cells": len(expected["cells"]), "scene_json_bytes": len(first.encode("utf-8")),
        })
    payload = {
        "schema_version": 1, "suite": "corrplot-ng-cross-language-scene-v1",
        "candidate": "Python", "iterations": args.iterations, "warmup_iterations": 3,
        "environment": {"python": sys.version, "platform": platform.platform(),
                        "machine": platform.machine(), "processor": platform.processor()},
        "results": results,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
