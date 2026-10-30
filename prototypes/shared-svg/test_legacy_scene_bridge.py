"""Verify legacy corrplot outputs of different dimensions feed scene-v1.

The R oracle exports 2x2, 3x3 (two label orders), and 4x4 matrices.
This exercises the serialization boundary and checks ordered labels,
row-major values, and preservation of missing cells.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from scene import build_scene


def verify_case(name: str, config: dict) -> None:
    scene = build_scene(config)
    labels = config["labels"]
    matrix = config["matrix"]
    size = len(labels)

    assert size in (2, 3, 4), (name, size)
    assert len(matrix) == size
    assert [item["text"] for item in scene["labels"]] == labels
    assert [(cell["row"], cell["column"]) for cell in scene["cells"]] == [
        (row, column) for row in range(size) for column in range(size)
    ]
    assert [cell["value"] for cell in scene["cells"]] == [
        value for row in matrix for value in row
    ]
    assert scene["annotation"] == config["annotation"]
    missing = sum(cell["value"] is None for cell in scene["cells"])
    assert missing == sum(
        value is None for row in matrix for value in row
    )
    assert scene["width"] == 104 + size * 84 + 24
    assert scene["height"] == 76 + size * 84 + 64

    if name == "perfect_correlations_and_constant":
        assert matrix[0][1] == 1
        assert matrix[0][2] == -1
        assert all(matrix[row][3] is None for row in range(3))
        assert matrix[3][3] == 1
    elif name == "fully_missing_variable":
        assert all(matrix[row][2] is None for row in range(size))
        assert all(matrix[2][column] is None for column in range(size))
        assert sum(value is None for row in matrix for value in row) == 5

    print(
        f"{name}: {size} labels, {len(scene['cells'])} cells, "
        f"{missing} missing values"
    )


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: test_legacy_scene_bridge.py LEGACY_INPUT.json")

    source = Path(sys.argv[1])
    payload = json.loads(source.read_text(encoding="utf-8"))
    cases = payload["cases"]
    expected_cases = {
        "three_original_order",
        "three_alphabetic_order",
        "two_variables",
        "four_variables_with_missing",
        "perfect_correlations_and_constant",
        "fully_missing_variable",
    }
    assert set(cases) == expected_cases

    for name, config in cases.items():
        verify_case(name, config)

    print(f"Legacy-to-scene bridge passed for {len(cases)} matrix cases")


if __name__ == "__main__":
    main()
