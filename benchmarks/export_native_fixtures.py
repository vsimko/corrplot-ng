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
    invalid_specs = [
        {
            "id": "invalid-dimensions",
            "labels": ["alpha", "beta"],
            "matrix": [[1.0]],
            "annotation": "",
            "expected_error_pattern": "matrix|dimension",
            "python_matrix": [[1.0]],
        },
        {
            "id": "invalid-out-of-range",
            "labels": ["alpha", "beta"],
            "matrix": [[1.0, 1.2], [1.2, 1.0]],
            "annotation": "",
            "expected_error_pattern": "between -1 and 1",
            "python_matrix": [[1.0, 1.2], [1.2, 1.0]],
        },
        {
            "id": "invalid-nan",
            "labels": ["alpha", "beta"],
            "matrix": [[1.0, "NaN"], ["NaN", 1.0]],
            "annotation": "",
            "expected_error_pattern": "finite",
            "python_matrix": [[1.0, float("nan")], [float("nan"), 1.0]],
        },
    ]
    for case in invalid_specs:
        invalid_config = {
            "labels": case["labels"],
            "matrix": case["python_matrix"],
            "annotation": case["annotation"],
        }
        try:
            build_scene(invalid_config)
        except ValueError as error:
            import re
            if not re.search(case["expected_error_pattern"], str(error), re.IGNORECASE):
                raise AssertionError(
                    f"{case['id']}: Python reference returned unexpected error: {error}"
                ) from error
        else:
            raise AssertionError(f"{case['id']}: Python reference accepted invalid input")
        case.pop("python_matrix")

    payload = {
        "schema_version": 1,
        "generator": "benchmarks/export_native_fixtures.py",
        "seed": 20261029,
        "workloads": workloads,
        "invalid_cases": invalid_specs,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2, allow_nan=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
