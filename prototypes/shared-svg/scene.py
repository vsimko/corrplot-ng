"""Renderer-neutral scene model for correlation plots.

The scene is plain JSON-compatible data: language adapters can construct it,
and independent renderers can consume it without sharing implementation code.
This module owns validation and geometry, not SVG serialization.
"""
from __future__ import annotations

from typing import Any


def build_scene(config: dict[str, Any]) -> dict[str, Any]:
    """Validate a correlation-plot config and return a deterministic plot scene."""
    if not isinstance(config, dict):
        raise ValueError("config must be an object")

    labels = config.get("labels")
    matrix = config.get("matrix")
    annotation = config.get("annotation", "")
    if not isinstance(labels, list) or not labels:
        raise ValueError("labels must be a non-empty list")
    if any(not isinstance(label, (str, int, float)) for label in labels):
        raise ValueError("labels must contain only strings or numbers")
    if not isinstance(matrix, list) or len(matrix) != len(labels):
        raise ValueError("matrix must be square and match labels")
    if any(not isinstance(row, list) or len(row) != len(labels) for row in matrix):
        raise ValueError("matrix must be square and match labels")
    if not isinstance(annotation, str):
        raise ValueError("annotation must be a string")

    for row in matrix:
        for value in row:
            if value is None:
                continue
            if isinstance(value, bool) or not isinstance(value, (int, float)):
                raise ValueError("correlation values must be numbers or null")
            if not -1 <= value <= 1:
                raise ValueError("correlation values must be null or between -1 and 1")

    cell = 84
    left, top = 104, 76
    width = left + len(labels) * cell + 24
    height = top + len(labels) * cell + 64
    cells = []
    for row_index, row in enumerate(matrix):
        for column_index, value in enumerate(row):
            cells.append({
                "row": row_index,
                "column": column_index,
                "value": value,
                "x": left + column_index * cell,
                "y": top + row_index * cell,
                "center_x": left + column_index * cell + cell / 2,
                "center_y": top + row_index * cell + cell / 2,
                "size": cell,
            })

    label_items = []
    for index, label in enumerate(labels):
        label_items.append({
            "text": str(label),
            "row_x": left - 12,
            "row_y": top + index * cell + cell / 2,
            "column_x": left + index * cell + cell / 2,
            "column_y": top - 16,
        })

    return {
        "schema_version": 1,
        "kind": "correlation_plot",
        "width": width,
        "height": height,
        "labels": label_items,
        "cells": cells,
        "annotation": annotation,
        "style": {
            "positive_color": "#2166ac",
            "negative_color": "#b2182b",
            "zero_color": "#bdbdbd",
            "missing_fill": "#f2f2f2",
            "grid_color": "#e0e0e0",
        },
    }
