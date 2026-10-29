import json
import unittest
from pathlib import Path

from renderer import render_svg


HERE = Path(__file__).parent


class SvgPrototypeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.config = json.loads((HERE / "example.json").read_text(encoding="utf-8"))

    def test_output_is_deterministic(self):
        first = render_svg(self.config)
        second = render_svg(self.config)
        self.assertEqual(first, second)

    def test_emits_svg_and_user_annotation(self):
        svg = render_svg(self.config)
        self.assertTrue(svg.startswith('<svg xmlns="http://www.w3.org/2000/svg"'))
        self.assertIn("Prototype annotation: synthetic example only", svg)
        self.assertEqual(svg.count("<circle "), 7)

    def test_missing_values_are_explicit(self):
        svg = render_svg(self.config)
        self.assertEqual(svg.count(">NA</text>"), 2)

    def test_rejects_non_square_matrix(self):
        invalid = {"labels": ["alpha", "beta"], "matrix": [[1.0]]}
        with self.assertRaises(ValueError):
            render_svg(invalid)

    def test_rejects_out_of_range_values(self):
        invalid = {"labels": ["alpha"], "matrix": [[1.2]]}
        with self.assertRaises(ValueError):
            render_svg(invalid)


if __name__ == "__main__":
    unittest.main()
