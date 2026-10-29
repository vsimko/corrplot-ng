"""SVG backend for the renderer-neutral correlation plot scene."""
from __future__ import annotations

import html
import json
import sys
from pathlib import Path
from typing import Any

from scene import build_scene


def render_scene_svg(scene: dict[str, Any]) -> str:
    """Serialize a validated plot scene to deterministic SVG."""
    width = scene["width"]
    height = scene["height"]
    style = scene["style"]
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<title>Correlation matrix</title>',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
    ]

    for label in scene["labels"]:
        parts.append(
            f'<text x="{label["row_x"]}" y="{label["row_y"]:.1f}" text-anchor="end" dominant-baseline="middle" font-family="sans-serif" font-size="13">{html.escape(label["text"])}</text>'
        )
        parts.append(
            f'<text x="{label["column_x"]:.1f}" y="{label["column_y"]}" text-anchor="middle" font-family="sans-serif" font-size="13">{html.escape(label["text"])}</text>'
        )

    for cell in scene["cells"]:
        x, y, size = cell["x"], cell["y"], cell["size"]
        cx, cy, value = cell["center_x"], cell["center_y"], cell["value"]
        if value is None:
            parts.append(f'<rect x="{x}" y="{y}" width="{size}" height="{size}" fill="{style["missing_fill"]}" stroke="#ffffff"/>')
            parts.append(f'<text x="{cx:.1f}" y="{cy:.1f}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif" font-size="12" fill="#666666">NA</text>')
            continue
        color = style["positive_color"] if value > 0 else style["negative_color"] if value < 0 else style["zero_color"]
        radius = abs(value) * (size * 0.43)
        parts.append(f'<rect x="{x}" y="{y}" width="{size}" height="{size}" fill="#ffffff" stroke="{style["grid_color"]}"/>')
        if radius > 0:
            parts.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{radius:.2f}" fill="{color}"/>')

    if scene["annotation"]:
        parts.append(
            f'<text x="104" y="{height - 22}" font-family="sans-serif" font-size="12" fill="#333333">{html.escape(scene["annotation"])}</text>'
        )
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
