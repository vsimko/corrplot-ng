"""Schema and behavior tests for renderer capability negotiation."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from renderer import render_scene_svg
from renderer_capabilities import (
    PYTHON_SVG_CAPABILITIES,
    RendererCapabilityError,
    negotiate_scene_capabilities,
    render_scene_svg_with_diagnostics,
)
from scene import build_scene


HERE = Path(__file__).parent
ROOT = HERE.parents[1]
CAPABILITY_SCHEMA = json.loads(
    (ROOT / "prototypes" / "renderer-capabilities-v1.schema.json").read_text(encoding="utf-8")
)
RUST_CAPABILITIES = json.loads(
    (ROOT / "tools" / "corrplot-ng" / "renderer-capabilities.json").read_text(encoding="utf-8")
)


class RendererCapabilityTests(unittest.TestCase):
    def test_current_renderer_declarations_match_capability_schema(self):
        validator = Draft202012Validator(CAPABILITY_SCHEMA)
        for declaration in (PYTHON_SVG_CAPABILITIES, RUST_CAPABILITIES):
            self.assertEqual(list(validator.iter_errors(declaration)), [])

    def test_capability_schema_separates_browser_canvas_host_targets_from_file_formats(self):
        declaration = dict(PYTHON_SVG_CAPABILITIES)
        declaration["host_targets"] = ["browser-canvas-2d", "browser-offscreen-canvas-2d"]
        validator = Draft202012Validator(CAPABILITY_SCHEMA)
        self.assertEqual(list(validator.iter_errors(declaration)), [])
        self.assertNotIn("canvas", declaration["output_formats"])

    def test_strict_mode_rejects_unsupported_plotmath_before_rendering(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["labels"][0]["plotmath"] = {"prefix": ":", "source": "alpha"}
        with self.assertRaises(RendererCapabilityError) as raised:
            render_scene_svg(scene)
        self.assertEqual(raised.exception.diagnostics[0]["code"], "renderer.unsupported_feature")
        diagnostic = raised.exception.diagnostics[0]
        self.assertEqual(diagnostic["feature"], "plotmath_labels")
        self.assertEqual(diagnostic["code"], "renderer.unsupported_feature")
        self.assertEqual(diagnostic["severity"], "error")
        self.assertEqual(diagnostic["stage"], "renderer")
        self.assertEqual(diagnostic["path"], "labels.plotmath")
        self.assertEqual(diagnostic["renderer_version"], "1")

    def test_permissive_mode_omits_only_explicitly_optional_plotmath_and_reports_it(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["labels"][0]["plotmath"] = {"prefix": ":", "source": "alpha"}
        with self.assertRaises(RendererCapabilityError):
            negotiate_scene_capabilities(scene, mode="permissive")
        svg, diagnostics = render_scene_svg_with_diagnostics(
            scene, mode="permissive", optional_features=["plotmath_labels"]
        )
        self.assertEqual(len(diagnostics), 1)
        self.assertEqual(diagnostics[0]["code"], "renderer.optional_feature_omitted")
        self.assertEqual(diagnostics[0]["severity"], "warning")
        self.assertEqual(diagnostics[0]["stage"], "renderer")
        self.assertEqual(diagnostics[0]["path"], "labels.plotmath")
        self.assertNotIn("alpha", svg)
        self.assertIsNotNone(scene["labels"][0].get("plotmath"))

    def test_permissive_mode_reports_each_optional_omission(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["labels"][0]["plotmath"] = {"prefix": ":", "source": "alpha"}
        scene["legend"] = [{"kind": "stop", "label": "low", "color": "#000000"}]
        svg, diagnostics = render_scene_svg_with_diagnostics(
            scene, mode="permissive", optional_features=["plotmath_labels", "legend"]
        )
        self.assertEqual(
            {item["feature"] for item in diagnostics},
            {"plotmath_labels", "legend"},
        )
        self.assertEqual(len(diagnostics), 2)
        self.assertEqual(
            {item["path"] for item in diagnostics},
            {"labels.plotmath", "legend"},
        )
        self.assertNotIn("alpha", svg)
        self.assertNotIn("low", svg)

    def test_unknown_scene_schema_version_is_never_optional(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["schema_version"] = 2
        with self.assertRaises(RendererCapabilityError) as raised:
            render_scene_svg_with_diagnostics(
                scene, mode="permissive", optional_features=["scene_schema_version"]
            )
        self.assertEqual(
            raised.exception.diagnostics[0]["code"],
            "renderer.schema_version_unsupported",
        )

    def test_unsupported_cell_method_matches_rust_cli_diagnostic_contract(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["cells"][0]["encoding"]["kind"] = "pie"

        with self.assertRaises(RendererCapabilityError) as raised:
            negotiate_scene_capabilities(scene, capabilities=RUST_CAPABILITIES)

        diagnostic = raised.exception.diagnostics[0]
        self.assertEqual(diagnostic["code"], "renderer.unsupported_feature")
        self.assertEqual(diagnostic["severity"], "error")
        self.assertEqual(diagnostic["stage"], "renderer")
        self.assertEqual(diagnostic["path"], "method")
        self.assertEqual(diagnostic["feature"], "pie")

    def test_unsupported_legend_has_stable_diagnostics_for_both_renderer_declarations(self):
        scene = build_scene({"labels": ["A"], "matrix": [[1.0]]})
        scene["legend"] = [{"kind": "stop", "label": "low", "color": "#000000"}]

        for capabilities in (PYTHON_SVG_CAPABILITIES, RUST_CAPABILITIES):
            with self.subTest(renderer=capabilities["renderer"]["id"]):
                with self.assertRaises(RendererCapabilityError) as raised:
                    negotiate_scene_capabilities(scene, capabilities=capabilities)

                diagnostic = raised.exception.diagnostics[0]
                self.assertEqual(diagnostic["code"], "renderer.unsupported_feature")
                self.assertEqual(diagnostic["severity"], "error")
                self.assertEqual(diagnostic["stage"], "renderer")
                self.assertEqual(diagnostic["path"], "legend")
                self.assertEqual(diagnostic["feature"], "legend")
                self.assertEqual(diagnostic["renderer"], capabilities["renderer"]["id"])

    def test_rust_cli_declaration_is_narrow_and_explicit(self):
        self.assertEqual(RUST_CAPABILITIES["cell_methods"], ["circle", "number"])
        self.assertEqual(RUST_CAPABILITIES["negotiation_modes"], ["strict"])
        self.assertFalse(RUST_CAPABILITIES["features"]["legend"])
        self.assertFalse(RUST_CAPABILITIES["features"]["figure_text"])


if __name__ == "__main__":
    unittest.main()
