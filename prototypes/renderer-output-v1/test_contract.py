"""Schema conformance tests for renderer output request v1."""

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


SCHEMA_PATH = Path(__file__).parent.parent / "renderer-output-v1.schema.json"


def request(**overrides):
    value = {
        "schema_version": 1,
        "format": "svg",
        "size": {"kind": "scene_units"},
        "background": {"kind": "color", "rgba": [255, 255, 255, 255]},
        "pixel_format": "rgba8",
        "alpha_mode": "straight",
        "negotiation": "strict",
        "required_features": [],
        "optional_features": [],
    }
    value.update(overrides)
    return value


class RendererOutputContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with SCHEMA_PATH.open(encoding="utf-8") as stream:
            cls.schema = json.load(stream)
        Draft202012Validator.check_schema(cls.schema)
        cls.validator = Draft202012Validator(cls.schema)

    def assert_valid(self, value):
        self.assertEqual(list(self.validator.iter_errors(value)), [])

    def assert_invalid(self, value):
        self.assertTrue(list(self.validator.iter_errors(value)))

    def test_svg_accepts_scene_units_and_physical_or_pixel_dimensions(self):
        self.assert_valid(request())
        self.assert_valid(request(size={
            "kind": "physical", "width": 180, "height": 120, "unit": "mm"
        }))
        self.assert_valid(request(size={"kind": "pixels", "width": 1280, "height": 720}))

    def test_raster_requires_explicit_pixels_or_physical_dpi(self):
        self.assert_invalid(request(format="png"))
        self.assert_invalid(request(format="rgba"))
        self.assert_valid(request(
            format="png",
            size={"kind": "physical", "width": 180, "height": 120, "unit": "mm", "dpi": 300},
        ))
        self.assert_valid(request(
            format="rgba",
            size={"kind": "pixels", "width": 640, "height": 480},
            background={"kind": "transparent"},
            alpha_mode="premultiplied",
        ))

    def test_png_rejects_premultiplied_alpha_and_transparent_opaque(self):
        self.assert_invalid(request(
            format="png",
            size={"kind": "pixels", "width": 640, "height": 480},
            alpha_mode="premultiplied",
        ))
        self.assert_invalid(request(
            format="rgba",
            size={"kind": "pixels", "width": 640, "height": 480},
            background={"kind": "transparent"},
            alpha_mode="opaque",
        ))

    def test_features_are_known_and_unique(self):
        self.assert_valid(request(required_features=["row_labels", "missing_cells"]))
        self.assert_invalid(request(required_features=["unknown_feature"]))
        self.assert_invalid(request(optional_features=["legend", "legend"]))


if __name__ == "__main__":
    unittest.main()
