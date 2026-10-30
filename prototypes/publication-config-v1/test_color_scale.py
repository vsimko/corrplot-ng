import unittest

from color_scale import build_color_scale


class ColorScaleTests(unittest.TestCase):
    def setUp(self):
        self.continuous = {
            "type": "continuous",
            "interpolation": "rgb",
            "stops": [
                {"value": -1, "color": "#2166ac"},
                {"value": 0, "color": "#f7f7f7"},
                {"value": 1, "color": "#b2182b"},
            ],
        }

    def test_continuous_endpoints_zero_and_interpolation(self):
        scale = build_color_scale(self.continuous)
        self.assertEqual(scale.map_value(-1), "#2166ac")
        self.assertEqual(scale.map_value(0), "#f7f7f7")
        self.assertEqual(scale.map_value(1), "#b2182b")
        self.assertEqual(scale.map_value(-0.5), "#8caed2")
        self.assertEqual(scale.map_value(None), "#d9d9d9")

    def test_discrete_intervals_have_explicit_boundary_semantics(self):
        scale = build_color_scale({
            "type": "discrete",
            "breaks": [-1, -0.5, 0, 0.5, 1],
            "colors": ["#053061", "#4393c3", "#d6604d", "#67001f"],
        })
        self.assertEqual(scale.map_value(-1), "#053061")
        self.assertEqual(scale.map_value(-0.5), "#4393c3")
        self.assertEqual(scale.map_value(0), "#d6604d")
        self.assertEqual(scale.map_value(0.5), "#67001f")
        self.assertEqual(scale.map_value(1), "#67001f")
        self.assertEqual(scale.legend_entries()[0]["include_upper"], False)
        self.assertEqual(scale.legend_entries()[-2]["include_upper"], True)

    def test_legend_contains_missing_entry(self):
        entries = build_color_scale(self.continuous).legend_entries()
        self.assertEqual(entries[-1], {"kind": "missing", "color": "#d9d9d9"})

    def test_rejects_incomplete_domain_or_missing_zero(self):
        for stops in (
            [{"value": -0.5, "color": "#000000"},
             {"value": 0, "color": "#ffffff"},
             {"value": 1, "color": "#ff0000"}],
            [{"value": -1, "color": "#000000"},
             {"value": 0.5, "color": "#ffffff"},
             {"value": 1, "color": "#ff0000"}],
        ):
            with self.subTest(stops=stops):
                with self.assertRaises(ValueError):
                    build_color_scale({"type": "continuous", "stops": stops})

    def test_rejects_asymmetric_signed_domains(self):
        with self.assertRaisesRegex(ValueError, "symmetric around zero"):
            build_color_scale({
                "type": "continuous",
                "stops": [
                    {"value": -1, "color": "#0000ff"},
                    {"value": 0, "color": "#ffffff"},
                    {"value": 0.8, "color": "#ff0000"},
                    {"value": 1, "color": "#ff0000"},
                ],
            })
        with self.assertRaisesRegex(ValueError, "symmetric around zero"):
            build_color_scale({
                "type": "discrete",
                "breaks": [-1, -0.2, 0, 0.5, 1],
                "colors": ["#0000ff", "#8888ff", "#ff8888", "#ff0000"],
            })

    def test_rejects_unsorted_or_duplicate_breaks(self):
        with self.assertRaisesRegex(ValueError, "strictly increasing"):
            build_color_scale({
                "type": "discrete", "breaks": [-1, 0, 0, 1],
                "colors": ["#000000", "#888888", "#ffffff"],
            })

    def test_rejects_wrong_color_count_and_invalid_colors(self):
        with self.assertRaisesRegex(ValueError, "one color per interval"):
            build_color_scale({
                "type": "discrete", "breaks": [-1, 0, 1],
                "colors": ["#000000"],
            })
        with self.assertRaisesRegex(ValueError, "six-digit"):
            build_color_scale({
                "type": "discrete", "breaks": [-1, 0, 1],
                "colors": ["red", "#ffffff"],
            })

    def test_rejects_out_of_range_and_non_finite_values(self):
        scale = build_color_scale(self.continuous)
        for value in (-1.01, 1.01, float("nan"), float("inf"), True):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    scale.map_value(value)

    def test_legacy_palette_vectors_adapt_without_changing_color_assignment(self):
        from color_scale import legacy_palette_scale

        for count in (1, 2, 3, 11, 200):
            with self.subTest(count=count):
                colors = [
                    f"#{index:02x}{(255 - index):02x}7f"
                    for index in range(count)
                ]
                spec = legacy_palette_scale(colors)
                scale = build_color_scale(spec)
                self.assertEqual(spec["type"], "discrete")
                self.assertEqual(spec["breaks"][0], -1.0)
                self.assertEqual(spec["breaks"][-1], 1.0)
                self.assertIn(0.0, spec["breaks"])
                self.assertEqual(len(spec["colors"]), len(spec["breaks"]) - 1)
                for index, color in enumerate(colors):
                    lower, upper = spec["breaks"][index], spec["breaks"][index + 1]
                    self.assertEqual(scale.map_value((lower + upper) / 2), color)

    def test_legacy_palette_adapter_validates_colors(self):
        from color_scale import legacy_palette_scale

        for value in ([], "red", ["red"], ["#123456", None]):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    legacy_palette_scale(value)


if __name__ == "__main__":
    unittest.main()
