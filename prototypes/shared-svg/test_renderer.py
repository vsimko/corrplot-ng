import json
import unittest
import xml.etree.ElementTree as ET
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

    def test_svg_accessibility_metadata(self):
        config = {"labels": ["alpha"], "matrix": [[1.0]], "title": "R&D <metrics> – Δ", "description": "Metrics & bounds < 1"}
        root = ET.fromstring(render_svg(config))
        ns = "{http://www.w3.org/2000/svg}"
        self.assertEqual(root.attrib["role"], "img")
        self.assertEqual(root.attrib["aria-labelledby"], "figure-title")
        self.assertEqual(root.attrib["aria-describedby"], "figure-description")
        self.assertEqual(root.find(f"{ns}title").text, "R&D <metrics> – Δ")
        self.assertEqual(root.find(f"{ns}desc").text, "Metrics & bounds < 1")

    def test_empty_description_has_no_dangling_reference(self):
        root = ET.fromstring(render_svg({"labels": ["alpha"], "matrix": [[1.0]]}))
        self.assertNotIn("aria-describedby", root.attrib)
        self.assertIsNone(root.find("{http://www.w3.org/2000/svg}desc"))

    def test_metadata_must_be_strings(self):
        for field in ("title", "description"):
            with self.subTest(field=field):
                with self.assertRaisesRegex(ValueError, field):
                    build_scene({"labels": ["alpha"], "matrix": [[1.0]], field: 123})

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

    def test_general_numeric_scene_preserves_values_and_uses_mapped_colors(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[12.0, 8.0], [8.0, 10.0]],
            "matrix_mode": "general_numeric",
            "matrix_semantics": {
                "mode": "general_numeric", "color_limits": [8.0, 12.0],
                "color_mapping": "sequential",
                "normalization": {"intercept": -8.0, "zoom": 0.25},
            },
            "cell_colors": ["#b2182b", "#f7f7f7", "#f7f7f7", "#d6604d"],
        })
        self.assertEqual(scene["kind"], "general_matrix_plot")
        self.assertEqual(scene["cells"][0]["value"], 12.0)
        self.assertEqual(scene["cells"][0]["normalized_value"], 1.0)
        svg = render_scene_svg(scene)
        self.assertIn('fill="#b2182b"', svg)
        self.assertIn('r="36.12"', svg)

    def test_renderer_uses_mapped_missing_color(self):
        scene = build_scene({
            "labels": ["A"], "matrix": [[None]],
            "cell_colors": ["#abcdef"],
        })
        self.assertIn('fill="#abcdef"', render_scene_svg(scene))

    def test_general_numeric_accepts_values_outside_correlation_domain(self):
        scene = build_scene({
            "labels": ["A"], "matrix": [[125.0]], "matrix_mode": "general_numeric",
            "matrix_semantics": {
                "mode": "general_numeric", "color_limits": [124.5, 125.5],
                "color_mapping": "sequential",
                "normalization": {"intercept": -124.5, "zoom": 1.0},
            },
        })
        self.assertEqual(scene["kind"], "general_matrix_plot")
        self.assertEqual(scene["cells"][0]["value"], 125.0)

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


    def test_coefficient_labels_are_separate_cell_annotations(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, -0.125], [-0.125, None]],
            "add_coef_color": "#222222",
        })
        labels = scene["text_annotations"]
        self.assertEqual([item["text"] for item in labels], ["1.00", "-0.12", "-0.12"])
        self.assertTrue(all(item["role"] == "coefficient" for item in labels))
        self.assertTrue(all(item["color"] == "#222222" for item in labels))
        self.assertEqual([(item["row"], item["column"]) for item in labels],
                         [(0, 0), (0, 1), (1, 0)])

    def test_percent_formatting_and_custom_number_typography(self):
        scene = build_scene({
            "labels": ["A"], "matrix": [[0.456]],
            "add_coef_color": "navy", "add_coef_as_percent": True,
            "number_digits": 1, "number_cex": 1.5, "number_font": 4,
        })
        label = scene["text_annotations"][0]
        self.assertEqual(label["text"], "45.6")
        self.assertEqual(label["font_size"], 19.5)
        self.assertEqual(label["font_weight"], "bold")
        self.assertEqual(label["font_style"], "italic")
        self.assertIn('font-style="italic"', render_scene_svg(scene))
        self.assertIn('fill="navy">45.6</text>', render_scene_svg(scene))

    def test_number_method_takes_precedence_over_coefficient_overlay(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, -0.5], [-0.5, 1.0]],
            "method": "number", "add_coef_color": "red",
            "number_digits": 1, "number_font": 3,
        })
        self.assertEqual(scene["glyph_method"], "number")
        self.assertTrue(all(item["role"] == "value" for item in scene["text_annotations"]))
        self.assertNotIn("red", [item["color"] for item in scene["text_annotations"]])
        svg = render_scene_svg(scene)
        self.assertNotIn("<circle ", svg)
        self.assertIn(">1.0</text>", svg)
        self.assertIn('font-style="italic"', svg)

    def test_coefficient_labels_respect_triangle_and_diagonal_visibility(self):
        scene = build_scene({
            "labels": ["A", "B", "C"],
            "matrix": [[1.0, 0.2, 0.3], [0.2, 1.0, 0.4], [0.3, 0.4, 1.0]],
            "add_coef_color": "black", "display_type": "upper",
            "show_diagonal": False,
        })
        self.assertEqual(
            [(item["row"], item["column"]) for item in scene["text_annotations"]],
            [(0, 1), (0, 2), (1, 2)],
        )


    def test_legacy_default_digits_and_rounding_boundaries(self):
        decimal = build_scene({
            "labels": ["A", "B"],
            "matrix": [[0.125, -0.125], [-0.125, 0.0]],
            "add_coef_color": "black",
        })
        self.assertEqual(
            [item["text"] for item in decimal["text_annotations"]],
            ["0.12", "-0.12", "-0.12", "0.00"],
        )
        percent = build_scene({
            "labels": ["A"], "matrix": [[0.125]],
            "add_coef_color": "black", "add_coef_as_percent": True,
        })
        self.assertEqual(percent["text_annotations"][0]["text"], "12")

    def test_invalid_coefficient_formatting_options_are_rejected(self):
        base = {"labels": ["A"], "matrix": [[1.0]]}
        for key, value in (("number_digits", -1), ("number_digits", 1.5),
                           ("number_cex", 0), ("number_font", 5),
                           ("add_coef_as_percent", "yes"),
                           ("display_type", "sideways")):
            with self.subTest(option=key, value=value):
                with self.assertRaises(ValueError):
                    build_scene({**base, key: value, "add_coef_color": "black"})
    def test_svg_renderer_rejects_plotmath_instead_of_silently_misrendering(self):
        scene = build_scene({"labels": [":alpha + beta"], "matrix": [[1.0]]})
        scene["labels"][0]["plotmath"] = {"prefix": ":", "source": "alpha + beta"}
        with self.assertRaisesRegex(ValueError, "does not interpret R plotmath"):
            render_scene_svg(scene)

    def test_svg_renderer_respects_label_visibility_and_rotation(self):
        scene = build_scene({"labels": ["alpha", "beta"], "matrix": [[1.0, 0.2], [0.2, 1.0]]})
        scene["labels"][0].update({"row_visible": False, "column_visible": True, "column_rotation": 35})
        scene["labels"][1].update({"row_visible": True, "column_visible": False})
        svg = render_scene_svg(scene)
        self.assertEqual(svg.count(">alpha</text>"), 1)
        self.assertIn("rotate(-35", svg)
        self.assertEqual(svg.count(">beta</text>"), 1)




    def test_scene_prepares_renderer_neutral_geometry_for_each_method(self):
        methods = ("circle", "square", "ellipse", "number", "shade", "color", "pie")
        for method in methods:
            with self.subTest(method=method):
                scene = build_scene({
                    "labels": ["A", "B"],
                    "matrix": [[1.0, -0.5], [-0.5, 1.0]],
                    "method": method,
                })
                encoding = scene["cells"][1]["encoding"]
                self.assertEqual(encoding["kind"], method)
                self.assertEqual(encoding["sign"], -1)
                self.assertEqual(encoding["magnitude"], 0.5)
                if method == "ellipse":
                    self.assertEqual(encoding["rotation"], -45)
                if method == "pie":
                    self.assertEqual(encoding["sweep_flag"], 0)
                if method == "shade":
                    self.assertLess(encoding["line"]["y1"], encoding["line"]["y2"])



    def test_all_seven_methods_have_distinct_deterministic_svg_encodings(self):
        examples = {
            "circle": "<circle cx=",
            "square": 'stroke="none"/>',
            "ellipse": "<ellipse cx=",
            "number": ">0.65</text>",
            "shade": "<line x1=",
            "color": 'fill="#',
            "pie": "<path d=",
        }
        outputs = {}
        for method, marker in examples.items():
            with self.subTest(method=method):
                config = {
                    "labels": ["A", "B"],
                    "matrix": [[1.0, 0.65], [0.65, 1.0]],
                    "method": method,
                }
                scene = build_scene(config)
                first = render_scene_svg(scene)
                self.assertEqual(first, render_scene_svg(build_scene(config)))
                ET.fromstring(first)
                self.assertIn(marker, first)
                outputs[method] = first
        self.assertEqual(len(set(outputs.values())), len(outputs))

    def test_signed_encodings_preserve_sign_and_magnitude(self):
        positive = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, 0.75], [0.75, 1.0]],
            "method": "ellipse",
        })
        negative = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, -0.75], [-0.75, 1.0]],
            "method": "ellipse",
        })
        self.assertIn('rotate(45 ', positive)
        self.assertIn('rotate(-45 ', negative)

        positive_pie = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, 0.25], [0.25, 1.0]],
            "method": "pie",
        })
        negative_pie = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, -0.25], [-0.25, 1.0]],
            "method": "pie",
        })
        self.assertIn("0 0 1", positive_pie)
        self.assertIn("0 0 0", negative_pie)

    def test_color_method_interpolates_magnitude_from_neutral(self):
        low = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, 0.2], [0.2, 1.0]],
            "method": "color",
        })
        high = render_svg({
            "labels": ["A", "B"], "matrix": [[1.0, 0.8], [0.8, 1.0]],
            "method": "color",
        })
        self.assertNotEqual(low, high)
        self.assertIn('fill="#', low)
        self.assertIn('fill="#', high)

    def test_mixed_triangle_methods_are_applied_per_cell(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.5], [0.5, 1.0]],
            "upper_method": "square",
            "lower_method": "pie",
            "diagonal_method": "number",
        })
        self.assertEqual([cell["method"] for cell in scene["cells"]],
                         ["number", "square", "pie", "number"])
        svg = render_scene_svg(scene)
        self.assertIn('stroke="none"/>', svg)
        self.assertIn("<path d=", svg)
        self.assertIn(">1.00</text>", svg)


    def test_zero_is_visually_distinct_from_missing_for_shade_and_color(self):
        for method in ("shade", "color"):
            with self.subTest(method=method):
                scene = build_scene({
                    "labels": ["A", "B"],
                    "matrix": [[1.0, 0.0], [0.0, None]],
                    "method": method,
                })
                zero = scene["cells"][1]["encoding"]
                self.assertEqual(zero["sign"], 0)
                self.assertEqual(zero["magnitude"], 0)
                svg = render_scene_svg(scene)
                self.assertIn("#bdbdbd", svg)
                self.assertIn(">NA</text>", svg)



    def test_missing_values_remain_explicit_for_every_method(self):
        for method in ("circle", "square", "ellipse", "number", "shade", "color", "pie"):
            with self.subTest(method=method):
                svg = render_svg({
                    "labels": ["A", "B"],
                    "matrix": [[1.0, None], [None, 1.0]],
                    "method": method,
                })
                self.assertEqual(svg.count(">NA</text>"), 2)




    def test_significance_modes_prepare_deterministic_annotations(self):
        base = {
            "labels": ["A", "B", "C"],
            "matrix": [[1.0, 0.6, 0.2], [0.6, 1.0, 0.3], [0.2, 0.3, 1.0]],
            "p_values": [[0.0, 0.001, None], [0.001, 0.0, 0.2], [None, 0.2, 0.0]],
            "significance": {"mode": "stars", "thresholds": [0.001, 0.01, 0.05], "symbol": "×"},
        }
        stars = build_scene(base)
        self.assertEqual(stars["cells"][1]["significance"]["annotation"], "***")
        self.assertEqual(stars["cells"][2]["significance"]["status"], "unknown")
        self.assertIsNone(stars["cells"][2]["significance"]["annotation"])

        pch = build_scene({**base, "significance": {**base["significance"], "mode": "pch"}})
        self.assertEqual(pch["cells"][5]["significance"]["annotation"], "×")
        p_value = build_scene({**base, "significance": {**base["significance"], "mode": "p-value"}})
        self.assertEqual(p_value["cells"][5]["significance"]["annotation"], "p=0.2")
        blank = build_scene({**base, "significance": {**base["significance"], "mode": "blank"}})
        self.assertTrue(blank["cells"][5]["significance"]["blank"])
        self.assertFalse(blank["cells"][2]["significance"]["blank"])

    def test_confidence_intervals_prepare_whisker_geometry_and_render(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.6], [0.6, 1.0]],
            "confidence_intervals": {
                "lower": [[1.0, 0.2], [0.2, 1.0]],
                "upper": [[1.0, 0.8], [0.8, 1.0]],
            },
        })
        interval = scene["cells"][1]["confidence_interval"]
        self.assertEqual(interval["lower"], 0.2)
        self.assertEqual(interval["upper"], 0.8)
        self.assertAlmostEqual(interval["x1"], scene["cells"][1]["x"] + scene["cells"][1]["size"] * 0.6)
        self.assertAlmostEqual(interval["x2"], scene["cells"][1]["x"] + scene["cells"][1]["size"] * 0.9)
        svg = render_scene_svg(scene)
        ET.fromstring(svg)
        self.assertIn('stroke="#333333"', svg)
        self.assertEqual(svg, render_scene_svg(scene))


    def test_significance_threshold_boundaries_are_inclusive_and_ordered(self):
        p_values = [
            [0.0, 0.001, 0.001001, 0.01],
            [0.001, 0.0, 0.01, 0.05],
            [0.001001, 0.01, 0.0, 0.050001],
            [0.01, 0.05, 0.050001, 0.0],
        ]
        scene = build_scene({
            "labels": ["A", "B", "C", "D"],
            "matrix": [[1.0 if i == j else 0.2 for j in range(4)] for i in range(4)],
            "p_values": p_values,
            "significance": {"mode": "stars", "thresholds": [0.001, 0.01, 0.05], "symbol": "×"},
        })
        cells = {(cell["row"], cell["column"]): cell for cell in scene["cells"]}
        self.assertEqual(cells[(0, 1)]["significance"]["annotation"], "***")
        self.assertEqual(cells[(0, 2)]["significance"]["annotation"], "**")
        self.assertEqual(cells[(0, 3)]["significance"]["annotation"], "**")
        self.assertEqual(cells[(1, 2)]["significance"]["annotation"], "**")
        self.assertEqual(cells[(1, 3)]["significance"]["annotation"], "*")
        self.assertIsNone(cells[(2, 3)]["significance"]["annotation"])

    def test_confidence_interval_must_contain_correlation_estimate(self):
        with self.assertRaisesRegex(ValueError, "must contain the correlation estimate"):
            build_scene({
                "labels": ["A", "B"],
                "matrix": [[1.0, 0.6], [0.6, 1.0]],
                "confidence_intervals": {
                    "lower": [[1.0, 0.7], [0.7, 1.0]],
                    "upper": [[1.0, 0.8], [0.8, 1.0]],
                },
            })



    def test_missing_p_values_remain_unknown_and_do_not_blank_cells(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
            "p_values": [[0.0, None], [None, 0.0]],
            "significance": {"mode": "blank", "thresholds": [0.05], "symbol": "×"},
        })
        cell = scene["cells"][1]
        self.assertEqual(cell["significance"]["status"], "unknown")
        self.assertFalse(cell["significance"]["blank"])
        svg = render_scene_svg(scene)
        self.assertIn("<circle", svg)

    def test_blank_significance_mode_hides_glyphs_and_value_annotations(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
            "p_values": [[0.0, 0.2], [0.2, 0.0]],
            "significance": {"mode": "blank", "thresholds": [0.05], "symbol": "×"},
            "method": "number",
        })
        svg = render_scene_svg(scene)
        self.assertNotIn(">0.40</text>", svg)
        self.assertIn(">1.00</text>", svg)



if __name__ == "__main__":
    unittest.main()
