"""Schema and semantic-reference checks for visualization-model-v1 fixtures."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "contracts" / "visualization-model-v1"
FIXTURES = ROOT / "fixtures" / "visualization-model-v1"


def load_schema() -> dict:
    return json.loads((CONTRACT / "schema.json").read_text(encoding="utf-8"))


def load_fixture(name: str) -> dict:
    return json.loads((FIXTURES / name).read_text(encoding="utf-8"))


def validate_document(document: dict, schema: dict) -> None:
    """Validate JSON shape plus invariants that JSON Schema cannot express."""
    jsonschema.validate(document, schema)

    data_sources = {source["id"]: source for source in document["data_sources"]}
    scales = {scale["id"]: scale for scale in document["scales"]}
    plots = document["plots"]

    all_ids: list[str] = []
    all_ids.extend(data_sources)
    all_ids.extend(scales)
    all_ids.extend(plot["id"] for plot in plots)

    for source in data_sources.values():
        if source["kind"] == "matrix":
            rows = source["values"]
            row_labels = source.get("row_labels", source.get("labels"))
            column_labels = source.get("column_labels", source.get("labels"))
            if row_labels is None or column_labels is None:
                raise ValueError(f"matrix source {source['id']} has no complete axis labels")
            if len(rows) != len(row_labels):
                raise ValueError(f"matrix source {source['id']} row count does not match row labels")
            if any(len(row) != len(column_labels) for row in rows):
                raise ValueError(f"matrix source {source['id']} column count does not match column labels")
            if source["semantics"] == "correlation":
                if len(row_labels) != len(column_labels):
                    raise ValueError(f"correlation source {source['id']} must be square")
                for row_index, row in enumerate(rows):
                    for column_index, value in enumerate(row):
                        if value is not None and not (-1.0 <= value <= 1.0):
                            raise ValueError(f"correlation source {source['id']} value is outside [-1, 1]")
                        opposite = rows[column_index][row_index]
                        if (value is None) != (opposite is None) or (
                            value is not None and abs(value - opposite) > 1e-8
                        ):
                            raise ValueError(f"correlation source {source['id']} must be symmetric")
                    if row[row_index] is None or abs(row[row_index] - 1.0) > 1e-8:
                        raise ValueError(f"correlation source {source['id']} diagonal must equal one")
        else:
            lengths = {len(column["values"]) for column in source["columns"].values()}
            if len(lengths) > 1:
                raise ValueError(f"table source {source['id']} columns have different row counts")

    for plot in plots:
        layers = {layer["id"]: layer for layer in plot["layers"]}
        all_ids.extend(layers)
        coordinates = plot["coordinate_system"]
        for scale_key in ("x_scale", "y_scale"):
            scale_id = coordinates.get(scale_key)
            if scale_id is not None and scale_id not in scales:
                raise ValueError(f"plot {plot['id']} references missing scale {scale_id}")

        for layer in layers.values():
            source = data_sources.get(layer["data"])
            if source is None:
                raise ValueError(f"layer {layer['id']} references missing data source {layer['data']}")
            encoding = layer["encoding"]
            scale_ids = [encoding.get("color_scale")]
            scale_ids.extend(
                channel.get("scale")
                for channel in encoding.values()
                if isinstance(channel, dict)
            )
            for scale_id in filter(None, scale_ids):
                if scale_id not in scales:
                    raise ValueError(f"layer {layer['id']} references missing scale {scale_id}")
            if source["kind"] == "table":
                fields = set(source["columns"])
                for channel_name in ("x", "y", "color", "size", "text"):
                    channel = encoding.get(channel_name)
                    if isinstance(channel, dict) and "field" in channel:
                        if channel["field"] not in fields:
                            raise ValueError(
                                f"layer {layer['id']} references missing table field {channel['field']}"
                            )

        all_ids.extend(panel["id"] for panel in plot.get("panels", []))
        all_ids.extend(legend["id"] for legend in plot.get("legends", []))
        all_ids.extend(annotation["id"] for annotation in plot.get("annotations", []))
        for legend in plot.get("legends", []):
            if legend["scale"] not in scales:
                raise ValueError(f"legend {legend['id']} references missing scale {legend['scale']}")
        for panel in plot.get("panels", []):
            for layer_id in panel["layer_ids"]:
                if layer_id not in layers:
                    raise ValueError(f"panel {panel['id']} references missing layer {layer_id}")

    if len(all_ids) != len(set(all_ids)):
        raise ValueError("document IDs must be globally unique")


class VisualizationModelSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = load_schema()
        jsonschema.Draft202012Validator.check_schema(cls.schema)

    def test_all_canonical_examples_validate(self) -> None:
        for name in (
            "legacy-corrplot.json",
            "general-matrix.json",
            "modern-bar-chart.json",
        ):
            with self.subTest(fixture=name):
                validate_document(load_fixture(name), self.schema)

    def test_general_matrix_fixture_is_rectangular(self) -> None:
        document = load_fixture("general-matrix.json")
        source = document["data_sources"][0]
        self.assertEqual(len(source["row_labels"]), 2)
        self.assertEqual(len(source["column_labels"]), 3)
        self.assertEqual([len(row) for row in source["values"]], [3, 3])

    def test_rejects_unknown_document_fields(self) -> None:
        document = load_fixture("legacy-corrplot.json")
        document["renderer"] = "canvas"
        with self.assertRaises(jsonschema.ValidationError):
            validate_document(document, self.schema)

    def test_rejects_matrix_dimension_mismatch(self) -> None:
        document = load_fixture("general-matrix.json")
        document["data_sources"][0]["values"][0].pop()
        with self.assertRaisesRegex(ValueError, "column count"):
            validate_document(document, self.schema)

    def test_rejects_unresolved_layer_data_reference(self) -> None:
        document = load_fixture("modern-bar-chart.json")
        document["plots"][0]["layers"][0]["data"] = "missing-data"
        with self.assertRaisesRegex(ValueError, "missing data source"):
            validate_document(document, self.schema)


if __name__ == "__main__":
    unittest.main()
