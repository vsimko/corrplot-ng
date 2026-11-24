"""Tests for the legacy feature matrix's deterministic invariants."""

from __future__ import annotations

import copy
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "repository"))

from validate_legacy_feature_matrix import derive_classification, validate_document  # noqa: E402


class FeatureMatrixClassificationTests(unittest.TestCase):
    def test_all_four_derived_classes(self):
        self.assertIsNone(derive_classification("verified", "verified"))
        self.assertEqual(derive_classification("verified", "verified", parity_verified=True), "both")
        self.assertEqual(derive_classification("verified", "unsupported"), "legacy-only")
        self.assertEqual(derive_classification("unsupported", "verified"), "rust-only")
        self.assertEqual(derive_classification("unsupported", "unsupported"), "neither")

    def test_unknown_and_not_applicable_are_not_misclassified(self):
        self.assertIsNone(derive_classification("verified", "unknown"))
        self.assertIsNone(derive_classification("unknown", "verified"))
        self.assertIsNone(derive_classification("verified", "not_applicable"))


class FeatureMatrixValidationTests(unittest.TestCase):
    def setUp(self):
        self.repo_root = ROOT
        self.matrix_path = ROOT / "r/corrplotng/tests/testthat/fixtures/legacy-api-feature-matrix.json"
        import json

        self.document = json.loads(self.matrix_path.read_text(encoding="utf-8"))

    def test_checked_in_matrix_and_evidence_are_valid(self):
        self.assertEqual(validate_document(self.document, self.repo_root), [])

    def test_rejects_hand_edited_inconsistent_classification(self):
        document = copy.deepcopy(self.document)
        row = next(item for item in document["rows"] if item["feature_id"] == "matrix-input-missing-values")
        row["classification"] = "legacy-only"
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("classification must be derived" in error for error in errors))

    def test_rejects_parity_claim_without_comparative_test(self):
        document = copy.deepcopy(self.document)
        row = next(item for item in document["rows"] if item["feature_id"] == "matrix-input-missing-values")
        row["parity_status"] = "verified"
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("verified parity requires comparative_test" in error for error in errors))

    def test_rejects_support_claim_without_executable_evidence(self):
        document = copy.deepcopy(self.document)
        row = next(item for item in document["rows"] if item["feature_id"] == "matrix-input-missing-values")
        row["evidence"]["rust"] = []
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("rust=verified requires executable test evidence" in error for error in errors))

    def test_rejects_unverifiable_comparative_evidence(self):
        document = copy.deepcopy(self.document)
        row = next(item for item in document["rows"] if item["feature_id"] == "matrix-input-missing-values")
        row["parity_status"] = "verified"
        row["parity_evidence"] = [{"kind": "comparative_test", "path": "missing.py", "test": "missing_test"}]
        row["classification"] = "both"
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("missing comparative evidence test file" in error for error in errors))

    def test_rejects_scene_v1_requirement_without_feature_row(self):
        document = copy.deepcopy(self.document)
        document["scene_v1_requirement_coverage"][0]["feature_ids"] = ["missing-feature"]
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("unknown feature row" in error for error in errors))

    def test_rejects_verified_scene_v1_requirement_without_rust_fixture(self):
        document = copy.deepcopy(self.document)
        requirement = next(
            item for item in document["scene_v1_requirement_coverage"]
            if item["id"] == "all-missing-matrix-state"
        )
        requirement["status"] = "verified"
        errors = validate_document(document, self.repo_root)
        self.assertTrue(any("verified scene-v1 requirement needs executable Rust conformance evidence" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
