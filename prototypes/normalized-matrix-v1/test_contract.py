from __future__ import annotations

import json
import unittest
from pathlib import Path

import jsonschema

from validate import validate_normalized_matrix


ROOT = Path(__file__).resolve().parent
SCHEMA = json.loads((ROOT / "schema.json").read_text(encoding="utf-8"))
VALID = json.loads((ROOT / "fixtures" / "valid-cases.json").read_text(encoding="utf-8"))
INVALID = json.loads((ROOT / "fixtures" / "invalid-cases.json").read_text(encoding="utf-8"))


class NormalizedMatrixV1Tests(unittest.TestCase):
    def test_valid_fixtures_match_schema_and_semantic_contract(self) -> None:
        for case in VALID["cases"]:
            with self.subTest(case=case["id"]):
                jsonschema.validate(case["input"], SCHEMA)
                validate_normalized_matrix(case["input"])

    def test_invalid_fixtures_fail_with_the_documented_semantic_error(self) -> None:
        for case in INVALID["cases"]:
            with self.subTest(case=case["id"]):
                with self.assertRaisesRegex(ValueError, case["expected_error"]):
                    validate_normalized_matrix(case["input"])

    def test_schema_rejects_invalid_primitive_types_and_duplicate_items(self) -> None:
        for case_id in (
            "duplicate-labels",
            "empty-label",
            "non-numeric-cell",
            "nan-cell",
            "duplicate-source-index",
            "unsupported-source-format",
        ):
            case = next(item for item in INVALID["cases"] if item["id"] == case_id)
            with self.subTest(case=case_id), self.assertRaises(jsonschema.ValidationError):
                jsonschema.validate(case["input"], SCHEMA)

    def test_all_missing_and_constant_zero_are_valid_normalized_inputs(self) -> None:
        all_missing = next(item["input"] for item in VALID["cases"] if item["id"] == "all-missing-matrix")
        all_zero = next(item["input"] for item in VALID["cases"] if item["id"] == "constant-zero-matrix")
        validate_normalized_matrix(all_missing)
        validate_normalized_matrix(all_zero)
        self.assertTrue(all(value is None for row in all_missing["values"] for value in row))
        self.assertTrue(all(value == 0.0 for row in all_zero["values"] for value in row))

    def test_null_and_numeric_zero_remain_distinct(self) -> None:
        case = next(item["input"] for item in VALID["cases"] if item["id"] == "missing-and-zero")
        self.assertIsNone(case["values"][0][1])
        self.assertEqual(case["values"][0][2], 0.0)
        validate_normalized_matrix(case)

    def test_unicode_labels_are_preserved_without_normalization(self) -> None:
        case = next(item["input"] for item in VALID["cases"] if item["id"] == "unicode-labels")
        self.assertEqual(case["labels"], ["Δ temperature", "湿度"])
        validate_normalized_matrix(case)


if __name__ == "__main__":
    unittest.main()
