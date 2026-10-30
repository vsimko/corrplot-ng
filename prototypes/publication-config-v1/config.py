"""Validation and stable normalization for publication configuration v1."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from color_scale import build_color_scale
from ordering import resolve_ordering


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
        import math
        for row_index, row in enumerate(values):
            for column_index, value in enumerate(row):
                if value is not None and (isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value)):
                    raise ValueError(f"$.input.values[{row_index}][{column_index}]: expected a finite number or null")
        if normalized["matrix"]["kind"] == "correlation":
            _validate_symmetric(values, "$.input.values", allow_null=True)
            _validate_diagonal(values, "$.input.values")

    semantics = _resolve_matrix_scale(normalized)
    scale_spec = _resolve_color_scale(normalized, semantics)
    build_color_scale(scale_spec, normalized["colors"]["missing"])

    significance = normalized["plot"]["significance"]
    if significance["enabled"]:
        p_values = significance["p_values"]
        size = len(source["labels"]) if source["kind"] == "inline_matrix" else None
        if size is not None:
            _validate_square(p_values, size, "$.plot.significance.p_values")
            _validate_symmetric(p_values, "$.plot.significance.p_values", allow_null=False)

    return normalized


def _resolve_matrix_scale(config: dict[str, Any]) -> dict[str, Any]:
    """Resolve the visual domain while preserving original matrix values."""
    if config["matrix"]["kind"] == "correlation":
        return {"mode": "correlation", "color_limits": [-1.0, 1.0],
                "color_mapping": "diverging", "palette": "RdBu",
                "normalization": {"intercept": 0.0, "zoom": 1.0}}
    import math
    explicit = config["matrix"].get("col_limits")
    if config["input"]["kind"] == "matrix_csv":
        if explicit is None:
            raise ValueError("$.matrix.col_limits: CSV-backed general_numeric requires explicit limits until the matrix is loaded")
        if (not all(math.isfinite(float(value)) for value in explicit) or explicit[0] >= explicit[1]):
            raise ValueError("$.matrix.col_limits: expected two finite, strictly increasing values")
        lower, upper = map(float, explicit)
        observed = None
    else:
        observed = [float(value) for row in config["input"]["values"] for value in row if value is not None]
        if not observed:
            raise ValueError("$.input.values: cannot resolve automatic color limits from an all-missing matrix")
        if any(not math.isfinite(value) for value in observed):
            raise ValueError("$.input.values: values must be finite or null")
    if explicit is not None:
        if (not all(math.isfinite(float(value)) for value in explicit) or explicit[0] >= explicit[1]):
            raise ValueError("$.matrix.col_limits: expected two finite, strictly increasing values")
        lower, upper = map(float, explicit)
        if observed is not None and (min(observed) < lower or max(observed) > upper):
            raise ValueError("$.matrix.col_limits: limits must cover every observed matrix value")
    elif observed is not None:
        lower, upper = min(observed), max(observed)
        if lower == upper:
            padding = max(0.5, abs(lower) * 0.05)
            lower, upper = lower - padding, upper + padding
    mixed_sign = (lower < 0 < upper) if observed is None else (min(observed) < 0 < max(observed))
    if mixed_sign:
        zoom, intercept, family = 1.0 / max(abs(lower), abs(upper)), 0.0, "diverging"
    else:
        zoom, intercept, family = 1.0 / (upper - lower), -lower, "sequential"
    return {"mode": "general_numeric", "color_limits": [lower, upper],
            "color_mapping": family, "palette": "RdBu" if family == "diverging" else "YlOrBr",
            "normalization": {"intercept": intercept, "zoom": zoom}}



def _resolve_color_scale(config: dict[str, Any], semantics: dict[str, Any]) -> dict[str, Any]:
    """Select an explicit scale or the legacy-compatible default palette."""
    explicit = config["colors"].get("scale")
    if explicit is not None:
        if explicit.get("type") == "sequential" and semantics["color_mapping"] != "sequential":
            raise ValueError("$.colors.scale: sequential scales require a one-sided general numeric matrix")
        semantics["palette"] = "custom"
        return explicit
    if semantics["color_mapping"] == "sequential":
        anchors = ["#ffffe5", "#fff7bc", "#fee391", "#fec44f", "#fe9929",
                   "#ec7014", "#cc4c02", "#993404", "#662506"]
        return {"type": "sequential", "interpolation": "rgb",
                "stops": [{"value": index / (len(anchors) - 1), "color": color}
                          for index, color in enumerate(anchors)]}
    colors = config["colors"]
    return {"type": "continuous", "interpolation": "rgb", "stops": [
        {"value": -1, "color": colors["negative"]},
        {"value": 0, "color": colors["neutral"]},
        {"value": 1, "color": colors["positive"]},
    ]}


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

    ordering = normalized["plot"]["ordering"]
    if normalized["matrix"]["kind"] == "general_numeric" and ordering not in ("original", "alphabetical"):
        raise ValueError("$.plot.ordering: general_numeric supports only original or alphabetical ordering")
    permutation = resolve_ordering(source["labels"], source["values"], ordering)
    ordered_labels = [source["labels"][index] for index in permutation]
    values = [[source["values"][row][column] for column in permutation] for row in permutation]

    colors = normalized["colors"]
    semantics = _resolve_matrix_scale(normalized)
    scale_spec = _resolve_color_scale(normalized, semantics)
    scale = build_color_scale(scale_spec, colors["missing"])
    transform = semantics["normalization"]
    cell_colors = [scale.map_value(None if value is None else
                    (value + transform["intercept"]) * transform["zoom"])
                   for row in values for value in row]

    def value_label(value: float) -> str:
        return "0" if value == 0 else f"{value:+g}"

    legend = []
    for entry in scale.legend_entries():
        if entry["kind"] == "stop":
            label_value = entry["value"]
            if semantics["mode"] == "general_numeric":
                label_value = label_value / transform["zoom"] - transform["intercept"]
            label = value_label(label_value)
        elif entry["kind"] == "interval":
            closing = "]" if entry["include_upper"] else ")"
            lower_label, upper_label = entry["lower"], entry["upper"]
            if semantics["mode"] == "general_numeric":
                lower_label = lower_label / transform["zoom"] - transform["intercept"]
                upper_label = upper_label / transform["zoom"] - transform["intercept"]
            label = f"[{value_label(lower_label)}, {value_label(upper_label)}{closing}"
        else:
            label = "Missing"
        legend.append({"kind": entry["kind"], "label": label, "color": entry["color"]})

    result = {
        "matrix_mode": semantics["mode"],
        "labels": ordered_labels,
        "matrix": values,
        "cell_colors": cell_colors,
        "legend": legend,
        "matrix_semantics": semantics,
        "variable_order": {
            "strategy": ordering,
            "indices": permutation,
            "labels": list(source["labels"]),
        },
    }
    significance = normalized["plot"]["significance"]
    if significance["enabled"]:
        p_values = significance["p_values"]
        result["p_values"] = [
            [p_values[row][column] for column in permutation]
            for row in permutation
        ]
    return result


def canonical_json(config: dict[str, Any]) -> str:
    """Return stable compact JSON after validation, without rounding numbers."""
    normalized = validate_config(config)
    return json.dumps(
        normalized, sort_keys=True, separators=(",", ":"),
        ensure_ascii=False, allow_nan=False,
    )
