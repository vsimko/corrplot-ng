import json
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path
import sys

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent / "shared-svg"))

from publication_svg import render_publication_svg
from scene import build_scene


SVG_NS = "{http://www.w3.org/2000/svg}"


class PublicationSvgTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        config = json.loads((HERE.parent / "shared-svg" / "example.json").read_text(encoding="utf-8"))
        cls.scene = build_scene(config)

    def render(self, **overrides):
        options = {
            "width_mm": 178,
            "height_mm": 132,
            "font_family": "Liberation Serif",
            "font_size_pt": 8.5,
        }
        options.update(overrides)
        return render_publication_svg(self.scene, **options)

    def test_physical_dimensions_and_logical_viewbox_are_explicit(self):
        root = ET.fromstring(self.render())
        self.assertEqual(root.attrib["width"], "178mm")
        self.assertEqual(root.attrib["height"], "132mm")
        self.assertRegex(root.attrib["viewBox"], r"^0 0 [0-9.]+ [0-9.]+$")
        self.assertEqual(root.attrib["preserveAspectRatio"], "xMidYMid meet")

    def test_typography_is_applied_and_escaped(self):
        svg = self.render(font_family='A&B "Serif"', font_size_pt=10)
        self.assertIn('font-family="A&amp;B &quot;Serif&quot;"', svg)
        self.assertIn('font-size="10pt"', svg)

    def test_long_labels_and_large_fonts_expand_viewbox(self):
        scene = build_scene({
            "labels": ["customer satisfaction score", "monthly recurring revenue"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
        })
        svg = render_publication_svg(
            scene, width_mm=180, height_mm=130,
            font_family="serif", font_size_pt=18,
        )
        root = ET.fromstring(svg)
        _, _, width, height = map(float, root.attrib["viewBox"].split())
        self.assertGreater(width, scene["width"])
        self.assertGreater(height, scene["height"])
        # The added left/top viewBox space is reflected by positive label coords.
        labels = root.findall(f".//{SVG_NS}text")
        self.assertTrue(labels)
        self.assertTrue(all(float(node.attrib["x"]) >= 0 for node in labels))

    def test_output_is_valid_xml_and_deterministic(self):
        first = self.render()
        second = self.render()
        ET.fromstring(first)
        self.assertEqual(first, second)
        self.assertTrue(first.endswith("</svg>\n"))

    def test_annotation_and_missing_values_are_preserved(self):
        svg = self.render()
        self.assertIn("Prototype annotation: synthetic example only", svg)
        self.assertEqual(svg.count(">NA</text>"), 2)

    def test_invalid_dimensions_and_typography_are_rejected(self):
        for field, value in [
            ("width_mm", 0), ("height_mm", float("inf")),
            ("font_size_pt", -1), ("font_family", "  "),
        ]:
            with self.subTest(field=field):
                with self.assertRaises(ValueError):
                    self.render(**{field: value})

    def test_rejects_unsupported_scene_versions(self):
        scene = dict(self.scene, schema_version=2)
        with self.assertRaisesRegex(ValueError, "version 1"):
            render_publication_svg(
                scene, width_mm=178, height_mm=132,
            )


if __name__ == "__main__":
    unittest.main()
