"""Renderer-neutral color scale semantics for correlation and matrix values.

Signed continuous/discrete scales cover [-1, 1]. Sequential continuous scales
cover normalized general-matrix values in [0, 1]. Continuous scales interpolate
sRGB channels between explicit stops. Discrete scales use left-closed,
right-open intervals, except that the final interval includes its upper endpoint.
Missing values are handled separately.
"""
from __future__ import annotations

import math
import re
from typing import Any

_HEX = re.compile(r"^#[0-9A-Fa-f]{6}$")


def _color(value: Any, path: str) -> str:
    if not isinstance(value, str) or not _HEX.fullmatch(value):
        raise ValueError(f"{path}: expected a six-digit sRGB hex color")
    return value.lower()


def _finite_number(value: Any, path: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{path}: expected a finite number")
    number = float(value)
    if not math.isfinite(number):
        raise ValueError(f"{path}: expected a finite number")
    return number


def _validate_domain(values: list[float], path: str) -> None:
    if not values or values[0] != -1.0 or values[-1] != 1.0:
        raise ValueError(f"{path}: scale must cover the full correlation domain [-1, 1]")
    if any(left >= right for left, right in zip(values, values[1:])):
        raise ValueError(f"{path}: values must be strictly increasing")


def _validate_symmetric_domain(values: list[float], path: str) -> None:
    if any(not math.isclose(left, -right, rel_tol=0.0, abs_tol=1e-12)
           for left, right in zip(values, reversed(values))):
        raise ValueError(f"{path}: signed correlation scales must be symmetric around zero")


def _mix(left: str, right: str, fraction: float) -> str:
    channels = []
    for offset in (1, 3, 5):
        a = int(left[offset:offset + 2], 16)
        b = int(right[offset:offset + 2], 16)
        channels.append(round(a + (b - a) * fraction))
    return "#" + "".join(f"{channel:02x}" for channel in channels)


class ColorScale:
    """Validated color mapping and renderer-neutral legend description."""

    def __init__(self, spec: dict[str, Any], missing_color: str = "#d9d9d9"):
        if not isinstance(spec, dict):
            raise ValueError("color_scale: expected an object")
        self.missing_color = _color(missing_color, "missing_color")
        self.kind = spec.get("type")

        if self.kind == "continuous":
            if spec.get("interpolation", "rgb") != "rgb":
                raise ValueError("color_scale.interpolation: only 'rgb' is supported")
            stops = spec.get("stops")
            if not isinstance(stops, list) or len(stops) < 3:
                raise ValueError("color_scale.stops: expected at least three stops including -1, 0, and 1")
            values = [_finite_number(stop.get("value"), f"color_scale.stops[{i}].value")
                      for i, stop in enumerate(stops) if isinstance(stop, dict)]
            if len(values) != len(stops):
                raise ValueError("color_scale.stops: each stop must be an object")
            _validate_domain(values, "color_scale.stops")
            _validate_symmetric_domain(values, "color_scale.stops")
            if 0.0 not in values:
                raise ValueError("color_scale.stops: a neutral stop at zero is required")
            self.stops = [(value, _color(stop.get("color"), f"color_scale.stops[{i}].color"))
                          for i, (value, stop) in enumerate(zip(values, stops))]
            self.breaks = []
            self.colors = []
        elif self.kind == "sequential":
            if spec.get("interpolation", "rgb") != "rgb":
                raise ValueError("color_scale.interpolation: only 'rgb' is supported")
            stops = spec.get("stops")
            if not isinstance(stops, list) or len(stops) < 2:
                raise ValueError("color_scale.stops: sequential scales require at least two stops")
            values = [_finite_number(stop.get("value"), f"color_scale.stops[{i}].value")
                      for i, stop in enumerate(stops) if isinstance(stop, dict)]
            if len(values) != len(stops):
                raise ValueError("color_scale.stops: each stop must be an object")
            if values[0] != 0.0 or values[-1] != 1.0:
                raise ValueError("color_scale.stops: sequential scales must cover [0, 1]")
            if any(left >= right for left, right in zip(values, values[1:])):
                raise ValueError("color_scale.stops: values must be strictly increasing")
            self.stops = [(value, _color(stop.get("color"), f"color_scale.stops[{i}].color")
                           ) for i, (value, stop) in enumerate(zip(values, stops))]
            self.breaks = []
            self.colors = []
        elif self.kind == "discrete":
            raw_breaks = spec.get("breaks")
            raw_colors = spec.get("colors")
            if not isinstance(raw_breaks, list) or not isinstance(raw_colors, list):
                raise ValueError("color_scale: discrete scales require breaks and colors arrays")
            values = [_finite_number(value, f"color_scale.breaks[{i}]")
                      for i, value in enumerate(raw_breaks)]
            _validate_domain(values, "color_scale.breaks")
            _validate_symmetric_domain(values, "color_scale.breaks")
            if 0.0 not in values:
                raise ValueError("color_scale.breaks: zero must be an explicit boundary")
            if len(raw_colors) != len(values) - 1:
                raise ValueError("color_scale.colors: expected one color per interval")
            self.breaks = values
            self.colors = [_color(value, f"color_scale.colors[{i}]")
                           for i, value in enumerate(raw_colors)]
            self.stops = []
        else:
            raise ValueError("color_scale.type: expected 'continuous' or 'discrete'")

    def map_value(self, value: Any) -> str:
        """Map a normalized scale value to a six-digit sRGB hex color."""
        if value is None:
            return self.missing_color
        number = _finite_number(value, "value")
        lower, upper = (0.0, 1.0) if self.kind == "sequential" else (-1.0, 1.0)
        if not lower <= number <= upper:
            domain = "[0, 1]" if self.kind == "sequential" else "[-1, 1]"
            raise ValueError(f"value: scale value must be within {domain}")

        if self.kind == "discrete":
            for index, (lower, upper) in enumerate(zip(self.breaks, self.breaks[1:])):
                if lower <= number < upper or (index == len(self.colors) - 1 and number == upper):
                    return self.colors[index]
            raise AssertionError("validated discrete scale did not cover the value")

        for (left_value, left_color), (right_value, right_color) in zip(self.stops, self.stops[1:]):
            if left_value <= number <= right_value:
                fraction = (number - left_value) / (right_value - left_value)
                return _mix(left_color, right_color, fraction)
        raise AssertionError("validated continuous scale did not cover the value")

    def legend_entries(self) -> list[dict[str, Any]]:
        """Return stable, backend-neutral legend entries in ascending order."""
        if self.kind == "discrete":
            entries = [
                {"kind": "interval", "lower": lower, "upper": upper,
                 "include_upper": index == len(self.colors) - 1,
                 "color": color}
                for index, (lower, upper, color) in enumerate(
                    zip(self.breaks, self.breaks[1:], self.colors)
                )
            ]
        else:
            entries = [
                {"kind": "stop", "value": value, "color": color}
                for value, color in self.stops
            ]
        entries.append({"kind": "missing", "color": self.missing_color})
        return entries


def legacy_palette_scale(colors: list[str]) -> dict[str, Any]:
    """Adapt a legacy corrplot color vector to the shared discrete scale.

    Colors are assigned to equal-width intervals over [-1, 1]. If the number
    of colors is odd, the central interval is split at zero and its color is
    repeated, preserving the original mapping while satisfying the shared
    scale's explicit-zero-boundary contract.
    """
    if not isinstance(colors, list) or not colors:
        raise ValueError("legacy_palette: expected a non-empty list of colors")
    normalized = [
        _color(color, f"legacy_palette.colors[{index}]")
        for index, color in enumerate(colors)
    ]
    count = len(normalized)
    breaks = [-1.0 + 2.0 * index / count for index in range(count + 1)]
    if count % 2:
        midpoint = count // 2
        breaks.insert(midpoint + 1, 0.0)
        normalized.insert(midpoint + 1, normalized[midpoint])
    return {"type": "discrete", "breaks": breaks, "colors": normalized}


def build_color_scale(spec: dict[str, Any], missing_color: str = "#d9d9d9") -> ColorScale:
    """Construct a validated color scale."""
    return ColorScale(spec, missing_color)
