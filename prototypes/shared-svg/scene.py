"""Renderer-neutral scene model for correlation plots.

This module owns validation and deterministic layout, not SVG serialization.
"""
from __future__ import annotations

from typing import Any
from math import ceil, isfinite

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
    title = config.get("title", "Correlation matrix")
    description = config.get("description", "")
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
    if not isinstance(title, str):
        raise ValueError("title must be a string")
    if not isinstance(description, str):
        raise ValueError("description must be a string")

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

    method = config.get("method", "circle")
    if method not in ("circle", "number"):
        raise ValueError("method must be 'circle' or 'number' in this scene slice")
    display_type = config.get("display_type", "full")
    if display_type not in ("full", "upper", "lower", "diagonal"):
        raise ValueError("display_type must be full, upper, lower, or diagonal")
    show_diagonal = config.get("show_diagonal", True)
    if not isinstance(show_diagonal, bool):
        raise ValueError("show_diagonal must be a boolean")
    supported_methods = ("circle", "number")
    upper_method = config.get("upper_method", method)
    lower_method = config.get("lower_method", method)
    diagonal_method = config.get("diagonal_method", method)
    for name, selected in (("upper_method", upper_method), ("lower_method", lower_method),
                           ("diagonal_method", diagonal_method)):
        if selected not in supported_methods:
            raise ValueError(f"{name} must be one of {', '.join(supported_methods)}")
    if display_type == "diagonal" and not show_diagonal:
        raise ValueError("diagonal display requires show_diagonal=true")

    method = config.get("method", "circle")
    if method not in ("circle", "number"):
        raise ValueError("method must be 'circle' or 'number' in this scene slice")
    display_type = config.get("display_type", "full")
    if display_type not in ("full", "upper", "lower", "diagonal"):
        raise ValueError("display_type must be full, upper, lower, or diagonal")
    show_diagonal = config.get("show_diagonal", True)
    if not isinstance(show_diagonal, bool):
        raise ValueError("show_diagonal must be a boolean")
    supported_methods = ("circle", "number")
    upper_method = config.get("upper_method", method)
    lower_method = config.get("lower_method", method)
    diagonal_method = config.get("diagonal_method", method)
    for name, selected in (("upper_method", upper_method), ("lower_method", lower_method),
                           ("diagonal_method", diagonal_method)):
        if selected not in supported_methods:
            raise ValueError(f"{name} must be one of {', '.join(supported_methods)}")
    if display_type == "diagonal" and not show_diagonal:
        raise ValueError("diagonal display requires show_diagonal=true")

    has_region_layout = any(key in config for key in (
        "display_type", "show_diagonal", "upper_method", "lower_method", "diagonal_method"
    ))
    cells = []
    cell_colors = config.get("cell_colors")
    if cell_colors is not None:
        if not isinstance(cell_colors, list) or len(cell_colors) != len(labels) ** 2:
            raise ValueError("cell_colors must contain one color per matrix cell")
        if any(not isinstance(color, str) or len(color) != 7 or color[0] != "#" for color in cell_colors):
            raise ValueError("cell_colors must contain six-digit hex colors")

    for row_index, row in enumerate(matrix):
        for column_index, value in enumerate(row):
            x, y = left + column_index * cell, top + row_index * cell
            is_diagonal = row_index == column_index
            visible = (
                (display_type == "full")
                or (display_type == "upper" and row_index <= column_index)
                or (display_type == "lower" and row_index >= column_index)
                or (display_type == "diagonal" and is_diagonal)
            )
            if is_diagonal and not show_diagonal:
                visible = False
            selected_method = diagonal_method if is_diagonal else (
                upper_method if row_index < column_index else lower_method
            )
            item = {
                "row": row_index, "column": column_index, "value": value,
                "x": x, "y": y, "center_x": x + cell / 2,
                "center_y": y + cell / 2, "size": cell,
            }
            if has_region_layout:
                item.update({
                    "visible": visible, "method": selected_method,
                    "region": "diagonal" if is_diagonal else ("upper" if row_index < column_index else "lower"),
                })
            if cell_colors is not None:
                item["color"] = cell_colors[row_index * len(labels) + column_index]
            cells.append(item)

    # Coefficient/value text is semantic scene data, not an SVG-specific detail.
    add_coef_color = config.get("add_coef_color")
    if add_coef_color is not None and (not isinstance(add_coef_color, str) or not add_coef_color.strip()):
        raise ValueError("add_coef_color must be a non-empty color string or null")
    add_coef_as_percent = config.get("add_coef_as_percent", False)
    if not isinstance(add_coef_as_percent, bool):
        raise ValueError("add_coef_as_percent must be a boolean")
    digits = config.get("number_digits")
    if digits is None:
        digits = 0 if add_coef_as_percent else 2
    if isinstance(digits, bool) or not isinstance(digits, (int, float)) or not float(digits).is_integer() or digits < 0:
        raise ValueError("number_digits must be a non-negative integer")
    digits = int(digits)
    number_cex = config.get("number_cex", 1)
    if isinstance(number_cex, bool) or not isinstance(number_cex, (int, float)) or not isfinite(number_cex) or not number_cex > 0:
        raise ValueError("number_cex must be greater than 0")
    number_font = config.get("number_font", 2)
    if isinstance(number_font, bool) or not isinstance(number_font, (int, float)) or number_font not in (1, 2, 3, 4):
        raise ValueError("number_font must be one of 1, 2, 3, or 4")
    number_font = int(number_font)

    def format_number(value: float) -> str:
        # Python's round and the legacy R round both use ties-to-even semantics.
        scaled = value * (100 if add_coef_as_percent else 1)
        return f"{round(scaled, digits):.{digits}f}"

    text_annotations = []
    for cell_item in cells:
        row_index, column_index, value = cell_item["row"], cell_item["column"], cell_item["value"]
        if value is None or not cell_item.get("visible", True):
            continue
        if cell_item.get("method", method) == "number":
            role = "value"
            color = (
                cell_item.get("color") or ("#2166ac" if value > 0 else "#b2182b" if value < 0 else "#bdbdbd")
            )
        elif add_coef_color is not None:
            role = "coefficient"
            color = add_coef_color
        else:
            continue
        text_annotations.append({
            "row": row_index, "column": column_index, "role": role,
            "text": format_number(value), "color": color,
            "font_size": LABEL_FONT_SIZE * float(number_cex),
            "font_weight": "bold" if number_font in (2, 4) else "normal",
            "font_style": "italic" if number_font in (3, 4) else "normal",
        })

    legend = config.get("legend")
    if legend is not None:
        if not isinstance(legend, list) or any(
            not isinstance(entry, dict)
            or not isinstance(entry.get("label"), str)
            or not isinstance(entry.get("color"), str)
            or len(entry["color"]) != 7 or entry["color"][0] != "#"
            for entry in legend
        ):
            raise ValueError("legend must contain label and six-digit color entries")

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
        "glyph_method": method, "text_annotations": text_annotations,
        "title": title, "description": description,
        "style": {
            "positive_color": "#2166ac", "negative_color": "#b2182b",
            "zero_color": "#bdbdbd", "missing_fill": "#f2f2f2",
            "grid_color": "#e0e0e0",
        },
    }
    if has_region_layout:
        scene["display_type"] = display_type
        scene["show_diagonal"] = show_diagonal
    if layout:
        scene["typography"] = {
            "label_font_family": font_family,
            "label_font_size": font_size,
            "label_font_weight": font_weight,
            "column_label_orientation": orientation,
            "label_collision": collision,
        }
    if legend is not None:
        scene["legend"] = legend
    return scene
