#!/usr/bin/env python3
"""Compare compatible repeated shared-core benchmark runs without hiding raw samples."""
from __future__ import annotations

import argparse
import hashlib
import json
import statistics
from pathlib import Path
from typing import Any

SUPPORTED_SUITE = "corrplot-ng-initial-shared-core"
SUPPORTED_SCHEMA = 1


def canonical_hash(value: Any) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def environment_key(environment: dict[str, Any]) -> dict[str, Any]:
    """Conservative compatibility key; ignore kernel patch/build identifiers."""
    python_version = str(environment.get("python", "")).split()[0].split(".")
    if len(python_version) < 2:
        raise ValueError("environment.python must include a major and minor version")
    platform_value = str(environment.get("platform", ""))
    os_family = platform_value.split("-", 1)[0].split(" ", 1)[0]
    machine = environment.get("machine")
    if not os_family or not machine:
        raise ValueError("environment.platform and environment.machine are required")
    return {
        "python_implementation": environment.get("implementation"),
        "python_major_minor": ".".join(python_version[:2]),
        "os_family": os_family,
        "machine": machine,
        "processor": environment.get("processor") or None,
        "cpu_count": environment.get("cpu_count"),
    }


def load_run(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("suite") != SUPPORTED_SUITE:
        raise ValueError(f"{path}: unsupported suite {payload.get('suite')!r}; expected {SUPPORTED_SUITE!r}")
    if payload.get("schema_version") != SUPPORTED_SCHEMA:
        raise ValueError(f"{path}: unsupported schema_version {payload.get('schema_version')!r}")
    if not isinstance(payload.get("results"), list) or not isinstance(payload.get("inputs"), dict):
        raise ValueError(f"{path}: expected 'inputs' and 'results' collections")
    key = environment_key(payload.get("environment") or {})
    fingerprints = {name: canonical_hash(config) for name, config in payload["inputs"].items()}
    return {"path": str(path), "payload": payload, "environment_key": key,
            "fingerprints": fingerprints}


def summarize(samples: list[float]) -> dict[str, float | int]:
    if not samples or any(not isinstance(value, (int, float)) for value in samples):
        raise ValueError("samples_ms must be a non-empty numeric array")
    ordered = sorted(float(value) for value in samples)
    q1, q3 = statistics.quantiles(ordered, n=4, method="inclusive")[0::2] if len(ordered) > 1 else (ordered[0], ordered[0])
    return {
        "sample_count": len(ordered),
        "median_ms": statistics.median(ordered),
        "iqr_ms": q3 - q1,
        "min_ms": ordered[0],
        "max_ms": ordered[-1],
    }


def compare_runs(paths: list[Path]) -> dict[str, Any]:
    if len(paths) < 2:
        raise ValueError("provide at least two benchmark result files")
    runs = [load_run(path) for path in paths]
    first = runs[0]
    for run in runs[1:]:
        if run["payload"].get("seed") != first["payload"].get("seed"):
            raise ValueError(f"incompatible workload generator seed: {first['path']} vs {run['path']}")
        if run["environment_key"] != first["environment_key"]:
            raise ValueError(
                "incompatible benchmark environments: "
                f"{first['path']}={first['environment_key']} vs "
                f"{run['path']}={run['environment_key']}"
            )

    grouped: dict[tuple[str, str], list[dict[str, Any]]] = {}
    for run_index, run in enumerate(runs):
        for result in run["payload"]["results"]:
            if result.get("status") != "passed":
                continue
            workload, measurement = result.get("workload"), result.get("measurement")
            if not workload or not measurement:
                continue
            fingerprint = run["fingerprints"].get(workload)
            if fingerprint is None:
                raise ValueError(f"{run['path']}: no input fingerprint for {workload}")
            samples = result.get("samples_ms")
            stats = summarize(samples)
            grouped.setdefault((workload, measurement), []).append({
                "run_index": run_index,
                "source": run["path"],
                "source_commit": run["payload"].get("source_commit"),
                "measured_at": run["payload"].get("measured_at"),
                "input_sha256": fingerprint,
                **stats,
                "samples_ms": list(samples),
            })

    comparisons = []
    for (workload, measurement), observations in sorted(grouped.items()):
        if len(observations) != len(runs):
            continue
        if len({item["input_sha256"] for item in observations}) != 1:
            continue
        baseline = observations[0]["median_ms"]
        for item in observations:
            item["change_vs_first_percent"] = (
                None if baseline == 0 else ((item["median_ms"] - baseline) / baseline) * 100
            )
        comparisons.append({
            "workload": workload,
            "measurement": measurement,
            "input_sha256": observations[0]["input_sha256"],
            "observations": observations,
        })

    return {
        "schema_version": 1,
        "suite": SUPPORTED_SUITE,
        "comparison_policy": {
            "environment": "same Python implementation and major/minor, OS family, machine, processor, and CPU count",
            "workload": "same deterministic input SHA-256 and generator seed",
            "measurement": "same workload and measurement label; only passed correctness-gated results",
            "baseline": "first input file; positive change means slower median",
            "raw_samples": "preserved for every observation",
        },
        "environment": first["environment_key"],
        "runs": [{
            "source": run["path"],
            "source_commit": run["payload"].get("source_commit"),
            "measured_at": run["payload"].get("measured_at"),
            "iterations": run["payload"].get("iterations"),
        } for run in runs],
        "comparisons": comparisons,
        "omitted_groups": "A workload/measurement is omitted unless every run passed and the input fingerprint matches.",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", nargs="+", type=Path, help="benchmark JSON files, baseline first")
    parser.add_argument("--output", type=Path, required=True, help="write comparison JSON to this path")
    args = parser.parse_args()
    try:
        report = compare_runs(args.results)
    except (OSError, json.JSONDecodeError, ValueError) as error:
        parser.error(str(error))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Compared {len(report['runs'])} runs; {len(report['comparisons'])} comparable workload/measurement groups.")
    print(f"Report written to {args.output}")


if __name__ == "__main__":
    main()
