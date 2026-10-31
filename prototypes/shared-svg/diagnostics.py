"""Language-neutral diagnostic records for corrplot-ng scene processing."""
from __future__ import annotations

from dataclasses import asdict, dataclass
import json
import re
from typing import Any


@dataclass(frozen=True)
class Diagnostic:
    code: str
    severity: str
    stage: str
    message: str
    path: str | None = None
    row: int | None = None
    column: int | None = None
    details: dict[str, Any] | None = None

    def to_dict(self) -> dict[str, Any]:
        return {"schema_version": 1, **{
            key: value for key, value in asdict(self).items() if value is not None
        }}

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), ensure_ascii=False, sort_keys=True)


class DiagnosticError(ValueError):
    """Public exception carrying a stable diagnostic record."""

    def __init__(self, diagnostic: Diagnostic):
        self.diagnostic = diagnostic
        super().__init__(diagnostic.message)


def scene_error_diagnostic(message: str) -> Diagnostic:
    """Map common scene validation failures to stable, actionable categories."""
    lowered = message.lower()
    if "square" in lowered or "match labels" in lowered:
        return Diagnostic("input.matrix.invalid_shape", "error", "input", message, path="matrix")
    if "label" in lowered:
        return Diagnostic("input.matrix.invalid_label", "error", "input", message, path="labels")
    if "finite" in lowered or "infinite" in lowered or "nan" in lowered:
        coordinates = re.search(r"row (\\d+), column (\\d+)", message)
        row, column = (int(coordinates.group(1)), int(coordinates.group(2))) if coordinates else (None, None)
        return Diagnostic("input.matrix.non_finite_value", "error", "input", message,
                          path="matrix", row=row, column=column)
    if "numeric" in lowered or "must be a number" in lowered or "non-numeric" in lowered:
        return Diagnostic("input.matrix.non_numeric", "error", "input", message, path="matrix")
    if "unsupported cell method" in lowered:
        return Diagnostic("renderer.unsupported_feature", "error", "renderer", message, path="glyph_method")
    if "scene" in lowered or "annotation" in lowered or "significance" in lowered or "cluster region" in lowered or "cluster_regions" in lowered:
        path = "cluster_regions" if "cluster region" in lowered or "cluster_regions" in lowered else None
        return Diagnostic("scene.invalid_content", "error", "scene", message, path=path)
    return Diagnostic("configuration.invalid_field", "error", "configuration", message)


def build_scene_with_diagnostics(config: dict[str, Any]) -> dict[str, Any]:
    """Build a scene and expose validation failures as diagnostic exceptions."""
    from scene import build_scene

    try:
        return build_scene(config)
    except ValueError as error:
        raise DiagnosticError(scene_error_diagnostic(str(error))) from None
