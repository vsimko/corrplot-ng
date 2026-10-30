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
    title = html.escape(scene["title"], quote=False)
    description = scene["description"]
    described_by = ' aria-describedby="figure-description"' if description else ""
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width:g}" height="{height:g}" viewBox="0 0 {width:g} {height:g}" role="img" aria-labelledby="figure-title"{described_by}>',
        f'<title id="figure-title">{title}</title>',
        f'<desc id="figure-description">{html.escape(description, quote=False)}</desc>' if description else "",
        '<rect width="100%" height="100%" fill="#ffffff"/>',
    ]

    for label in scene["labels"]:
        if label.get("plotmath") is not None:
            raise ValueError(
                "the SVG renderer does not interpret R plotmath labels; "
                "use the R graphics renderer or convert the label explicitly"
            )
        text = html.escape(label["text"])
        if collision == "omit":
            row_text = ""
        elif collision == "truncate":
            max_chars = max(1, int(label["row_x"] / (font_size * 0.6)))
            row_text = text if len(text) <= max_chars else text[:max(1, max_chars - 1)] + "…"
        else:
            row_text = text
        if row_text and label.get("row_visible", True):
            parts.append(
                f'<text x="{label["row_x"]:g}" y="{label["row_y"]:.1f}" text-anchor="end" dominant-baseline="middle" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{row_text}</text>'
            )
        if not label.get("column_visible", True):
            continue
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
            rotation = -label.get("column_rotation", 90)
            parts.append(
                f'<text x="{label["column_x"]:.1f}" y="{label["column_y"]:g}" transform="rotate({rotation:g} {label["column_x"]:.1f} {label["column_y"]:g})" text-anchor="start" font-family="{family}" font-size="{font_size:g}" font-weight="{weight}">{html.escape(column_text)}</text>'
            )

    for cell in scene["cells"]:
        if not cell.get("visible", True):
            continue
        x, y, size = cell["x"], cell["y"], cell["size"]
        cx, cy, value = cell["center_x"], cell["center_y"], cell["value"]
        if value is None:
            missing_color = cell.get("color") or style["missing_fill"]
            parts.append(f'<rect x="{x:g}" y="{y:g}" width="{size:g}" height="{size:g}" fill="{missing_color}" stroke="#ffffff"/>')
            parts.append(f'<text x="{cx:.1f}" y="{cy:.1f}" text-anchor="middle" dominant-baseline="middle" font-family="{family}" font-size="{min(font_size, 12):g}" fill="#666666">NA</text>')
            continue
        color = cell.get("color") or (style["positive_color"] if value > 0 else style["negative_color"] if value < 0 else style["zero_color"])
        visual_value = cell.get("normalized_value", value)
        radius = abs(visual_value) * (size * 0.43)
        parts.append(f'<rect x="{x:g}" y="{y:g}" width="{size:g}" height="{size:g}" fill="#ffffff" stroke="{style["grid_color"]}"/>')
        if cell.get("method", scene.get("glyph_method", "circle")) != "number" and radius > 0:
            parts.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{radius:.2f}" fill="{color}"/>')

    cells_by_position = {
        (item["row"], item["column"]): item for item in scene["cells"] if item.get("visible", True)
    }
    for label in scene.get("text_annotations", []):
        cell = cells_by_position.get((label["row"], label["column"]))
        if cell is None:
            continue
        parts.append(
            f'<text x="{cell["center_x"]:.1f}" y="{cell["center_y"]:.1f}" '
            f'text-anchor="middle" dominant-baseline="middle" '
            f'font-family="{family}" font-size="{label["font_size"]:g}" '
            f'font-weight="{label["font_weight"]}" font-style="{label["font_style"]}" '
            f'fill="{html.escape(label["color"], quote=True)}">{html.escape(label["text"])}</text>'
        )

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
