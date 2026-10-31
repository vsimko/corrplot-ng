from __future__ import annotations

import sys
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))

from validate_svg import SvgValidationError, validate_svg_text
from renderer import render_svg


SVG_NS = "http://www.w3.org/2000/svg"


def minimal_svg(body: str = "", attributes: str = "") -> str:
    return (
        f'<svg xmlns="{SVG_NS}" width="100" height="80" '
        f'viewBox="0 0 100 80" {attributes}>{body}</svg>'
    )


class SvgValidationTests(unittest.TestCase):
    def test_accepts_python_renderer_output_with_escaped_metadata(self) -> None:
        svg = render_svg({
            "labels": ["alpha"],
            "matrix": [[1.0]],
            "title": "R&D <metrics> – Δ",
            "description": "A & B < C",
        })
        result = validate_svg_text(svg)
        self.assertGreater(result["elements"], 3)
        self.assertEqual(result["width"], 212)
        root = ET.fromstring(svg)
        self.assertEqual(root.find(f"{{{SVG_NS}}}title").text, "R&D <metrics> – Δ")
        self.assertEqual(root.find(f"{{{SVG_NS}}}desc").text, "A & B < C")

    def test_accepts_internal_references_and_forward_accessibility_ids(self) -> None:
        svg = minimal_svg(
            '<rect id="plot" clip-path="url(#clip)"/>'
            '<defs><clipPath id="clip"><circle cx="10" cy="10" r="5"/></clipPath></defs>',
            'role="img" aria-labelledby="figure-title" aria-describedby="figure-description"',
        ).replace(
            '<rect id="plot"',
            '<title id="figure-title">Figure</title><desc id="figure-description">Details</desc><rect id="plot"',
        )
        result = validate_svg_text(svg)
        self.assertEqual(result["ids"], 4)

    def test_rejects_malformed_xml(self) -> None:
        with self.assertRaisesRegex(SvgValidationError, "malformed XML"):
            validate_svg_text("<svg><rect></svg>")

    def test_rejects_non_svg_root_namespace(self) -> None:
        with self.assertRaisesRegex(SvgValidationError, "SVG namespace"):
            validate_svg_text('<svg width="100" height="80" viewBox="0 0 100 80"/>')

    def test_rejects_invalid_dimensions_and_viewbox(self) -> None:
        for svg in (
            minimal_svg().replace('width="100"', 'width="0"'),
            minimal_svg().replace('height="80"', 'height="auto"'),
            minimal_svg().replace('viewBox="0 0 100 80"', 'viewBox="0 0 100"'),
            minimal_svg().replace('viewBox="0 0 100 80"', 'viewBox="0 0 -1 80"'),
        ):
            with self.subTest(svg=svg), self.assertRaises(SvgValidationError):
                validate_svg_text(svg)

    def test_rejects_duplicate_ids_and_broken_references(self) -> None:
        duplicate = minimal_svg('<title id="same">A</title><desc id="same">B</desc>')
        with self.assertRaisesRegex(SvgValidationError, "duplicate SVG id"):
            validate_svg_text(duplicate)

        broken_aria = minimal_svg('<title id="title">A</title>', 'aria-labelledby="missing"')
        with self.assertRaisesRegex(SvgValidationError, "broken @aria-labelledby"):
            validate_svg_text(broken_aria)

        broken_url = minimal_svg('<rect fill="url(#missing)"/>')
        with self.assertRaisesRegex(SvgValidationError, "broken URL reference"):
            validate_svg_text(broken_url)

    def test_rejects_external_assets_and_active_content(self) -> None:
        invalid = (
            minimal_svg('<image href="https://example.invalid/picture.png"/>'),
            minimal_svg('<script>alert(1)</script>'),
            minimal_svg('<foreignObject><div/></foreignObject>'),
            minimal_svg('<rect style="fill:url(https://example.invalid/paint.svg#x)"/>'),
            minimal_svg('<rect onclick="alert(1)"/>'),
            minimal_svg('<rect fill="url(#paint)"/>').replace(
                '<rect fill="url(#paint)"/>',
                '<rect fill="url(#paint)"/>',
            ),
        )
        for svg in invalid:
            with self.subTest(svg=svg), self.assertRaises(SvgValidationError):
                validate_svg_text(svg)

    def test_rejects_unknown_elements_instead_of_silently_accepting_them(self) -> None:
        with self.assertRaisesRegex(SvgValidationError, "unsupported SVG element"):
            validate_svg_text(minimal_svg("<foreignThing/>"))

    def test_reports_source_path_in_errors(self) -> None:
        with self.assertRaisesRegex(SvgValidationError, "figure.svg"):
            validate_svg_text("<svg>", "figure.svg")


if __name__ == "__main__":
    unittest.main()
