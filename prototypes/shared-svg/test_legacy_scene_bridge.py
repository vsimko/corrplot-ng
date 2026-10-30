"""Verify that real legacy corrplot output can feed the scene-v1 builder.

The R oracle writes a minimal JSON input from the pinned legacy result. This
test exercises the actual cross-language serialization boundary and checks
that the scene preserves ordered labels, values, and missing cells.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from scene import build_scene


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: test_legacy_scene_bridge.py LEGACY_INPUT.json")

    source = Path(sys.argv[1])
    config = json.loads(source.read_text(encoding="utf-8"))
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
    assert sum(cell["value"] is None for cell in scene["cells"]) == sum(
        value is None for row in matrix for value in row
    )

    print(
        f"Legacy-to-scene bridge passed: {size} labels, "
        f"{len(scene['cells'])} cells, "
        f"{sum(cell['value'] is None for cell in scene['cells'])} missing values"
    )


if __name__ == "__main__":
    main()
