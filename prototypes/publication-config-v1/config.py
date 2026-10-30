"""Validation and stable normalization for publication configuration v1."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from color_scale import build_color_scale


SCHEMA_PATH = Path(__file__).with_name("schema.json")
SCHEMA = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))


def validate_config(config: dict[str, Any]) -> dict[str, Any]:
    """Validate schema and matrix-level invariants; return an independent copy."""
    errors = sorted(
        Draft202012Validator(SCHEMA).iter_errors(config),
        key=lambda error: list(map(str, error.path)),
    )
    if errors:
        error = errors[0]
        location = "$" + "".join(
            f"[{part}]" if isinstance(part, int) else f".{part}"
            for part in error.absolute_path
        )
        raise ValueError(f"{location}: {error.message}")

    normalized = deepcopy(config)
    source = normalized["input"]
    if source["kind"] == "inline_matrix":
        labels = source["labels"]
        values = source["values"]
        size = len(labels)
        if len(set(labels)) != size:
            raise ValueError("$.input.labels: labels must be unique")
        _validate_square(values, size, "$.input.values")
        _validate_symmetric(values, "$.input.values", allow_null=True)
        _validate_diagonal(values, "$.input.values")

    scale_spec = normalized["colors"].get("scale") or {
        "type": "continuous",
        "interpolation": "rgb",
        "stops": [
            {"value": -1, "color": normalized["colors"]["negative"]},
            {"value": 0, "color": normalized["colors"]["neutral"]},
            {"value": 1, "color": normalized["colors"]["positive"]},
        ],
    }
    build_color_scale(scale_spec, normalized["colors"]["missing"])

    significance = normalized["plot"]["significance"]
    if significance["enabled"]:
        p_values = significance["p_values"]
        size = len(source["labels"]) if source["kind"] == "inline_matrix" else None
        if size is not None:
            _validate_square(p_values, size, "$.plot.significance.p_values")
            _validate_symmetric(p_values, "$.plot.significance.p_values", allow_null=False)

    return normalized


def _validate_square(matrix: list[list[Any]], size: int, path: str) -> None:
    if len(matrix) != size or any(len(row) != size for row in matrix):
        raise ValueError(f"{path}: expected a square {size} by {size} matrix")


def _validate_diagonal(matrix: list[list[Any]], path: str) -> None:
    for index, row in enumerate(matrix):
        value = row[index]
        if value is not None and abs(value - 1.0) > 1e-8:
            raise ValueError(f"{path}[{index}][{index}]: correlation diagonal must be 1 or null")


def _validate_symmetric(matrix: list[list[Any]], path: str, allow_null: bool) -> None:
    for row in range(len(matrix)):
        for column in range(row + 1, len(matrix)):
            left, right = matrix[row][column], matrix[column][row]
            if left is None or right is None:
                if allow_null and left is None and right is None:
                    continue
                raise ValueError(f"{path}[{row}][{column}]: matrix must be symmetric")
            if abs(left - right) > 1e-8:
                raise ValueError(f"{path}[{row}][{column}]: matrix must be symmetric")


def to_scene_config(config: dict[str, Any]) -> dict[str, Any]:
    """Adapt an inline publication config to the renderer-neutral scene input."""
    normalized = validate_config(config)
    source = normalized["input"]
    if source["kind"] != "inline_matrix":
        raise ValueError("$.input: scene adaptation currently requires inline_matrix")

    colors = normalized["colors"]
    scale_spec = colors.get("scale") or {
        "type": "continuous",
        "interpolation": "rgb",
        "stops": [
            {"value": -1, "color": colors["negative"]},
            {"value": 0, "color": colors["neutral"]},
            {"value": 1, "color": colors["positive"]},
        ],
    }
    scale = build_color_scale(scale_spec, colors["missing"])
    values = source["values"]
    cell_colors = [scale.map_value(value) for row in values for value in row]

    def value_label(value: float) -> str:
        return "0" if value == 0 else f"{value:+g}"

    legend = []
    for entry in scale.legend_entries():
        if entry["kind"] == "stop":
            label = value_label(entry["value"])
        elif entry["kind"] == "interval":
            closing = "]" if entry["include_upper"] else ")"
            label = f"[{value_label(entry['lower'])}, {value_label(entry['upper'])}{closing}"
        else:
            label = "Missing"
        legend.append({"kind": entry["kind"], "label": label, "color": entry["color"]})

    return {
        "labels": source["labels"],
        "matrix": values,
        "cell_colors": cell_colors,
        "legend": legend,
    }


def canonical_json(config: dict[str, Any]) -> str:
    """Return stable compact JSON after validation, without rounding numbers."""
    normalized = validate_config(config)
    return json.dumps(
        normalized, sort_keys=True, separators=(",", ":"),
        ensure_ascii=False, allow_nan=False,
    )
