"""Validate the structure of shared correlation test fixtures.

These tests validate test data only. They do not test corrplot behavior.
"""

import json
import math
import unittest
from pathlib import Path


FIXTURE_PATH = Path(__file__).parent / "fixtures" / "correlation-cases.json"
EXPECTED_METHODS = {
    "circle",
    "square",
    "ellipse",
    "number",
    "shade",
    "color",
    "pie",
}


class FixtureIntegrityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with FIXTURE_PATH.open(encoding="utf-8") as fixture_file:
            cls.data = json.load(fixture_file)

    def test_schema_version_is_supported(self):
        self.assertEqual(self.data.get("schema_version"), 1)

    def test_all_legacy_plot_methods_are_listed(self):
        self.assertEqual(set(self.data["legacy_plot_methods"]), EXPECTED_METHODS)

    def test_case_ids_are_unique_and_present(self):
        case_ids = [case.get("id") for case in self.data["cases"]]
        self.assertTrue(case_ids)
        self.assertTrue(all(isinstance(case_id, str) and case_id for case_id in case_ids))
        self.assertEqual(len(case_ids), len(set(case_ids)))

    def test_each_matrix_matches_its_labels(self):
        for case in self.data["cases"]:
            with self.subTest(case=case.get("id")):
                labels = case["labels"]
                matrix = case["matrix"]
                size = len(labels)
                self.assertGreater(size, 0)
                self.assertEqual(len(set(labels)), size)
                self.assertEqual(len(matrix), size)
                self.assertTrue(all(len(row) == size for row in matrix))

    def test_matrix_is_symmetric_and_values_are_valid(self):
        for case in self.data["cases"]:
            with self.subTest(case=case.get("id")):
                matrix = case["matrix"]
                size = len(matrix)
                for row_index, row in enumerate(matrix):
                    self.assertEqual(row[row_index], 1.0)
                    for column_index, value in enumerate(row):
                        mirror = matrix[column_index][row_index]
                        if value is None:
                            self.assertIsNone(mirror)
                            continue
                        self.assertIsInstance(value, (int, float))
                        self.assertTrue(math.isfinite(value))
                        self.assertGreaterEqual(value, -1.0)
                        self.assertLessEqual(value, 1.0)
                        self.assertEqual(value, mirror)

    def test_cases_declare_expected_properties(self):
        required = {"square", "symmetric", "unit_diagonal", "values_in_range"}
        for case in self.data["cases"]:
            with self.subTest(case=case.get("id")):
                self.assertTrue(required.issubset(set(case["expected_properties"])))
                self.assertTrue(case.get("description"))


if __name__ == "__main__":
    unittest.main()
