"""Scene/renderer capability negotiation for the experimental SVG backend."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path
from typing import Any


CAPABILITIES_PATH = Path(__file__).with_name("renderer-capabilities.json")
PYTHON_SVG_CAPABILITIES = json.loads(CAPABILITIES_PATH.read_text(encoding="utf-8"))


class RendererCapabilityError(ValueError):
    """Raised when a renderer cannot preserve required scene semantics."""

    def __init__(self, diagnostics: list[dict[str, Any]]):
        self.diagnostics = diagnostics
        details = "; ".join(item["message"] for item in diagnostics)
        super().__init__(details)


def _scene_features(scene: dict[str, Any]) -> set[str]:
    features = set()
    if any(label.get("plotmath") is not None for label in scene.get("labels", [])):
        features.add("plotmath_labels")
    if scene.get("legend"):
        features.add("legend")
    if scene.get("text_annotations"):
        features.add("cell_text_annotations")
    if scene.get("significance") or any(cell.get("significance") for cell in scene.get("cells", [])):
        features.add("significance")
    if any(cell.get("confidence_interval") is not None for cell in scene.get("cells", [])):
        features.add("confidence_intervals")
    if scene.get("figure_text"):
        features.add("figure_text")
    return features


def negotiate_scene_capabilities(
    scene: dict[str, Any],
    capabilities: dict[str, Any] | None = None,
    *,
    mode: str = "strict",
    optional_features: tuple[str, ...] | list[str] = (),
) -> list[dict[str, Any]]:
    """Validate scene requirements; permissive mode only omits caller-marked optional features."""
    capabilities = capabilities or PYTHON_SVG_CAPABILITIES
    renderer = capabilities["renderer"]
    if mode not in capabilities["negotiation_modes"]:
        raise ValueError(f"renderer {renderer['id']} does not support {mode!r} negotiation")
    if scene.get("schema_version") not in capabilities["scene_schema_versions"]:
        raise RendererCapabilityError([{
            "code": "renderer.schema_version_unsupported", "severity": "error",
            "renderer": renderer["id"], "feature": "scene_schema_version",
            "message": f"renderer {renderer['id']} does not support scene schema version {scene.get('schema_version')!r}",
        }])
    kind = scene.get("kind")
    unsupported = []
    if kind not in capabilities["matrix_kinds"]:
        unsupported.append(("matrix_kind", kind))
    methods = {
        cell.get("encoding", {}).get("kind", cell.get("method", scene.get("glyph_method", "circle")))
        for cell in scene.get("cells", [])
    }
    for method in sorted(methods - set(capabilities["cell_methods"])):
        unsupported.append(("cell_method", method))
    for feature in sorted(_scene_features(scene)):
        if not capabilities["features"].get(feature, False):
            unsupported.append((feature, feature))

    optional = set(optional_features)
    diagnostics = []
    errors = []
    for category, feature in unsupported:
        may_omit = mode == "permissive" and feature in optional
        message = (
            "the SVG renderer does not interpret R plotmath labels; use the R graphics renderer or convert the label explicitly"
            if feature == "plotmath_labels"
            else f"renderer {renderer['id']} does not support {category} {feature!r}"
        )
        diagnostic = {
            "code": "renderer.optional_feature_omitted" if may_omit else "renderer.unsupported_feature",
            "severity": "warning" if may_omit else "error",
            "renderer": renderer["id"],
            "renderer_version": renderer["version"],
            "feature": feature,
            "message": message,
        }
        diagnostics.append(diagnostic)
        if not may_omit:
            errors.append(diagnostic)
    if errors:
        raise RendererCapabilityError(errors)
    return diagnostics


def render_scene_svg_with_diagnostics(
    scene: dict[str, Any],
    *,
    mode: str = "strict",
    optional_features: tuple[str, ...] | list[str] = (),
) -> tuple[str, list[dict[str, Any]]]:
    """Render SVG and return a structured warning for each explicitly omitted feature."""
    diagnostics = negotiate_scene_capabilities(
        scene, mode=mode, optional_features=optional_features
    )
    prepared = deepcopy(scene)
    for item in diagnostics:
        feature = item["feature"]
        if feature == "plotmath_labels":
            for label in prepared.get("labels", []):
                if label.get("plotmath") is not None:
                    label["row_visible"] = False
                    label["column_visible"] = False
                    label.pop("plotmath", None)
        elif feature == "legend":
            prepared.pop("legend", None)
        elif feature == "cell_text_annotations":
            prepared.pop("text_annotations", None)
        elif feature == "significance":
            prepared.pop("significance", None)
            for cell in prepared.get("cells", []):
                cell.pop("significance", None)
        elif feature == "confidence_intervals":
            for cell in prepared.get("cells", []):
                cell.pop("confidence_interval", None)
        elif feature == "figure_text":
            prepared.pop("figure_text", None)

    from renderer import _render_scene_svg_unchecked
    return _render_scene_svg_unchecked(prepared), diagnostics
