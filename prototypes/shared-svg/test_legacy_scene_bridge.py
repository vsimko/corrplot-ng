"""Verify real legacy corrplot outputs can feed the scene-v1 builder.

The R oracle exports original-order and alphabetically reordered matrices.
This test exercises the serialization boundary and checks ordered labels,
row-major values, and preservation of missing cells for both cases.
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
    assert set(cases) == {"original_order", "alphabetic_order"}

    for name, config in cases.items():
        verify_case(name, config)

    print(f"Legacy-to-scene bridge passed for {len(cases)} legacy orderings")


if __name__ == "__main__":
    main()
