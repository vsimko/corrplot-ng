"""Tests for renderer-neutral figure-level text roles and layout."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from renderer import render_scene_svg
from scene import build_scene


HERE = Path(__file__).parent
SCHEMA = json.loads((HERE / "scene-v1.schema.json").read_text(encoding="utf-8"))


class FigureTextTests(unittest.TestCase):
    def test_empty_figure_text_keeps_accessible_metadata_without_visible_duplicate(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
            "title": "Accessible figure name",
            "description": "A concise accessible description.",
        })
        self.assertNotIn("figure_text", scene)
        svg = render_scene_svg(scene)
        self.assertIn("<title id=\"figure-title\">Accessible figure name</title>", svg)
        self.assertNotIn("data-role=", svg)

    def test_all_roles_are_ordered_and_validated_against_scene_schema(self):
        scene = build_scene({
            "labels": ["A", "B"],
            "matrix": [[1.0, 0.4], [0.4, 1.0]],
            "figure_text": [
                {"role": "footnote", "text": "Footnote"},
                {"role": "caption", "text": "Source: synthetic data"},
                {"role": "subtitle", "text": "A short explanation"},
                {"role": "title", "text": "Correlation overview"},
            ],
        })
        self.assertEqual(
            [item["role"] for item in scene["figure_text"]],
            ["title", "subtitle", "caption", "footnote"],
        )
        self.assertEqual(list(Draft202012Validator(SCHEMA).iter_errors(scene)), [])
        svg = render_scene_svg(scene)
        for text in ("Correlation overview", "A short explanation",
                     "Source: synthetic data", "Footnote"):
            self.assertIn(text, svg)

    def test_long_text_wraps_without_truncation_and_expands_canvas(self):
        long_text = " ".join(["correlation"] * 160)
        base = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene = build_scene({
            "labels": ["A"],
            "matrix": [[1.0]],
            "layout": {"width": 360},
            "figure_text": [{"role": "caption", "text": long_text}],
        })
        caption = scene["figure_text"][0]
        self.assertGreater(len(caption["lines"]), 1)
        self.assertEqual(" ".join(caption["lines"]), long_text)
        self.assertGreater(scene["height"], base["height"])
        self.assertNotIn("…", render_scene_svg(scene))

    def test_explicit_newlines_unicode_and_xml_characters_are_preserved_safely(self):
        text = "Résumé — α & β\nSecond line <safe>"
        scene = build_scene({
            "labels": ["A"],
            "matrix": [[1.0]],
            "figure_text": [{"role": "title", "text": text}],
        })
        title = scene["figure_text"][0]
        self.assertEqual(title["lines"], ["Résumé — α & β", "Second line <safe>"])
        svg = render_scene_svg(scene)
        self.assertIn("Résumé — α &amp; β", svg)
        self.assertIn("Second line &lt;safe&gt;", svg)
        self.assertNotIn("Second line <safe>", svg)

    def test_duplicate_roles_and_invalid_typography_are_rejected(self):
        config = {"labels": ["A"], "matrix": [[1.0]], "figure_text": [
            {"role": "title", "text": "First"},
            {"role": "title", "text": "Second"},
        ]}
        with self.assertRaisesRegex(ValueError, "duplicate role"):
            build_scene(config)
        config["figure_text"] = [
            {"role": "caption", "text": "Caption", "alignment": "justify"},
        ]
        with self.assertRaisesRegex(ValueError, "alignment"):
            build_scene(config)


if __name__ == "__main__":
    unittest.main()
