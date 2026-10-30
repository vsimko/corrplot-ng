import json
import unittest
from pathlib import Path

from renderer import render_scene_svg, render_svg
from scene import build_scene


HERE = Path(__file__).parent


class PlotSceneTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.config = json.loads((HERE / "example.json").read_text(encoding="utf-8"))
        cls.golden_scene = json.loads(
            (HERE / "scene-v1.example.json").read_text(encoding="utf-8")
        )

    def test_scene_is_renderer_neutral_json_data(self):
        scene = build_scene(self.config)
        self.assertEqual(scene["kind"], "correlation_plot")
        self.assertEqual(scene["schema_version"], 1)
        self.assertEqual(len(scene["cells"]), 9)
        self.assertEqual(scene["cells"][1]["value"], 0.65)
        self.assertIsNone(scene["cells"][5]["value"])
        json.dumps(scene)

    def test_generated_scene_matches_versioned_golden_fixture(self):
        # This fixture is the cross-language contract example consumed by R.
        self.assertEqual(build_scene(self.config), self.golden_scene)

    def test_cells_are_in_row_major_order(self):
        cells = build_scene(self.config)["cells"]
        self.assertEqual(
            [(cell["row"], cell["column"]) for cell in cells],
            [(row, column) for row in range(3) for column in range(3)],
        )

    def test_scene_and_svg_output_are_deterministic(self):
        scene = build_scene(self.config)
        self.assertEqual(scene, build_scene(self.config))
        self.assertEqual(render_scene_svg(scene), render_scene_svg(build_scene(self.config)))
        self.assertEqual(render_svg(self.config), render_scene_svg(scene))

    def test_svg_backend_emits_annotation_and_expected_symbols(self):
        svg = render_svg(self.config)
        self.assertTrue(svg.startswith('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertIn("Prototype annotation: synthetic example only", svg)
        self.assertEqual(svg.count("<circle "), 7)

    def test_missing_values_are_explicit(self):
        svg = render_svg(self.config)
        self.assertEqual(svg.count(">NA</text>"), 2)

    def test_rejects_empty_labels(self):
        with self.assertRaisesRegex(ValueError, "labels"):
            build_scene({"labels": [], "matrix": []})

    def test_rejects_non_square_matrix(self):
        with self.assertRaisesRegex(ValueError, "square"):
            build_scene({"labels": ["alpha", "beta"], "matrix": [[1.0]]})

    def test_rejects_out_of_range_values(self):
        with self.assertRaisesRegex(ValueError, "between -1 and 1"):
            build_scene({"labels": ["alpha"], "matrix": [[1.2]]})

    def test_rejects_boolean_as_correlation(self):
        with self.assertRaisesRegex(ValueError, "numbers or null"):
            build_scene({"labels": ["alpha"], "matrix": [[True]]})

    def test_rejects_non_string_annotation(self):
        with self.assertRaisesRegex(ValueError, "annotation"):
            build_scene({"labels": ["alpha"], "matrix": [[1.0]], "annotation": 123})


if __name__ == "__main__":
    unittest.main()
