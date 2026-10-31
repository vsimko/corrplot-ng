#!/usr/bin/env python3
"""Validate the self-contained SVG subset emitted by corrplot-ng.

This deliberately uses only the Python standard library. It is a structural
validator, not a claim of complete SVG 2 conformance or browser equivalence.
"""
from __future__ import annotations

import argparse
import math
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

SVG_NS = "http://www.w3.org/2000/svg"
XLINK_NS = "http://www.w3.org/1999/xlink"
DIMENSION = re.compile(r"^(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:px|pt|pc|mm|cm|in)?$")
URL_REFERENCE = re.compile(r"url\(([^)]+)\)")
ALLOWED_ELEMENTS = {
    "svg", "title", "desc", "g", "defs", "rect", "circle", "ellipse",
    "path", "line", "polyline", "polygon", "text", "tspan", "clipPath",
    "mask", "linearGradient", "radialGradient", "stop",
}
FORBIDDEN_ELEMENTS = {"script", "foreignObject", "image", "iframe", "object", "embed", "link"}


class SvgValidationError(ValueError):
    """A diagnosable failure in the supported self-contained SVG subset."""


def _local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _element_name(element: ET.Element) -> str:
    name = _local_name(element.tag)
    identifier = element.attrib.get("id")
    return f"{name}#{identifier}" if identifier else name


def _parse_positive_dimension(value: str | None, name: str) -> float:
    if value is None or not DIMENSION.fullmatch(value.strip()):
        raise SvgValidationError(f"root @{name} must be a positive absolute SVG dimension")
    numeric = re.match(r"^([0-9]+(?:\.[0-9]*)?|\.[0-9]+)", value.strip())
    assert numeric is not None
    number = float(numeric.group(1))
    if not math.isfinite(number) or number <= 0:
        raise SvgValidationError(f"root @{name} must be finite and greater than zero")
    return number


def validate_svg_text(source_text: str, source: str = "<memory>") -> dict[str, Any]:
    """Validate XML structure, geometry, references, and self-containment."""
    try:
        root = ET.fromstring(source_text)
    except ET.ParseError as error:
        raise SvgValidationError(f"{source}: malformed XML: {error}") from error

    if root.tag != f"{{{SVG_NS}}}svg":
        raise SvgValidationError(f"{source}: root element must be in the SVG namespace")

    width = _parse_positive_dimension(root.attrib.get("width"), "width")
    height = _parse_positive_dimension(root.attrib.get("height"), "height")
    view_box = root.attrib.get("viewBox")
    if view_box is None:
        raise SvgValidationError(f"{source}: root @viewBox is required")
    try:
        view_values = [float(item) for item in re.split(r"[ ,]+", view_box.strip()) if item]
    except ValueError as error:
        raise SvgValidationError(f"{source}: root @viewBox must contain four numbers") from error
    if len(view_values) != 4 or not all(math.isfinite(item) for item in view_values):
        raise SvgValidationError(f"{source}: root @viewBox must contain four finite numbers")
    if view_values[2] <= 0 or view_values[3] <= 0:
        raise SvgValidationError(f"{source}: root @viewBox width and height must be positive")

    identifiers: dict[str, ET.Element] = {}
    elements = list(root.iter())
    for element in elements:
        name = _local_name(element.tag)
        label = _element_name(element)
        if name in FORBIDDEN_ELEMENTS:
            raise SvgValidationError(f"{source}: forbidden external/active element <{name}>")
        if name not in ALLOWED_ELEMENTS:
            raise SvgValidationError(f"{source}: unsupported SVG element <{name}>")
        identifier = element.attrib.get("id")
        if identifier:
            if identifier in identifiers:
                raise SvgValidationError(f"{source}: duplicate SVG id {identifier!r} at <{label}>")
            identifiers[identifier] = element

        for attribute, value in element.attrib.items():
            attribute_name = _local_name(attribute).lower()
            if attribute_name.startswith("on"):
                raise SvgValidationError(f"{source}: event handler @{attribute_name} is not allowed on <{label}>")
            if attribute_name in {"href"}:
                if not value.startswith("#"):
                    raise SvgValidationError(f"{source}: external href {value!r} is not allowed on <{label}>")
            if attribute_name in {"style"} and re.search(r"@import|url\((?!\s*['\"]?#)", value, re.IGNORECASE):
                raise SvgValidationError(f"{source}: external CSS dependency is not allowed on <{label}>")
            if "javascript:" in value.lower():
                raise SvgValidationError(f"{source}: JavaScript URL is not allowed on <{label}>")
            if attribute_name in {"aria-labelledby", "aria-describedby"}:
                for reference in value.split():
                    if not reference.startswith("#") and reference not in identifiers:
                        # References may point forward; resolve after collecting all IDs.
                        pass
            for match in URL_REFERENCE.findall(value):
                reference = match.strip().strip("'\"")
                if not reference.startswith("#"):
                    raise SvgValidationError(f"{source}: external URL reference {reference!r} is not allowed on <{label}>")

    for element in elements:
        label = _element_name(element)
        for attribute, value in element.attrib.items():
            attribute_name = _local_name(attribute).lower()
            if attribute_name in {"href"} and value.startswith("#"):
                reference = value[1:]
                if reference not in identifiers:
                    raise SvgValidationError(f"{source}: broken href reference #{reference} on <{label}>")
            if attribute_name in {"aria-labelledby", "aria-describedby"}:
                for reference in value.split():
                    if reference not in identifiers:
                        raise SvgValidationError(f"{source}: broken @{attribute_name} reference {reference!r} on <{label}>")
            for match in URL_REFERENCE.findall(value):
                reference = match.strip().strip("'\"")
                if reference.startswith("#") and reference[1:] not in identifiers:
                    raise SvgValidationError(f"{source}: broken URL reference {reference} on <{label}>")

    return {
        "elements": len(elements),
        "ids": len(identifiers),
        "width": width,
        "height": height,
        "view_box": view_values,
    }


def validate_svg_file(path: str | Path) -> dict[str, Any]:
    source = Path(path)
    try:
        content = source.read_text(encoding="utf-8")
    except OSError as error:
        raise SvgValidationError(f"{source}: cannot read SVG: {error}") from error
    return validate_svg_text(content, str(source))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("svg", type=Path, nargs="+", help="SVG artifact(s) to validate")
    args = parser.parse_args()
    try:
        for path in args.svg:
            result = validate_svg_file(path)
            print(
                f"VALID SVG {path}: {result['elements']} elements, "
                f"{result['ids']} ids, {result['width']:g} x {result['height']:g}"
            )
    except SvgValidationError as error:
        print(f"SVG VALIDATION ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
