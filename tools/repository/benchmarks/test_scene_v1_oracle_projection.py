"""Tests for the Python-to-native scene-v1 benchmark oracle projection."""
from __future__ import annotations

import unittest

from export_native_fixtures import project_scene_v1


class SceneV1OracleProjectionTests(unittest.TestCase):
    def test_keeps_only_the_shared_native_contract(self) -> None:
        scene = {
            "schema_version": 1,
            "kind": "correlation_plot",
            "width": 296,
            "height": 308,
            "labels": [{
                "text": "a", "row_x": 92, "row_y": 118,
                "column_x": 146, "column_y": 60,
                "renderer_hint": "ignored",
            }],
            "cells": [{
                "row": 0, "column": 0, "value": 1.0,
                "x": 104, "y": 76, "center_x": 146,
                "center_y": 118, "size": 84,
                "encoding": {"kind": "circle", "radius": 36.12},
            }],
            "annotation": "",
            "style": {"positive_color": "#2166ac"},
            "typography": {"label_font_size": 13},
            "title": "newer Python-only field",
        }

        projected = project_scene_v1(scene)

        self.assertEqual(
            set(projected),
            {"schema_version", "kind", "width", "height", "labels",
             "cells", "annotation", "style"},
        )
        self.assertEqual(
            set(projected["labels"][0]),
            {"text", "row_x", "row_y", "column_x", "column_y"},
        )
        self.assertEqual(
            set(projected["cells"][0]),
            {"row", "column", "value", "x", "y", "center_x", "center_y", "size"},
        )
        self.assertEqual(projected["cells"][0]["value"], 1.0)

    def test_preserves_missing_values_as_null(self) -> None:
        scene = {
            "schema_version": 1, "kind": "correlation_plot",
            "width": 296, "height": 308, "labels": [],
            "cells": [{
                "row": 0, "column": 1, "value": None,
                "x": 188, "y": 76, "center_x": 230,
                "center_y": 118, "size": 84,
            }],
            "annotation": "", "style": {},
        }

        self.assertIsNone(project_scene_v1(scene)["cells"][0]["value"])


if __name__ == "__main__":
    unittest.main()
