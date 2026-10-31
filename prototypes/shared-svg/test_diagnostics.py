"""Tests for the language-neutral diagnostic-v1 contract."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from diagnostics import Diagnostic, DiagnosticError, build_scene_with_diagnostics
from renderer_capabilities import RendererCapabilityError, negotiate_scene_capabilities
from scene import build_scene


HERE = Path(__file__).parent
ROOT = HERE.parents[1]
SCHEMA = json.loads((ROOT / "prototypes" / "diagnostic-v1.schema.json").read_text(encoding="utf-8"))


class DiagnosticContractTests(unittest.TestCase):
    def test_scene_shape_failure_has_stable_machine_readable_diagnostic(self):
        with self.assertRaises(DiagnosticError) as raised:
            build_scene_with_diagnostics({"labels": ["A", "B"], "matrix": [[1.0]]})
        diagnostic = raised.exception.diagnostic
        self.assertEqual(diagnostic.code, "input.matrix.invalid_shape")
        self.assertEqual(diagnostic.stage, "input")
        self.assertEqual(diagnostic.path, "matrix")
        self.assertEqual(list(Draft202012Validator(SCHEMA).iter_errors(diagnostic.to_dict())), [])
        self.assertEqual(json.loads(diagnostic.to_json())["code"], diagnostic.code)

    def test_label_failure_uses_a_stable_code(self):
        with self.assertRaises(DiagnosticError) as raised:
            build_scene_with_diagnostics({
                "labels": ["A", True],
                "matrix": [[1.0, 0.2], [0.2, 1.0]],
            })
        self.assertEqual(raised.exception.diagnostic.code, "input.matrix.invalid_label")

    def test_deprecation_warning_record_follows_diagnostic_v1(self):
        diagnostic = Diagnostic(
            "configuration.option.deprecated",
            "warning",
            "configuration",
            "The legacy option is deprecated; use the replacement option.",
            path="legacy_option",
        )
        self.assertEqual(list(Draft202012Validator(SCHEMA).iter_errors(diagnostic.to_dict())), [])
        self.assertEqual(json.loads(diagnostic.to_json())["severity"], "warning")

    def test_renderer_capability_diagnostics_follow_diagnostic_v1(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["labels"][0]["plotmath"] = {"prefix": ":", "source": "alpha"}
        with self.assertRaises(RendererCapabilityError) as raised:
            negotiate_scene_capabilities(scene)
        diagnostic = raised.exception.diagnostics[0]
        self.assertEqual(list(Draft202012Validator(SCHEMA).iter_errors(diagnostic)), [])
        self.assertEqual(diagnostic["stage"], "renderer")


if __name__ == "__main__":
    unittest.main()
