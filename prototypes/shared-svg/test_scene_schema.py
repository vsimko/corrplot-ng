"""Validate the prepared scene against the published experimental JSON Schema."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from scene import build_scene


HERE = Path(__file__).parent
SCHEMA_PATH = HERE / "scene-v1.schema.json"
EXAMPLE_PATH = HERE / "example.json"


class SceneSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.config = json.loads(EXAMPLE_PATH.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(cls.schema)

    def test_generated_scene_matches_schema(self):
        scene = build_scene(self.config)
        errors = sorted(
            Draft202012Validator(self.schema).iter_errors(scene),
            key=lambda error: list(map(str, error.path)),
        )
        self.assertEqual(
            errors,
            [],
            "\n".join(f"{list(error.path)}: {error.message}" for error in errors),
        )

    def test_variable_order_and_cell_p_values_are_part_of_scene_contract(self):
        config = dict(self.config)
        config["variable_order"] = {
            "strategy": "alphabetical",
            "indices": [1, 0, 2],
            "labels": ["beta", "alpha", "gamma"],
        }
        config["p_values"] = [
            [0.0, 0.01, 0.02],
            [0.01, 0.0, 0.03],
            [0.02, 0.03, 0.0],
        ]
        scene = build_scene(config)
        self.assertEqual(scene["variable_order"]["indices"], [1, 0, 2])
        self.assertEqual(scene["cells"][1]["p_value"], 0.01)
        self.assertEqual(list(Draft202012Validator(self.schema).iter_errors(scene)), [])

    def test_schema_rejects_unknown_top_level_fields(self):
        scene = build_scene(self.config)
        scene["renderer_specific_hint"] = "should not be in the shared contract"
        errors = list(Draft202012Validator(self.schema).iter_errors(scene))
        self.assertTrue(errors)

    def test_legacy_label_metadata_matches_scene_schema(self):
        scene = build_scene(self.config)
        for label in scene["labels"]:
            label.update({
                "row_visible": True, "column_visible": True,
                "row_position": "left", "column_position": "top",
                "column_rotation": 90, "offset": 0.4,
                "plotmath": {"prefix": ":", "source": "alpha + beta"},
            })
        self.assertEqual(list(Draft202012Validator(self.schema).iter_errors(scene)), [])


if __name__ == "__main__":
    unittest.main()
