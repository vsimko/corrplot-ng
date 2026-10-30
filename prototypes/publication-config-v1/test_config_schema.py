"""Validate publication configuration examples and semantic invariants."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from config import canonical_json, validate_config


HERE = Path(__file__).parent
SCHEMA = json.loads((HERE / "schema.json").read_text(encoding="utf-8"))
EXAMPLES = sorted((HERE / "examples").glob("*.json"))


class PublicationConfigTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        Draft202012Validator.check_schema(SCHEMA)

    def test_all_examples_are_valid_and_normalizable(self):
        self.assertEqual(len(EXAMPLES), 3)
        for path in EXAMPLES:
            with self.subTest(example=path.name):
                config = json.loads(path.read_text(encoding="utf-8"))
                validate_config(config)

    def test_schema_rejects_unknown_fields_and_versions(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["plot"]["renderer_magic"] = True
        self.assertTrue(list(Draft202012Validator(SCHEMA).iter_errors(config)))
        config["plot"].pop("renderer_magic")
        config["config_version"] = 2
        self.assertTrue(list(Draft202012Validator(SCHEMA).iter_errors(config)))

    def test_rejects_rectangular_matrix(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0].pop()
        with self.assertRaisesRegex(ValueError, "square"):
            validate_config(config)

    def test_rejects_asymmetric_matrix(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0][1] = 0.2
        with self.assertRaisesRegex(ValueError, "symmetric"):
            validate_config(config)

    def test_rejects_invalid_correlation_diagonal(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0][0] = 0.8
        with self.assertRaisesRegex(ValueError, "diagonal must be 1"):
            validate_config(config)

    def test_rejects_duplicate_labels(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["labels"][1] = config["input"]["labels"][0]
        with self.assertRaisesRegex(ValueError, "unique"):
            validate_config(config)

    def test_rejects_significance_matrix_with_wrong_dimensions(self):
        config = json.loads((HERE / "examples" / "significance.json").read_text(encoding="utf-8"))
        config["plot"]["significance"]["p_values"].pop()
        with self.assertRaisesRegex(ValueError, "p_values.*square"):
            validate_config(config)

    def test_canonical_json_is_deterministic(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        self.assertEqual(canonical_json(config), canonical_json(config))
        self.assertEqual(json.loads(canonical_json(config)), config)


if __name__ == "__main__":
    unittest.main()
