"""Verify legacy corrplot outputs of different dimensions feed scene-v1.

The R oracle exports 2x2, 3x3 (two label orders), and 4x4 matrices.
This exercises the serialization boundary and checks ordered labels,
row-major values, preservation of missing cells, and adaptive label margins.
"""
from __future__ import annotations

import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

from renderer import render_scene_svg
from scene import LABEL_CHAR_WIDTH, LABEL_GAP, build_scene


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

    estimated_label_width = max(len(str(label)) * LABEL_CHAR_WIDTH for label in labels)
    expected_left = max(104, int(estimated_label_width + LABEL_GAP * 2))
    expected_top = max(76, int(estimated_label_width + 32))
    assert scene["cells"][0]["x"] == expected_left
    assert scene["cells"][0]["y"] == expected_top
    assert scene["width"] == expected_left + size * 84 + 24
    assert scene["height"] == expected_top + size * 84 + 64
    assert all(label["row_x"] < expected_left for label in scene["labels"])
    assert all(label["column_y"] < expected_top for label in scene["labels"])

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


def verify_svg_structure(name: str, config: dict, method: str) -> None:
    """Classify SVG markup/layout failures separately from semantic failures."""
    case_config = {**config, "method": method}
    try:
        scene = build_scene(case_config)
        svg = render_scene_svg(scene)
        root = ET.fromstring(svg)
        namespace = "{http://www.w3.org/2000/svg}"
        labels = config["labels"]
        expected_cells = len(labels) * len(labels)
        missing_count = sum(value is None for row in config["matrix"] for value in row)

        assert root.tag == f"{namespace}svg", "root element is not SVG"
        assert root.attrib.get("role") == "img", "SVG accessibility role is missing"
        assert float(root.attrib["width"]) == scene["width"], "SVG width differs from scene"
        assert float(root.attrib["height"]) == scene["height"], "SVG height differs from scene"
        assert len(scene["cells"]) == expected_cells, "scene cell count differs from matrix dimensions"
        assert all(
            cell.get("encoding", {}).get("kind") == method
            for cell in scene["cells"] if cell["value"] is not None
        ), "one or more cells have the wrong glyph encoding"

        missing_nodes = [
            node for node in root.findall(f"{namespace}text") if node.text == "NA"
        ]
        assert len(missing_nodes) == missing_count, "SVG missing-cell labels differ from matrix missingness"

        annotation_nodes = [
            node for node in root.findall(f"{namespace}text")
            if node.text == config.get("annotation", "")
        ]
        assert len(annotation_nodes) == 1, "figure annotation is missing or duplicated"
        assert abs(float(annotation_nodes[0].attrib["y"]) - (scene["height"] - 22)) < 1e-9, (
            "figure annotation is not placed in the reserved footer"
        )

        shape_tags = {
            "circle": "circle", "square": "rect", "ellipse": "ellipse",
            "shade": "line", "color": "rect", "pie": "path",
        }
        if method == "number":
            assert scene.get("text_annotations"), "number method has no value annotations"
            assert all(item["role"] == "value" for item in scene["text_annotations"]), (
                "number method emitted non-value annotations"
            )
        else:
            tag = shape_tags[method]
            shapes = root.findall(f"{namespace}{tag}")
            if method in ("square", "color"):
                shapes = [item for item in shapes if item.attrib.get("stroke") == "none"]
            assert shapes, f"expected SVG {tag} encoding was not emitted"

        assert svg == render_scene_svg(build_scene(case_config)), "SVG output is not deterministic"
    except (AssertionError, KeyError, ValueError, ET.ParseError) as error:
        raise AssertionError(f"STRUCTURAL SVG failure for legacy case {name!r}, method {method!r}: {error}") from error


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

    methods = ("circle", "square", "ellipse", "number", "shade", "color", "pie")
    for name, config in cases.items():
        try:
            verify_case(name, config)
        except AssertionError as error:
            raise AssertionError(f"SEMANTIC compatibility failure for legacy case {name!r}: {error}") from error
        for method in methods:
            verify_svg_structure(name, config, method)

    print(
        f"Semantic bridge passed for {len(cases)} legacy matrices; "
        f"SVG structure passed for {len(cases) * len(methods)} case/method combinations"
    )


if __name__ == "__main__":
    main()
