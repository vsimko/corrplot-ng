"""Regression tests for shared triangular and mixed matrix layouts."""
import unittest
from scene import build_scene
from renderer import render_scene_svg


class TriangleLayoutTests(unittest.TestCase):
    def setUp(self):
        self.config = {
            "labels": ["A", "B", "C"],
            "matrix": [[1.0, 0.2, None], [0.2, 1.0, -0.4], [None, -0.4, 1.0]],
        }

    def test_upper_lower_and_diagonal_visibility_keep_all_source_cells(self):
        expected = {
            "upper": [(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2)],
            "lower": [(0, 0), (1, 0), (1, 1), (2, 0), (2, 1), (2, 2)],
            "diagonal": [(0, 0), (1, 1), (2, 2)],
        }
        for display, positions in expected.items():
            with self.subTest(display=display):
                scene = build_scene({**self.config, "display_type": display})
                self.assertEqual(len(scene["cells"]), 9)
                visible = [(c["row"], c["column"]) for c in scene["cells"] if c["visible"]]
                self.assertEqual(visible, positions)

    def test_hidden_diagonal_is_applied_to_both_triangles(self):
        for display in ("full", "upper", "lower"):
            with self.subTest(display=display):
                scene = build_scene({**self.config, "display_type": display, "show_diagonal": False})
                self.assertFalse(any(c["visible"] and c["row"] == c["column"] for c in scene["cells"]))

    def test_mixed_methods_are_assigned_by_cell_region(self):
        scene = build_scene({
            **self.config, "upper_method": "number", "lower_method": "circle",
            "diagonal_method": "number",
        })
        by_position = {(c["row"], c["column"]): c for c in scene["cells"]}
        self.assertEqual(by_position[(0, 1)]["method"], "number")
        self.assertEqual(by_position[(1, 0)]["method"], "circle")
        self.assertEqual(by_position[(1, 1)]["method"], "number")
        self.assertEqual(by_position[(2, 0)]["region"], "lower")

    def test_annotations_only_target_visible_cells_and_use_region_method(self):
        scene = build_scene({
            **self.config, "display_type": "upper",
            "upper_method": "number", "lower_method": "circle",
            "add_coef_color": "black",
        })
        self.assertEqual(
            [(a["row"], a["column"]) for a in scene["text_annotations"]],
            [(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
        )
        roles = {(a["row"], a["column"]): a["role"] for a in scene["text_annotations"]}
        self.assertEqual(roles[(0, 1)], "value")
        self.assertEqual(roles[(1, 2)], "value")
        self.assertEqual(roles[(0, 0)], "coefficient")

    def test_renderer_skips_hidden_cells_and_keeps_missing_visible_cells(self):
        scene = build_scene({**self.config, "display_type": "upper"})
        svg = render_scene_svg(scene)
        self.assertEqual(svg.count("<rect x="), 6)
        self.assertIn(">NA</text>", svg)

    def test_diagonal_visibility_rejects_conflicting_diagonal_only_request(self):
        with self.assertRaisesRegex(ValueError, "requires show_diagonal"):
            build_scene({**self.config, "display_type": "diagonal", "show_diagonal": False})

    def test_diagonal_only_layout_at_small_and_larger_sizes(self):
        for size in (1, 2, 5):
            labels = [f"v{i}" for i in range(size)]
            matrix = [[1.0 if row == column else 0.25 for column in range(size)]
                      for row in range(size)]
            scene = build_scene({"labels": labels, "matrix": matrix, "display_type": "diagonal"})
            self.assertEqual(
                [(cell["row"], cell["column"]) for cell in scene["cells"] if cell["visible"]],
                [(index, index) for index in range(size)],
            )

    def test_invalid_region_method_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "upper_method"):
            build_scene({**self.config, "upper_method": "ellipse"})


if __name__ == "__main__":
    unittest.main()
