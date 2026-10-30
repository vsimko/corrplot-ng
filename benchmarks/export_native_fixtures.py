#!/usr/bin/env python3
"""Export deterministic inputs and Python scene-v1 oracle for native benchmarks."""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "benchmarks"))
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from run_shared_core import make_cases
from scene import build_scene


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stress-only", action="store_true", help="export only B03/B04 stress fixtures")
    args = parser.parse_args()
    cases = make_cases(include_stress=args.stress_only)
    workloads = []
    case_ids = ("B03", "B04") if args.stress_only else ("B01", "B02", "B05", "B06", "B07")
    for case_id in case_ids:
        config = cases[case_id]
        scene = build_scene(config)
        workloads.append({
            "id": case_id,
            "labels": config["labels"],
            "matrix": config["matrix"],
            "annotation": config.get("annotation", ""),
            "expected_scene": scene,
        })
    payload = {
        "schema_version": 1,
        "generator": "benchmarks/export_native_fixtures.py",
        "seed": 20261029,
        "workloads": workloads,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2, allow_nan=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
