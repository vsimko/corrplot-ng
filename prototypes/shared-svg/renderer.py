"""Small architecture spike: render a language-neutral correlation plot as SVG.

This is deliberately not production code and does not select the final core language.
"""
from __future__ import annotations

import html
import json
import sys
from pathlib import Path
from typing import Any


def render_svg(config: dict[str, Any]) -> str:
    labels = config["labels"]
    matrix = config["matrix"]
    annotation = config.get("annotation", "")
    n = len(labels)
    if n == 0 or len(matrix) != n or any(len(row) != n for row in matrix):
        raise ValueError("matrix must be a non-empty square matrix matching labels")
    if any(value is not None and not -1 <= value <= 1 for row in matrix for value in row):
        raise ValueError("correlation values must be null or between -1 and 1")

    cell = 84
    left, top = 104, 76
    right_margin, bottom_margin = 24, 64
    width = left + n * cell + right_margin
    height = top + n * cell + bottom_margin
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<title>Correlation matrix</title>',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
    ]

    for i, label in enumerate(labels):
        y = top + i * cell + cell / 2
        x = left + i * cell + cell / 2
        parts.append(
            f'<text x="{left - 12}" y="{y:.1f}" text-anchor="end" dominant-baseline="middle" font-family="sans-serif" font-size="13">{html.escape(str(label))}</text>'
        )
        parts.append(
            f'<text x="{x:.1f}" y="{top - 16}" text-anchor="middle" font-family="sans-serif" font-size="13">{html.escape(str(label))}</text>'
        )

    for i, row in enumerate(matrix):
        for j, value in enumerate(row):
            cx = left + j * cell + cell / 2
            cy = top + i * cell + cell / 2
            if value is None:
                parts.append(f'<rect x="{left + j * cell}" y="{top + i * cell}" width="{cell}" height="{cell}" fill="#f2f2f2" stroke="#ffffff"/>')
                parts.append(f'<text x="{cx:.1f}" y="{cy:.1f}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif" font-size="12" fill="#666666">NA</text>')
                continue
            color = "#2166ac" if value > 0 else "#b2182b" if value < 0 else "#bdbdbd"
            radius = abs(value) * (cell * 0.43)
            parts.append(f'<rect x="{left + j * cell}" y="{top + i * cell}" width="{cell}" height="{cell}" fill="#ffffff" stroke="#e0e0e0"/>')
            if radius > 0:
                parts.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{radius:.2f}" fill="{color}"/>')

    if annotation:
        parts.append(
            f'<text x="{left}" y="{height - 22}" font-family="sans-serif" font-size="12" fill="#333333">{html.escape(annotation)}</text>'
        )
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("example.json")
    print(render_svg(json.loads(source.read_text(encoding="utf-8"))), end="")


if __name__ == "__main__":
    main()
