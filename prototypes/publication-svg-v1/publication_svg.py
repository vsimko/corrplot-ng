"""Deterministic publication SVG export for scene-v1 data.

The exporter deliberately consumes the renderer-neutral scene contract. Physical
page dimensions and typography are output settings, not scene semantics.
"""
from __future__ import annotations

import html
import math
from typing import Any


def _positive_finite(value: Any, name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{name} must be a positive finite number")
    value = float(value)
    if not math.isfinite(value) or value <= 0:
        raise ValueError(f"{name} must be a positive finite number")
    return value


def _number(value: float) -> str:
    return format(float(value), ".6g")


def render_publication_svg(
    scene: dict[str, Any],
    *,
    width_mm: float,
    height_mm: float,
    font_family: str = "sans-serif",
    font_size_pt: float = 9.0,
) -> str:
    """Render scene-v1 to stable SVG with explicit physical size and typography.

    The viewBox remains in scene logical units. SVG width/height are physical
    millimetres, allowing the same figure to retain its page size in print and
    LaTeX. Label and annotation margins expand for the requested font size.
    """
    width_mm = _positive_finite(width_mm, "width_mm")
    height_mm = _positive_finite(height_mm, "height_mm")
    font_size_pt = _positive_finite(font_size_pt, "font_size_pt")
    if not isinstance(font_family, str) or not font_family.strip():
        raise ValueError("font_family must be a non-empty string")
    if not isinstance(scene, dict) or scene.get("schema_version") != 1:
        raise ValueError("scene must use scene schema version 1")
    if scene.get("kind") != "correlation_plot":
        raise ValueError("scene kind must be correlation_plot")
    if not scene.get("cells") or not scene.get("labels"):
        raise ValueError("scene must contain labels and cells")

    # CSS/SVG font sizes use points; estimate text extents conservatively so a
    # larger requested font cannot push labels or annotations outside the viewBox.
    font_px = font_size_pt * (96.0 / 72.0)
    char_width = font_px * 0.6
    min_row_left = min(
        item["row_x"] - len(item["text"]) * char_width
        for item in scene["labels"]
    )
    min_column_top = min(
        item["column_y"] - len(item["text"]) * char_width
        for item in scene["labels"]
    )
    shift_x = max(0.0, 12.0 - min_row_left)
    shift_y = max(0.0, 12.0 - min_column_top)

    annotation = scene.get("annotation", "")
    annotation_right = 0.0
    if annotation:
        annotation_x = scene["cells"][0]["x"] + shift_x
        annotation_right = annotation_x + len(annotation) * char_width + 12.0

    legend = scene.get("legend", [])
    legend_width = max((28.0 + len(entry["label"]) * char_width for entry in legend), default=0.0)
    legend_x = float(scene["cells"][0]["x"]) + shift_x
    view_width = max(float(scene["width"]) + shift_x, annotation_right,
                     legend_x + legend_width + 12.0)
    annotation_extra = max(0.0, font_px + 8.0 - 22.0)
    legend_top = float(scene["height"]) + shift_y + annotation_extra + (12.0 if annotation else 0.0)
    legend_extra = (len(legend) * 20.0 + 12.0) if legend else 0.0
    view_height = float(scene["height"]) + shift_y + annotation_extra + legend_extra

    family = html.escape(font_family.strip(), quote=True)
    style = scene["style"]
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{_number(width_mm)}mm" height="{_number(height_mm)}mm" viewBox="0 0 {_number(view_width)} {_number(view_height)}" preserveAspectRatio="xMidYMid meet">',
        '<title>Correlation matrix</title>',
        '<desc>Deterministic publication figure generated from a version 1 correlation plot scene.</desc>',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
    ]

    for label in scene["labels"]:
        row_x = label["row_x"] + shift_x
        row_y = label["row_y"] + shift_y
        column_x = label["column_x"] + shift_x
        column_y = label["column_y"] + shift_y
        text = html.escape(label["text"])
        parts.append(
            f'<text x="{_number(row_x)}" y="{_number(row_y)}" text-anchor="end" dominant-baseline="middle" font-family="{family}" font-size="{_number(font_size_pt)}pt">{text}</text>'
        )
        parts.append(
            f'<text x="{_number(column_x)}" y="{_number(column_y)}" transform="rotate(-90 {_number(column_x)} {_number(column_y)})" text-anchor="start" font-family="{family}" font-size="{_number(font_size_pt)}pt">{text}</text>'
        )

    for cell in scene["cells"]:
        x = cell["x"] + shift_x
        y = cell["y"] + shift_y
        size = cell["size"]
        cx = cell["center_x"] + shift_x
        cy = cell["center_y"] + shift_y
        value = cell["value"]
        if value is None:
            parts.append(
                f'<rect x="{_number(x)}" y="{_number(y)}" width="{_number(size)}" height="{_number(size)}" fill="{cell.get("color", style["missing_fill"])}" stroke="#ffffff"/>'
            )
            parts.append(
                f'<text x="{_number(cx)}" y="{_number(cy)}" text-anchor="middle" dominant-baseline="middle" font-family="{family}" font-size="{_number(font_size_pt)}pt" fill="#666666">NA</text>'
            )
            continue
        color = cell.get("color", style["positive_color"] if value > 0 else style["negative_color"] if value < 0 else style["zero_color"])
        radius = abs(value) * (size * 0.43)
        if value == 0 and "color" in cell:
            radius = 2.5
        parts.append(
            f'<rect x="{_number(x)}" y="{_number(y)}" width="{_number(size)}" height="{_number(size)}" fill="#ffffff" stroke="{style["grid_color"]}"/>'
        )
        if radius > 0:
            parts.append(
                f'<circle cx="{_number(cx)}" cy="{_number(cy)}" r="{_number(radius)}" fill="{color}"/>'
            )

    if annotation:
        annotation_x = scene["cells"][0]["x"] + shift_x
        annotation_y = float(scene["height"]) - 22.0 + shift_y
        parts.append(
            f'<text x="{_number(annotation_x)}" y="{_number(annotation_y)}" font-family="{family}" font-size="{_number(font_size_pt)}pt" fill="#333333">{html.escape(annotation)}</text>'
        )
    for index, entry in enumerate(legend):
        legend_y = legend_top + index * 20.0 + 10.0
        parts.append(
            f'<rect x="{_number(legend_x)}" y="{_number(legend_y - 10.0)}" width="12" height="12" fill="{entry["color"]}" stroke="#666666" stroke-width="0.5"/>'
        )
        parts.append(
            f'<text x="{_number(legend_x + 19.0)}" y="{_number(legend_y)}" dominant-baseline="middle" font-family="{family}" font-size="{_number(font_size_pt)}pt">{html.escape(entry["label"])}</text>'
        )
    parts.append("</svg>")
    return "\n".join(parts) + "\n"
