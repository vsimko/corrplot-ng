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

    def test_schema_rejects_unknown_top_level_fields(self):
        scene = build_scene(self.config)
        scene["renderer_specific_hint"] = "should not be in the shared contract"
        errors = list(Draft202012Validator(self.schema).iter_errors(scene))
        self.assertTrue(errors)


if __name__ == "__main__":
    unittest.main()
