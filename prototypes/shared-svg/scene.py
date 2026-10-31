"""Renderer-neutral scene model for correlation plots.

This module owns validation and deterministic layout, not SVG serialization.
"""
from __future__ import annotations

from typing import Any
from math import ceil, cos, isfinite, radians, sin
import textwrap

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




def _blend_hex(start: str, end: str, fraction: float) -> str:
    """Linearly interpolate two six-digit sRGB colors."""
    fraction = min(1.0, max(0.0, fraction))
    start_rgb = tuple(int(start[index:index + 2], 16) for index in (1, 3, 5))
    end_rgb = tuple(int(end[index:index + 2], 16) for index in (1, 3, 5))
    channels = [round(a + (b - a) * fraction) for a, b in zip(start_rgb, end_rgb)]
    return "#" + "".join(f"{channel:02x}" for channel in channels)


def _cell_encoding(cell: dict[str, Any], method: str, style: dict[str, str]) -> dict[str, Any]:
    value = cell["value"]
    visual_value = cell.get("normalized_value", value)
    magnitude = min(1.0, max(0.0, abs(visual_value)))
    sign = 1 if value > 0 else -1 if value < 0 else 0
    color = cell.get("color") or (
        style["positive_color"] if sign > 0 else
        style["negative_color"] if sign < 0 else style["zero_color"]
    )
    encoding: dict[str, Any] = {
        "kind": method, "magnitude": magnitude, "sign": sign, "color": color,
    }
    x, y, size = cell["x"], cell["y"], cell["size"]
    cx, cy = cell["center_x"], cell["center_y"]

    if method == "circle":
        encoding["radius"] = magnitude * size * 0.43
    elif method == "square":
        side = magnitude * size * 0.86
        encoding.update({
            "x": cx - side / 2, "y": cy - side / 2,
            "width": side, "height": side,
        })
    elif method == "ellipse":
        rx = size * 0.43
        encoding.update({
            "center_x": cx, "center_y": cy, "rx": rx,
            "ry": max(size * 0.015, rx * (1.0 - magnitude)),
            "rotation": 45 if sign > 0 else -45 if sign < 0 else 0,
        })
    elif method == "number":
        pass
    elif method == "shade":
        encoding["fill_opacity"] = 0.12 + 0.45 * magnitude
        if sign != 0:
            if sign > 0:
                line = {"x1": x, "y1": y + size, "x2": x + size, "y2": y}
            else:
                line = {"x1": x, "y1": y, "x2": x + size, "y2": y + size}
            encoding["line"] = {
                **line,
                "stroke_width": max(1.0, size * 0.055 * magnitude),
                "stroke_opacity": magnitude,
            }
    elif method == "color":
        if cell.get("color"):
            fill = cell["color"]
        else:
            target = style["positive_color"] if sign > 0 else style["negative_color"]
            fill = style["zero_color"] if sign == 0 else _blend_hex(style["zero_color"], target, magnitude)
        encoding["fill"] = fill
    elif method == "pie":
        radius = size * 0.43
        start_angle = -90.0
        end_angle = start_angle + sign * 360.0 * magnitude
        encoding.update({
            "center_x": cx, "center_y": cy, "radius": radius,
            "full_circle": magnitude >= 1.0 - 1e-12,
            "large_arc": magnitude > 0.5,
            "sweep_flag": 1 if sign > 0 else 0,
            "start_x": cx + radius * cos(radians(start_angle)),
            "start_y": cy + radius * sin(radians(start_angle)),
            "end_x": cx + radius * cos(radians(end_angle)),
            "end_y": cy + radius * sin(radians(end_angle)),
        })
    else:
        raise ValueError(f"unsupported cell method {method!r}")
    return encoding



_FIGURE_TEXT_ORDER = ("title", "subtitle", "caption", "footnote")
_FIGURE_TEXT_DEFAULTS = {
    "title": {"alignment": "center", "font_size": 20, "font_weight": "bold"},
    "subtitle": {"alignment": "center", "font_size": 14, "font_weight": "normal"},
    "caption": {"alignment": "left", "font_size": 11, "font_weight": "normal"},
    "footnote": {"alignment": "left", "font_size": 10, "font_weight": "normal"},
}


def _prepare_figure_text(entries: Any, width: float, default_family: str) -> list[dict[str, Any]]:
    if not isinstance(entries, list):
        raise ValueError("figure_text must be a list of role objects")
    prepared = []
    seen = set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            raise ValueError(f"figure_text[{index}] must be an object")
        allowed = {"role", "text", "alignment", "font_family", "font_size",
                   "font_weight", "font_style", "spacing_after"}
        unknown = set(entry) - allowed
        if unknown:
            raise ValueError(f"figure_text[{index}] has unknown fields: {', '.join(sorted(unknown))}")
        role = entry.get("role")
        if not isinstance(role, str) or role not in _FIGURE_TEXT_ORDER:
            raise ValueError(f"figure_text[{index}].role must be title, subtitle, caption, or footnote")
        if role in seen:
            raise ValueError(f"figure_text contains duplicate role {role!r}")
        seen.add(role)
        text = entry.get("text")
        if not isinstance(text, str):
            raise ValueError(f"figure_text[{index}].text must be a string")
        if not text:
            continue
        defaults = _FIGURE_TEXT_DEFAULTS[role]
        alignment = entry.get("alignment", defaults["alignment"])
        if alignment not in ("left", "center", "right"):
            raise ValueError(f"figure_text[{index}].alignment must be left, center, or right")
        family = entry.get("font_family", default_family)
        if not isinstance(family, str) or not family.strip():
            raise ValueError(f"figure_text[{index}].font_family must be a non-empty string")
        size = _number(entry.get("font_size", defaults["font_size"]),
                       f"figure_text[{index}].font_size", 6, 48)
        weight = entry.get("font_weight", defaults["font_weight"])
        if weight not in ("normal", "bold"):
            raise ValueError(f"figure_text[{index}].font_weight must be normal or bold")
        style = entry.get("font_style", "normal")
        if style not in ("normal", "italic"):
            raise ValueError(f"figure_text[{index}].font_style must be normal or italic")
        spacing = _number(entry.get("spacing_after", 6), f"figure_text[{index}].spacing_after", 0, 100)
        max_chars = max(1, int((width - 48) / (size * 0.6)))
        lines = []
        for paragraph in text.split("\n"):
            lines.extend(textwrap.wrap(paragraph, width=max_chars, break_long_words=True,
                                       break_on_hyphens=False) or [""])
        prepared.append({
            "role": role, "text": text, "lines": lines, "alignment": alignment,
            "font_family": family, "font_size": size, "font_weight": weight,
            "font_style": style, "line_height": size * 1.25,
            "spacing_after": spacing,
        })
    return sorted(prepared, key=lambda item: _FIGURE_TEXT_ORDER.index(item["role"]))



def _prepare_cluster_regions(
    entries: Any,
    labels: list[str],
    variable_order: dict[str, Any] | None,
    left: float,
    top: float,
    cell: float,
) -> list[dict[str, Any]]:
    if not isinstance(entries, list):
        raise ValueError("cluster_regions must be a list")
    display_labels = variable_order["labels"] if variable_order is not None else labels
    display_indices = variable_order["indices"] if variable_order is not None else list(range(len(labels)))
    if len(set(display_labels)) != len(display_labels):
        raise ValueError("cluster_regions require unique variable identities after ordering")
    if len(set(display_indices)) != len(display_indices):
        raise ValueError("cluster_regions require unique source indices after ordering")
    prepared = []
    seen_ids = set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            raise ValueError(f"cluster_regions[{index}] must be an object")
        allowed = {"id", "variables", "indices", "color", "line_width"}
        unknown = set(entry) - allowed
        if unknown:
            raise ValueError(f"cluster_regions[{index}] has unknown fields: {', '.join(sorted(unknown))}")
        region_id = entry.get("id")
        if not isinstance(region_id, str) or not region_id.strip():
            raise ValueError(f"cluster_regions[{index}].id must be a non-empty string")
        if region_id in seen_ids:
            raise ValueError(f"cluster_regions contains duplicate id {region_id!r}")
        seen_ids.add(region_id)
        variables = entry.get("variables")
        indices = entry.get("indices")
        if (variables is None) == (indices is None):
            raise ValueError(f"cluster region {region_id!r} must specify exactly one of variables or indices")
        if variables is not None:
            if (not isinstance(variables, list) or not variables
                or any(not isinstance(name, str) or not name for name in variables)
                or len(set(variables)) != len(variables)):
                raise ValueError(f"cluster_regions[{index}].variables must contain unique non-empty variable names")
            unknown_variables = [name for name in variables if name not in display_labels]
            if unknown_variables:
                raise ValueError(
                    f"cluster region {region_id!r} references unknown variables: {', '.join(unknown_variables)}"
                )
            positions = [position for position, name in enumerate(display_labels) if name in set(variables)]
        else:
            if (not isinstance(indices, list) or not indices
                or any(isinstance(value, bool) or not isinstance(value, int) or value < 0 for value in indices)
                or len(set(indices)) != len(indices)):
                raise ValueError(f"cluster_regions[{index}].indices must contain unique non-negative source indices")
            unknown_indices = [value for value in indices if value not in display_indices]
            if unknown_indices:
                raise ValueError(
                    f"cluster region {region_id!r} references unknown source indices: {unknown_indices}"
                )
            positions = [position for position, value in enumerate(display_indices) if value in set(indices)]
        start, end = min(positions), max(positions)
        if positions != list(range(start, end + 1)):
            raise ValueError(
                f"cluster region {region_id!r} variables must be contiguous after ordering"
            )
        resolved_variables = [display_labels[position] for position in positions]
        resolved_indices = [display_indices[position] for position in positions]
        color = entry.get("color", "#000000")
        if (not isinstance(color, str) or len(color) != 7 or color[0] != "#"
            or any(char not in "0123456789abcdefABCDEF" for char in color[1:])):
            raise ValueError(f"cluster_regions[{index}].color must be a six-digit hex color")
        line_width = _number(entry.get("line_width", 1.5),
                             f"cluster_regions[{index}].line_width", 0.1, 12)
        span = (end - start + 1) * cell
        prepared.append({
            "id": region_id,
            "variables": resolved_variables,
            "source_indices": resolved_indices,
            "row_start": start,
            "row_end": end,
            "column_start": start,
            "column_end": end,
            "x": left + start * cell,
            "y": top + start * cell,
            "width": span,
            "height": span,
            "color": color,
            "line_width": line_width,
        })
    return prepared


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

    matrix_mode = config.get("matrix_mode", "correlation")
    if matrix_mode not in ("correlation", "general_numeric"):
        raise ValueError("matrix_mode must be correlation or general_numeric")
    if matrix_mode == "general_numeric" and not isinstance(config.get("matrix_semantics"), dict):
        raise ValueError("general_numeric scenes require explicit matrix_semantics")
    for row in matrix:
        for value in row:
            if value is None:
                continue
            if isinstance(value, bool) or not isinstance(value, (int, float)) or not isfinite(value):
                raise ValueError("matrix values must be finite numbers or null")
            if matrix_mode == "correlation" and not -1 <= value <= 1:
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

    figure_text = _prepare_figure_text(config.get("figure_text", []), width, font_family)
    top_roles = [item for item in figure_text if item["role"] in ("title", "subtitle")]
    bottom_roles = [item for item in figure_text if item["role"] in ("caption", "footnote")]
    top_cursor = 12.0
    for item in top_roles:
        item["x"] = {"left": 24.0, "center": width / 2, "right": width - 24.0}[item["alignment"]]
        item["y"] = top_cursor + item["font_size"]
        top_cursor += len(item["lines"]) * item["line_height"] + item["spacing_after"]
    top_extra = top_cursor + 8.0 if top_roles else 0.0
    top += top_extra
    plot_bottom = top + len(labels) * cell
    bottom_cursor = plot_bottom + 24.0
    for item in bottom_roles:
        item["x"] = {"left": 24.0, "center": width / 2, "right": width - 24.0}[item["alignment"]]
        item["y"] = bottom_cursor + item["font_size"]
        bottom_cursor += len(item["lines"]) * item["line_height"] + item["spacing_after"]
    if bottom_roles:
        height = max(height + top_extra + bottom_cursor - (plot_bottom + 24.0),
                     bottom_cursor + 16.0)
    elif top_roles:
        height += top_extra

    matrix_values = matrix
    p_values = config.get("p_values")
    if p_values is not None:
        if not isinstance(p_values, list) or len(p_values) != len(labels) or any(
            not isinstance(row, list) or len(row) != len(labels) for row in p_values
        ):
            raise ValueError("p_values must be square and match labels")
        for row in p_values:
            for value in row:
                if value is not None and (
                    isinstance(value, bool) or not isinstance(value, (int, float))
                    or not isfinite(value) or not 0 <= value <= 1
                ):
                    raise ValueError("p_values must contain numbers between 0 and 1 or null")
        for row_index in range(len(labels)):
            for column_index in range(row_index + 1, len(labels)):
                p_left, p_right = p_values[row_index][column_index], p_values[column_index][row_index]
                if (p_left is None) != (p_right is None) or (
                    p_left is not None and abs(p_left - p_right) > 1e-8
                ):
                    raise ValueError("p_values must be symmetric, including matching missing entries")

    significance = config.get("significance")
    if significance is not None:
        if not isinstance(significance, dict) or set(significance) != {"mode", "thresholds", "symbol"}:
            raise ValueError("significance must contain mode, thresholds, and symbol")
        thresholds = significance["thresholds"]
        if (significance["mode"] not in ("pch", "p-value", "blank", "stars")
            or not isinstance(thresholds, list) or not thresholds
            or any(isinstance(value, bool) or not isinstance(value, (int, float))
                   or not isfinite(value) or not 0 < value <= 1 for value in thresholds)
            or any(left >= right for left, right in zip(thresholds, thresholds[1:]))
            or not isinstance(significance["symbol"], str) or not significance["symbol"].strip()
            or len(significance["symbol"]) > 8):
            raise ValueError("significance mode, thresholds, or symbol are invalid")
        if p_values is None:
            raise ValueError("significance settings require p_values")

    confidence_intervals = config.get("confidence_intervals")
    if confidence_intervals is not None:
        if matrix_mode != "correlation":
            raise ValueError("confidence_intervals currently require a correlation matrix")
        if not isinstance(confidence_intervals, dict) or set(confidence_intervals) != {"lower", "upper"}:
            raise ValueError("confidence_intervals must contain lower and upper matrices")
        for key in ("lower", "upper"):
            matrix = confidence_intervals[key]
            if not isinstance(matrix, list) or len(matrix) != len(labels) or any(
                not isinstance(row, list) or len(row) != len(labels) for row in matrix
            ):
                raise ValueError(f"confidence_intervals.{key} must be square and match labels")
            for row in matrix:
                for value in row:
                    if value is not None and (
                        isinstance(value, bool) or not isinstance(value, (int, float))
                        or not isfinite(value) or not -1 <= value <= 1
                    ):
                        raise ValueError("confidence interval bounds must be between -1 and 1 or null")
            for row_index in range(len(labels)):
                for column_index in range(row_index + 1, len(labels)):
                    left, right = matrix[row_index][column_index], matrix[column_index][row_index]
                    if (left is None) != (right is None) or (
                        left is not None and abs(left - right) > 1e-8
                    ):
                        raise ValueError(f"confidence_intervals.{key} must be symmetric")
        lower, upper = confidence_intervals["lower"], confidence_intervals["upper"]
        for row_index in range(len(labels)):
            for column_index in range(len(labels)):
                low, high = lower[row_index][column_index], upper[row_index][column_index]
                if (low is None) != (high is None):
                    raise ValueError("confidence interval lower and upper must both be null or numeric")
                if low is not None:
                    if low > high:
                        raise ValueError("confidence interval lower bound must not exceed upper bound")
                    estimate = matrix_values[row_index][column_index]
                    if estimate is None:
                        raise ValueError("confidence intervals must be null when the correlation is missing")
                    if low > estimate or estimate > high:
                        raise ValueError("confidence interval must contain the correlation estimate")
    variable_order = config.get("variable_order")
    if variable_order is not None:
        if not isinstance(variable_order, dict) or set(variable_order) != {"strategy", "indices", "labels"}:
            raise ValueError("variable_order must contain strategy, indices, and labels")
        indices = variable_order["indices"]
        original_labels = variable_order["labels"]
        if (not isinstance(variable_order["strategy"], str)
            or not isinstance(indices, list) or len(indices) != len(labels)
            or any(isinstance(i, bool) or not isinstance(i, int) or i < 0 for i in indices)
            or len(set(indices)) != len(indices)
            or not isinstance(original_labels, list) or len(original_labels) != len(labels)
            or any(not isinstance(label, str) for label in original_labels)):
            raise ValueError("variable_order must preserve one unique original index and label per variable")


    method = config.get("method", "circle")
    supported_methods = ("circle", "square", "ellipse", "number", "shade", "color", "pie")
    if method not in supported_methods:
        raise ValueError(f"method must be one of {', '.join(supported_methods)}")
    display_type = config.get("display_type", "full")
    if display_type not in ("full", "upper", "lower", "diagonal"):
        raise ValueError("display_type must be full, upper, lower, or diagonal")
    show_diagonal = config.get("show_diagonal", True)
    if not isinstance(show_diagonal, bool):
        raise ValueError("show_diagonal must be a boolean")
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
            if p_values is not None:
                item["p_value"] = p_values[row_index][column_index]
            cells.append(item)

    cluster_regions = _prepare_cluster_regions(
        config.get("cluster_regions", []), labels, variable_order, left, top, cell
    )

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
        "schema_version": 1,
        "kind": "correlation_plot" if matrix_mode == "correlation" else "general_matrix_plot",
        "width": width, "height": height, "labels": label_items, "cells": cells,
        "annotation": annotation,
        "glyph_method": method, "text_annotations": text_annotations,
        "title": title, "description": description,
        **({"cluster_regions": cluster_regions} if cluster_regions else {}),
        **({"figure_text": figure_text} if figure_text else {}),
        "style": {
            "positive_color": "#2166ac", "negative_color": "#b2182b",
            "zero_color": "#bdbdbd", "missing_fill": "#f2f2f2",
            "grid_color": "#e0e0e0",
        },
    }
    if config.get("matrix_semantics") is not None:
        semantics = config["matrix_semantics"]
        if not isinstance(semantics, dict) or semantics.get("mode") != matrix_mode:
            raise ValueError("matrix_semantics.mode must match matrix_mode")
        limits = semantics.get("color_limits")
        normalization = semantics.get("normalization")
        if (not isinstance(limits, list) or len(limits) != 2
            or any(isinstance(v, bool) or not isinstance(v, (int, float)) or not isfinite(v) for v in limits)
            or limits[0] >= limits[1]
            or semantics.get("color_mapping") not in ("sequential", "diverging")
            or not isinstance(normalization, dict)
            or isinstance(normalization.get("intercept"), bool)
            or not isinstance(normalization.get("intercept"), (int, float))
            or isinstance(normalization.get("zoom"), bool)
            or not isinstance(normalization.get("zoom"), (int, float))
            or not isfinite(normalization["intercept"]) or not isfinite(normalization["zoom"])
            or normalization["zoom"] <= 0):
            raise ValueError("matrix_semantics must contain valid limits, mapping, and normalization")
        scene["matrix_semantics"] = semantics
        transform = normalization
        for cell in cells:
            value = cell["value"]
            cell["normalized_value"] = None if value is None else (
                (value + transform["intercept"]) * transform["zoom"]
            )
    for cell in cells:
        if cell["value"] is not None:
            cell["encoding"] = _cell_encoding(
                cell, cell.get("method", method), scene["style"]
            )
    if significance is not None:
        scene["significance"] = {
            "mode": significance["mode"],
            "thresholds": list(significance["thresholds"]),
            "symbol": significance["symbol"],
        }
        cutoff = significance["thresholds"][-1]
        for cell_item in cells:
            p_value = cell_item.get("p_value")
            if p_value is None:
                status, annotation, blank = "unknown", None, False
            else:
                is_significant = p_value <= cutoff
                status, annotation, blank = (
                    ("significant", None, False) if is_significant
                    else ("not_significant", None, False)
                )
                mode = significance["mode"]
                if mode == "stars" and is_significant:
                    star_count = next(
                        (len(significance["thresholds"]) - index
                         for index, threshold in enumerate(significance["thresholds"])
                         if p_value <= threshold),
                        0,
                    )
                    annotation = "*" * star_count if star_count else None
                elif mode == "pch" and not is_significant:
                    annotation = significance["symbol"]
                elif mode == "p-value" and not is_significant:
                    annotation = f"p={p_value:.3g}"
                elif mode == "blank" and not is_significant:
                    blank = True
            cell_item["significance"] = {
                "p_value": p_value, "status": status,
                "annotation": annotation, "blank": blank,
            }

    if confidence_intervals is not None:
        for cell_item in cells:
            row_index, column_index = cell_item["row"], cell_item["column"]
            lower = confidence_intervals["lower"][row_index][column_index]
            upper = confidence_intervals["upper"][row_index][column_index]
            if lower is None:
                continue
            cell_item["confidence_interval"] = {
                "lower": lower, "upper": upper,
                "x1": cell_item["x"] + (lower + 1.0) * 0.5 * cell_item["size"],
                "x2": cell_item["x"] + (upper + 1.0) * 0.5 * cell_item["size"],
                "y": cell_item["center_y"],
                "cap_height": cell_item["size"] * 0.14,
                "stroke_width": max(1.0, cell_item["size"] * 0.035),
                "color": "#333333",
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
    if variable_order is not None:
        scene["variable_order"] = variable_order
    return scene
