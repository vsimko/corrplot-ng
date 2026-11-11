"""Semantic validator for normalized-matrix v1 fixtures and adapter output."""
from __future__ import annotations

import math
from typing import Any

SOURCE_FORMATS = {"r_matrix", "r_data_frame", "csv", "tsv", "json", "inline"}


def validate_normalized_matrix(value: Any) -> None:
    if not isinstance(value, dict):
        raise ValueError("normalized matrix must be a JSON object")
    required = {"schema_version", "kind", "labels", "values", "source", "source_indices"}
    missing = sorted(required - value.keys())
    if missing:
        raise ValueError(f"missing required fields: {', '.join(missing)}")
    allowed = required | {"metadata"}
    unknown = sorted(value.keys() - allowed)
    if unknown:
        raise ValueError(f"unknown top-level fields: {', '.join(unknown)}")
    if type(value["schema_version"]) is not int or value["schema_version"] != 1:
        raise ValueError("schema_version must be 1")
    if value["kind"] != "normalized_matrix":
        raise ValueError("kind must be 'normalized_matrix'")

    labels = value["labels"]
    if not isinstance(labels, list) or not labels:
        raise ValueError("labels must be a non-empty array")
    for index, label in enumerate(labels):
        if not isinstance(label, str) or not label.strip():
            raise ValueError(f"labels[{index}] must be a non-empty string")
    if len(set(labels)) != len(labels):
        raise ValueError("labels must be unique")

    size = len(labels)
    rows = value["values"]
    if not isinstance(rows, list) or len(rows) != size:
        raise ValueError("values must be a square matrix with one row per label")
    for row_index, row in enumerate(rows):
        if not isinstance(row, list) or len(row) != size:
            raise ValueError(f"values[{row_index}] must contain exactly {size} columns")
        for column_index, cell in enumerate(row):
            if cell is None:
                continue
            if isinstance(cell, bool) or not isinstance(cell, (int, float)) or not math.isfinite(cell):
                raise ValueError(
                    f"values[{row_index}][{column_index}] must be a finite number or null"
                )

    source = value["source"]
    if not isinstance(source, dict):
        raise ValueError("source must be an object")
    source_required = {"format", "row_labels_present", "column_labels_present"}
    source_missing = sorted(source_required - source.keys())
    if source_missing:
        raise ValueError(f"source is missing required fields: {', '.join(source_missing)}")
    source_unknown = sorted(source.keys() - (source_required | {"path"}))
    if source_unknown:
        raise ValueError(f"unknown source fields: {', '.join(source_unknown)}")
    if not isinstance(source["format"], str) or source["format"] not in SOURCE_FORMATS:
        raise ValueError(f"source.format must be one of {', '.join(sorted(SOURCE_FORMATS))}")
    for field in ("row_labels_present", "column_labels_present"):
        if not isinstance(source[field], bool):
            raise ValueError(f"source.{field} must be a boolean")
    if "path" in source and (not isinstance(source["path"], str) or not source["path"]):
        raise ValueError("source.path must be a non-empty string when present")

    indices = value["source_indices"]
    if not isinstance(indices, list) or len(indices) != size:
        raise ValueError("source_indices length must match labels")
    if any(isinstance(index, bool) or not isinstance(index, int) or index < 0 for index in indices):
        raise ValueError("source_indices must contain non-negative integers")
    if len(set(indices)) != len(indices):
        raise ValueError("source_indices must be unique")

    if "metadata" in value and not isinstance(value["metadata"], dict):
        raise ValueError("metadata must be an object when present")
