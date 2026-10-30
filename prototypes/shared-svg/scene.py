"""Renderer-neutral scene model for correlation plots.

This module owns validation and deterministic layout, not SVG serialization.
"""
from __future__ import annotations

from typing import Any
from math import ceil

LABEL_FONT_SIZE = 13
LABEL_CHAR_WIDTH = LABEL_FONT_SIZE * 0.6  # Backwards-compatible estimate used by the legacy bridge.
LABEL_GAP = 12
DEFAULT_CELL_SIZE = 84


def _number(value: Any, name: str, minimum: float, maximum: float | None = None) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"layout.{name} must be a number")
    if value < minimum or (maximum is not None and value > maximum):
        limit = f" between {minimum} and {maximum}" if maximum is not None else f" at least {minimum}"
        raise ValueError(f"layout.{name} must be{limit}")
    return float(value)


def build_scene(config: dict[str, Any]) -> dict[str, Any]:
    """Validate a correlation-plot config and return a deterministic plot scene."""
    if not isinstance(config, dict):
        raise ValueError("config must be an object")

    labels = config.get("labels")
    matrix = config.get("matrix")
    annotation = config.get("annotation", "")
    layout = config.get("layout", {})
    if not isinstance(layout, dict):
        raise ValueError("layout must be an object")
    if not isinstance(labels, list) or not labels:
        raise ValueError("labels must be a non-empty list")
    if any(not isinstance(label, (str, int, float)) or isinstance(label, bool) for label in labels):
        raise ValueError("labels must contain only strings or numbers")
    if not isinstance(matrix, list) or len(matrix) != len(labels):
        raise ValueError("matrix must be square and match labels")
    if any(not isinstance(row, list) or len(row) != len(labels) for row in matrix):
        raise ValueError("matrix must be square and match labels")
    if not isinstance(annotation, str):
        raise ValueError("annotation must be a string")

    for row in matrix:
        for value in row:
            if value is None:
                continue
            if isinstance(value, bool) or not isinstance(value, (int, float)):
                raise ValueError("correlation values must be numbers or null")
            if not -1 <= value <= 1:
                raise ValueError("correlation values must be null or between -1 and 1")

    font_size = _number(layout.get("label_font_size", LABEL_FONT_SIZE), "label_font_size", 6, 48)
    cell = _number(layout.get("cell_size", DEFAULT_CELL_SIZE), "cell_size", 8, 240)
    min_cell = _number(layout.get("min_cell_size", 18), "min_cell_size", 8, 240)
    max_cell = _number(layout.get("max_cell_size", 120), "max_cell_size", min_cell, 240)
    if "width" in layout:
        available = _number(layout["width"], "width", 120) - 24
        cell = min(cell, available / len(labels))
    if "height" in layout:
        available = _number(layout["height"], "height", 120) - 64
        cell = min(cell, available / len(labels))
    cell = min(max(cell, min_cell), max_cell)

    font_family = layout.get("label_font_family", "sans-serif")
    if not isinstance(font_family, str) or not font_family.strip():
        raise ValueError("layout.label_font_family must be a non-empty string")
    font_weight = layout.get("label_font_weight", "normal")
    if font_weight not in ("normal", "bold"):
        raise ValueError("layout.label_font_weight must be 'normal' or 'bold'")
    orientation = layout.get("column_label_orientation", "vertical")
    if orientation not in ("vertical", "horizontal"):
        raise ValueError("layout.column_label_orientation must be 'vertical' or 'horizontal'")
    collision = layout.get("label_collision", "preserve")
    if collision not in ("preserve", "wrap", "truncate", "omit"):
        raise ValueError("layout.label_collision must be preserve, wrap, truncate, or omit")

    label_texts = [str(label) for label in labels]
    # Conservative deterministic estimate; does not depend on browser/font APIs.
    char_width = font_size * 0.6
    max_label_width = max(len(label) * char_width for label in label_texts)
    row_label_width = min(max_label_width, cell) if collision == "truncate" else max_label_width
    left = max(104, int(row_label_width + LABEL_GAP * 2))
    horizontal_lines = max(1, ceil(max_label_width / max(cell, 1))) if collision == "wrap" else 1
    top = max(76, int((max_label_width if orientation == "vertical" else horizontal_lines * font_size * 1.1) + 32))
    width = max(left + len(labels) * cell + 24, _number(layout.get("width", left + len(labels) * cell + 24), "width", 120))
    height = max(top + len(labels) * cell + 64, _number(layout.get("height", top + len(labels) * cell + 64), "height", 120))

    cells = []
    for row_index, row in enumerate(matrix):
        for column_index, value in enumerate(row):
            x, y = left + column_index * cell, top + row_index * cell
            cells.append({
                "row": row_index, "column": column_index, "value": value,
                "x": x, "y": y, "center_x": x + cell / 2,
                "center_y": y + cell / 2, "size": cell,
            })

    label_items = []
    for index, label in enumerate(label_texts):
        label_items.append({
            "text": label, "row_x": left - LABEL_GAP,
            "row_y": top + index * cell + cell / 2,
            "column_x": left + index * cell + cell / 2,
            "column_y": top - (16 if orientation == "vertical" else font_size),
        })

    scene = {
        "schema_version": 1, "kind": "correlation_plot",
        "width": width, "height": height, "labels": label_items, "cells": cells,
        "annotation": annotation,
        "style": {
            "positive_color": "#2166ac", "negative_color": "#b2182b",
            "zero_color": "#bdbdbd", "missing_fill": "#f2f2f2",
            "grid_color": "#e0e0e0",
        },
    }
    if layout:
        scene["typography"] = {
            "label_font_family": font_family,
            "label_font_size": font_size,
            "label_font_weight": font_weight,
            "column_label_orientation": orientation,
            "label_collision": collision,
        }
    return scene
