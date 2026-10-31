"""Renderer-neutral cluster-region semantics and geometry tests."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from renderer import render_scene_svg
from scene import build_scene


HERE = Path(__file__).parent
SCHEMA = json.loads((HERE / "scene-v1.schema.json").read_text(encoding="utf-8"))


def base_config():
    return {
        "labels": ["alpha", "beta", "gamma"],
        "matrix": [
            [1.0, 0.4, 0.1],
            [0.4, 1.0, 0.3],
            [0.1, 0.3, 1.0],
        ],
    }


class ClusterRegionTests(unittest.TestCase):
    def test_name_based_region_resolves_to_inclusive_cell_boundaries(self):
        config = base_config()
        config["cluster_regions"] = [{
            "id": "cluster-ab",
            "variables": ["alpha", "beta"],
            "color": "#123456",
            "line_width": 2.0,
        }]
        scene = build_scene(config)
        region = scene["cluster_regions"][0]
        self.assertEqual(region["variables"], ["alpha", "beta"])
        self.assertEqual(region["source_indices"], [0, 1])
        self.assertEqual(
            (region["row_start"], region["row_end"], region["column_start"], region["column_end"]),
            (0, 1, 0, 1),
        )
        self.assertEqual(region["width"], 2 * scene["cells"][0]["size"])
        self.assertEqual(list(Draft202012Validator(SCHEMA).iter_errors(scene)), [])
        svg = render_scene_svg(scene)
        self.assertIn('class="cluster-region"', svg)
        self.assertIn('data-region-id="cluster-ab"', svg)
        self.assertIn('stroke="#123456"', svg)

    def test_source_indices_follow_deterministic_reordered_leaf_order(self):
        config = base_config()
        config["labels"] = ["gamma", "alpha", "beta"]
        config["matrix"] = [
            [1.0, 0.1, 0.3],
            [0.1, 1.0, 0.4],
            [0.3, 0.4, 1.0],
        ]
        config["variable_order"] = {
            "strategy": "hclust",
            "indices": [2, 0, 1],
            "labels": ["gamma", "alpha", "beta"],
        }
        config["cluster_regions"] = [{
            "id": "cluster-ab",
            "indices": [0, 1],
        }]
        scene = build_scene(config)
        region = scene["cluster_regions"][0]
        self.assertEqual(region["variables"], ["alpha", "beta"])
        self.assertEqual(region["source_indices"], [0, 1])
        self.assertEqual((region["row_start"], region["row_end"]), (1, 2))

    def test_name_based_region_maps_through_reordered_labels(self):
        config = base_config()
        config["labels"] = ["gamma", "alpha", "beta"]
        config["matrix"] = [
            [1.0, 0.1, 0.3],
            [0.1, 1.0, 0.4],
            [0.3, 0.4, 1.0],
        ]
        config["variable_order"] = {
            "strategy": "hclust",
            "indices": [2, 0, 1],
            "labels": ["gamma", "alpha", "beta"],
        }
        config["cluster_regions"] = [{"id": "ab", "variables": ["beta", "alpha"]}]
        region = build_scene(config)["cluster_regions"][0]
        self.assertEqual(region["variables"], ["alpha", "beta"])
        self.assertEqual((region["row_start"], region["row_end"]), (1, 2))

    def test_rectangle_geometry_is_stable_for_display_layouts(self):
        geometries = []
        for display_type in ("full", "upper", "lower", "diagonal"):
            config = base_config()
            config["display_type"] = display_type
            config["cluster_regions"] = [{"id": "ab", "variables": ["alpha", "beta"]}]
            scene = build_scene(config)
            region = scene["cluster_regions"][0]
            geometries.append((region["x"], region["y"], region["width"], region["height"]))
        self.assertTrue(all(geometry == geometries[0] for geometry in geometries))

    def test_non_contiguous_unknown_and_ambiguous_regions_are_rejected(self):
        config = base_config()
        config["cluster_regions"] = [{"id": "ac", "variables": ["alpha", "gamma"]}]
        with self.assertRaisesRegex(ValueError, "contiguous"):
            build_scene(config)
        config["cluster_regions"] = [{"id": "unknown", "variables": ["missing"]}]
        with self.assertRaisesRegex(ValueError, "unknown variables"):
            build_scene(config)
        config["cluster_regions"] = [{"id": "ambiguous", "variables": ["alpha"], "indices": [0]}]
        with self.assertRaisesRegex(ValueError, "exactly one"):
            build_scene(config)

    def test_invalid_region_style_and_duplicate_ids_are_rejected(self):
        config = base_config()
        config["cluster_regions"] = [
            {"id": "same", "variables": ["alpha"], "color": "red"},
        ]
        with self.assertRaisesRegex(ValueError, "six-digit hex"):
            build_scene(config)
        config["cluster_regions"] = [
            {"id": "same", "variables": ["alpha"]},
            {"id": "same", "variables": ["beta"]},
        ]
        with self.assertRaisesRegex(ValueError, "duplicate id"):
            build_scene(config)


if __name__ == "__main__":
    unittest.main()
