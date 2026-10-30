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
        cls.correlation_fixtures = json.loads(
            (HERE.parents[1] / "tests" / "fixtures" / "correlation-cases.json").read_text(encoding="utf-8")
        )["cases"]
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
        self.assertEqual(build_scene(self.config), self.golden_scene)

    def test_cells_are_in_row_major_order(self):
        cells = build_scene(self.config)["cells"]
        self.assertEqual(
            [(cell["row"], cell["column"]) for cell in cells],
            [(row, column) for row in range(3) for column in range(3)],
        )

    def test_all_shared_correlation_fixtures_build_ordered_scenes(self):
        for case in self.correlation_fixtures:
            with self.subTest(case=case["id"]):
                scene = build_scene({"labels": case["labels"], "matrix": case["matrix"]})
                size = len(case["labels"])
                self.assertEqual(
                    [label["text"] for label in scene["labels"]],
                    case["labels"],
                )
                self.assertEqual(
                    [(cell["row"], cell["column"]) for cell in scene["cells"]],
                    [(row, column) for row in range(size) for column in range(size)],
                )
                self.assertEqual(
                    [cell["value"] for cell in scene["cells"]],
                    [value for row in case["matrix"] for value in row],
                )

    def test_long_labels_expand_row_and_column_label_margins(self):
        labels = ["customer satisfaction score", "monthly recurring revenue"]
        scene = build_scene({
            "labels": labels,
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
        })
        self.assertGreater(scene["cells"][0]["x"], 104)
        self.assertGreater(scene["cells"][0]["y"], 76)
        self.assertGreaterEqual(
            scene["labels"][0]["row_x"] - len(labels[0]) * 7.8,
            0,
        )
        self.assertLess(scene["labels"][0]["column_y"], scene["cells"][0]["y"])

    def test_long_column_labels_are_rotated_in_svg(self):
        svg = render_svg({
            "labels": ["customer satisfaction score", "monthly recurring revenue"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
        })
        self.assertIn('transform="rotate(-90 ', svg)

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

    def test_scene_preserves_optional_mapped_colors_and_legend(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.0], [0.0, None]],
            "cell_colors": ["#ff0000", "#ffffff", "#ffffff", "#d9d9d9"],
            "legend": [
                {"kind": "stop", "label": "-1", "color": "#0000ff"},
                {"kind": "stop", "label": "0", "color": "#ffffff"},
                {"kind": "stop", "label": "+1", "color": "#ff0000"},
                {"kind": "missing", "label": "Missing", "color": "#d9d9d9"},
            ],
        })
        self.assertEqual(
            [cell["color"] for cell in scene["cells"]],
            ["#ff0000", "#ffffff", "#ffffff", "#d9d9d9"],
        )
        self.assertEqual(scene["legend"][-1]["label"], "Missing")

    def test_rejects_mismatched_cell_color_count(self):
        with self.assertRaisesRegex(ValueError, "one color per matrix cell"):
            build_scene({
                "labels": ["A", "B"],
                "matrix": [[1.0, 0.0], [0.0, 1.0]],
                "cell_colors": ["#ff0000"],
            })

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


    def test_layout_can_set_physical_dimensions_and_cell_size(self):
        scene = build_scene({
            "labels": ["a", "b"],
            "matrix": [[1.0, 0.2], [0.2, 1.0]],
            "layout": {"width": 520, "height": 420, "cell_size": 72},
        })
        self.assertEqual(scene["width"], 520)
        self.assertEqual(scene["height"], 420)
        self.assertEqual(scene["cells"][0]["size"], 72)
        self.assertEqual(scene["cells"][1]["x"] - scene["cells"][0]["x"], 72)

    def test_dense_matrix_respects_minimum_readable_cell_size(self):
        scene = build_scene({
            "labels": [f"v{i}" for i in range(12)],
            "matrix": [[1.0 if i == j else 0.1 for j in range(12)] for i in range(12)],
            "layout": {"width": 360, "height": 360, "min_cell_size": 12},
        })
        self.assertGreaterEqual(scene["cells"][0]["size"], 12)
        self.assertEqual(len(scene["cells"]), 144)

    def test_custom_label_typography_is_serialized(self):
        config = {
            "labels": ["long variable", "another variable"],
            "matrix": [[1.0, 0.3], [0.3, 1.0]],
            "layout": {
                "label_font_family": "serif",
                "label_font_size": 16,
                "label_font_weight": "bold",
                "column_label_orientation": "horizontal",
            },
        }
        svg = render_svg(config)
        self.assertIn('font-family="serif"', svg)
        self.assertIn('font-size="16"', svg)
        self.assertIn('font-weight="bold"', svg)
        self.assertNotIn('rotate(-90', svg)

    def test_label_collision_policies_are_applied(self):
        config = {
            "labels": ["long variable name", "second long variable name"],
            "matrix": [[1.0, 0.3], [0.3, 1.0]],
            "layout": {"label_collision": "truncate"},
        }
        svg = render_svg(config)
        self.assertIn("…</text>", svg)
        omitted = render_svg({**config, "layout": {"label_collision": "omit"}})
        self.assertNotIn(">long variable name</text>", omitted)

    def test_invalid_layout_options_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "label_font_size"):
            build_scene({
                "labels": ["a"], "matrix": [[1.0]],
                "layout": {"label_font_size": 100},
            })
        with self.assertRaisesRegex(ValueError, "column_label_orientation"):
            build_scene({
                "labels": ["a"], "matrix": [[1.0]],
                "layout": {"column_label_orientation": "diagonal"},
            })



    def test_horizontal_labels_can_wrap_without_losing_text(self):
        svg = render_svg({
            "labels": ["customer satisfaction score", "monthly recurring revenue"],
            "matrix": [[1.0, 0.3], [0.3, 1.0]],
            "layout": {
                "column_label_orientation": "horizontal",
                "label_collision": "wrap",
                "label_font_size": 16,
                "cell_size": 36,
            },
        })
        self.assertIn("<tspan", svg)
        self.assertIn("customer", svg)
        self.assertIn("satisfaction", svg)

if __name__ == "__main__":
    unittest.main()
