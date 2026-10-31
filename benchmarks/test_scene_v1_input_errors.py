#!/usr/bin/env python3
"""Check that the Python scene-v1 reference rejects invalid native-spike inputs."""
from __future__ import annotations

import math
import unittest
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "prototypes" / "shared-svg"))
from scene import build_scene


class SceneV1InvalidInputTests(unittest.TestCase):
    def test_rejects_dimension_mismatch(self) -> None:
        with self.assertRaisesRegex(ValueError, "matrix must be square and match labels"):
            build_scene({"labels": ["a", "b"], "matrix": [[1.0, 0.2]]})

    def test_rejects_out_of_range_correlation(self) -> None:
        with self.assertRaisesRegex(ValueError, "correlation values must be null or between -1 and 1"):
            build_scene({"labels": ["a", "b"], "matrix": [[1.0, 1.1], [1.1, 1.0]]})

    def test_rejects_non_finite_values(self) -> None:
        with self.assertRaisesRegex(ValueError, "matrix values must be finite numbers or null"):
            build_scene({"labels": ["a", "b"], "matrix": [[1.0, math.inf], [math.inf, 1.0]]})

    def test_rejects_nan_instead_of_treating_it_as_missing(self) -> None:
        with self.assertRaisesRegex(ValueError, "matrix values must be finite numbers or null"):
            build_scene({"labels": ["a", "b"], "matrix": [[1.0, math.nan], [math.nan, 1.0]]})


if __name__ == "__main__":
    unittest.main()
