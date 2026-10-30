"""SVG backend for the renderer-neutral correlation plot scene."""
from __future__ import annotations

import html
import json
import sys
from pathlib import Path
from typing import Any

from scene import build_scene


def render_scene_svg(scene: dict[str, Any]) -> str:
    """Serialize a prepared plot scene to deterministic SVG."""
    width, height = scene["width"], scene["height"]
    style = scene["style"]
    typography = scene.get("typography", {})
    family = html.escape(typography.get("label_font_family", "sans-serif"), quote=True)
    font_size = typography.get("label_font_size", 13)
    weight = typography.get("label_font_weight", "normal")
    orientation = typography.get("column_label_orientation", "vertical")
    collision = typography.get("label_collision", "preserve")
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width:g}" height="{height:g}" viewBox="0 0 {width:g} {height:g}">',
        '<title>Correlation matrix</title>',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
    ]

    for label in scene["labels"]:
        text = html.escape(label["text"])
        if collision == "omit":
            row_text = ""
        elif collision == "truncate":
            max_chars = max(1, int(label["row_x"] / (font_size * 0.6)))
            row_text = text if len(text) <= max_chars else text[:max(1, max_chars - 1)] + "…"
        else:
            row_text = text
        if row_text:
            parts.append(
                f'<text x="{label["row_x"]:g}" y="{label["row_y"]:.1f}" text-anchor="end" dominant-baseline="middle" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{row_text}</text>'
            )
        if orientation == "horizontal":
            column_text = label["text"]
            max_chars = max(1, int(scene["cells"][0]["size"] / (font_size * 0.6)))
            if collision == "truncate":
                column_text = column_text if len(column_text) <= max_chars else column_text[:max(1, max_chars - 1)] + "…"
            elif collision == "omit":
                column_text = ""
            if column_text and collision == "wrap":
                lines = [column_text[index:index + max_chars] for index in range(0, len(column_text), max_chars)]
                line_height = font_size * 1.1
                start_y = scene["cells"][0]["y"] - (len(lines) - 1) * line_height - font_size
                spans = "".join(
                    f'<tspan x="{label["column_x"]:.1f}" dy="{0 if line_index == 0 else line_height:g}">{html.escape(line)}</tspan>'
                    for line_index, line in enumerate(lines)
                )
                parts.append(
                    f'<text x="{label["column_x"]:.1f}" y="{start_y:g}" text-anchor="middle" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{spans}</text>'
                )
            elif column_text:
                parts.append(
                    f'<text x="{label["column_x"]:.1f}" y="{label["column_y"]:g}" text-anchor="middle" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{html.escape(column_text)}</text>'
                )
        elif collision != "omit":
            column_text = label["text"]
            if collision == "truncate":
                max_chars = max(1, int(scene["cells"][0]["size"] / (font_size * 0.6)))
                column_text = column_text if len(column_text) <= max_chars else column_text[:max(1, max_chars - 1)] + "…"
            parts.append(
                f'<text x="{label["column_x"]:.1f}" y="{label["column_y"]:g}" transform="rotate(-90 {label["column_x"]:.1f} {label["column_y"]:g})" text-anchor="start" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{html.escape(column_text)}</text>'
            )

    for cell in scene["cells"]:
        x, y, size = cell["x"], cell["y"], cell["size"]
        cx, cy, value = cell["center_x"], cell["center_y"], cell["value"]
        if value is None:
            parts.append(f'<rect x="{x:g}" y="{y:g}" width="{size:g}" height="{size:g}" fill="{style["missing_fill"]}" stroke="#ffffff"/>')
            parts.append(f'<text x="{cx:.1f}" y="{cy:.1f}" text-anchor="middle" dominant-baseline="middle" font-family="{family}" font-size="{min(font_size, 12):g}" fill="#666666">NA</text>')
            continue
        color = style["positive_color"] if value > 0 else style["negative_color"] if value < 0 else style["zero_color"]
        radius = abs(value) * (size * 0.43)
        parts.append(f'<rect x="{x:g}" y="{y:g}" width="{size:g}" height="{size:g}" fill="#ffffff" stroke="{style["grid_color"]}"/>')
        if radius > 0:
            parts.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{radius:.2f}" fill="{color}"/>')

    if scene["annotation"]:
        parts.append(f'<text x="{scene["cells"][0]["x"]:g}" y="{height - 22:g}" font-family="{family}" font-size="{min(font_size, 12):g}" fill="#333333">{html.escape(scene["annotation"])}</text>')
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def render_svg(config: dict[str, Any]) -> str:
    """Compatibility convenience wrapper: config -> scene -> SVG."""
    return render_scene_svg(build_scene(config))


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("example.json")
    print(render_svg(json.loads(source.read_text(encoding="utf-8"))), end="")


if __name__ == "__main__":
    main()
