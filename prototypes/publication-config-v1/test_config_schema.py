"""Validate publication configuration examples and semantic invariants."""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from config import canonical_json, to_scene_config, validate_config
from ordering import resolve_ordering


HERE = Path(__file__).parent
SCHEMA = json.loads((HERE / "schema.json").read_text(encoding="utf-8"))
EXAMPLES = sorted((HERE / "examples").glob("*.json"))


class PublicationConfigTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        Draft202012Validator.check_schema(SCHEMA)

    def test_all_examples_are_valid_and_normalizable(self):
        self.assertEqual(len(EXAMPLES), 5)
        for path in EXAMPLES:
            with self.subTest(example=path.name):
                config = json.loads(path.read_text(encoding="utf-8"))
                validate_config(config)

    def test_schema_rejects_unknown_fields_and_versions(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["plot"]["renderer_magic"] = True
        self.assertTrue(list(Draft202012Validator(SCHEMA).iter_errors(config)))
        config["plot"].pop("renderer_magic")
        config["config_version"] = 2
        self.assertTrue(list(Draft202012Validator(SCHEMA).iter_errors(config)))

    def test_rejects_rectangular_matrix(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0].pop()
        with self.assertRaisesRegex(ValueError, "square"):
            validate_config(config)

    def test_rejects_asymmetric_matrix(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0][1] = 0.2
        with self.assertRaisesRegex(ValueError, "symmetric"):
            validate_config(config)

    def test_rejects_invalid_correlation_diagonal(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0][0] = 0.8
        with self.assertRaisesRegex(ValueError, "diagonal must be 1"):
            validate_config(config)

    def test_rejects_duplicate_labels(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["labels"][1] = config["input"]["labels"][0]
        with self.assertRaisesRegex(ValueError, "unique"):
            validate_config(config)

    def test_rejects_significance_matrix_with_wrong_dimensions(self):
        config = json.loads((HERE / "examples" / "significance.json").read_text(encoding="utf-8"))
        config["plot"]["significance"]["p_values"].pop()
        with self.assertRaisesRegex(ValueError, "p_values.*square"):
            validate_config(config)

    def test_continuous_scale_is_validated_and_adapted_to_scene(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["colors"]["scale"] = {
            "type": "continuous",
            "interpolation": "rgb",
            "stops": [
                {"value": -1, "color": "#0000ff"},
                {"value": 0, "color": "#ffffff"},
                {"value": 1, "color": "#ff0000"},
            ],
        }
        scene_config = to_scene_config(config)
        self.assertEqual(len(scene_config["cell_colors"]), 9)
        self.assertEqual(scene_config["legend"][1]["label"], "0")
        self.assertEqual(scene_config["cell_colors"][0], "#ff0000")
        self.assertEqual(scene_config["legend"][-1]["label"], "Missing")

    def test_discrete_scale_has_explicit_boundary_labels(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["colors"]["scale"] = {
            "type": "discrete",
            "breaks": [-1, -0.25, 0, 0.25, 1],
            "colors": ["#0000ff", "#6666ff", "#ff6666", "#ff0000"],
        }
        scene_config = to_scene_config(config)
        self.assertEqual(
            [entry["label"] for entry in scene_config["legend"]],
            ["[-1, -0.25)", "[-0.25, 0)", "[0, +0.25)", "[+0.25, +1]", "Missing"],
        )

    def test_invalid_scale_definitions_fail_validation(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["colors"]["scale"] = {
            "type": "continuous",
            "stops": [
                {"value": -1, "color": "#0000ff"},
                {"value": 0.5, "color": "#ffffff"},
                {"value": 1, "color": "#ff0000"},
            ],
        }
        with self.assertRaisesRegex(ValueError, "zero"):
            validate_config(config)


    def test_original_and_alphabetical_order_preserve_original_indices(self):
        labels = ["beta", "Alpha", "alpha", "gamma"]
        matrix = [[1.0 if i == j else 0.0 for j in range(4)] for i in range(4)]
        self.assertEqual(resolve_ordering(labels, matrix, "original"), [0, 1, 2, 3])
        self.assertEqual(resolve_ordering(labels, matrix, "alphabetical"), [1, 2, 0, 3])

    def test_hclust_has_deterministic_expected_permutation(self):
        labels = ["a", "b", "c", "d"]
        matrix = [
            [1.0, 0.9, -0.8, -0.7],
            [0.9, 1.0, -0.75, -0.65],
            [-0.8, -0.75, 1.0, 0.92],
            [-0.7, -0.65, 0.92, 1.0],
        ]
        self.assertEqual(resolve_ordering(labels, matrix, "hclust"), [0, 1, 2, 3])

    def test_hclust_uses_pairwise_observed_correlations_and_stable_ties(self):
        labels = ["a", "b", "c"]
        matrix = [
            [1.0, 0.5, None],
            [0.5, 1.0, 0.5],
            [None, 0.5, 1.0],
        ]
        self.assertEqual(resolve_ordering(labels, matrix, "hclust"), [0, 1, 2])

    def test_hclust_rejects_constant_or_unconnected_variables(self):
        labels = ["observed", "constant", "other"]
        matrix = [
            [1.0, None, 0.2],
            [None, 1.0, None],
            [0.2, None, 1.0],
        ]
        with self.assertRaisesRegex(ValueError, "without observed off-diagonal"):
            resolve_ordering(labels, matrix, "hclust")

    def test_aoe_is_rejected_with_actionable_diagnostic(self):
        with self.assertRaisesRegex(ValueError, "aoe.*not supported"):
            resolve_ordering(["a"], [[1.0]], "aoe")

    def test_scene_adapter_reorders_matrix_and_significance_metadata(self):
        config = json.loads((HERE / "examples" / "significance.json").read_text(encoding="utf-8"))
        config["input"]["labels"] = ["C", "A", "B"]
        config["plot"]["ordering"] = "alphabetical"
        scene = to_scene_config(config)
        permutation = scene["variable_order"]["indices"]
        original = config["input"]["values"]
        p_values = config["plot"]["significance"]["p_values"]
        self.assertEqual(scene["matrix"], [[original[i][j] for j in permutation] for i in permutation])
        self.assertEqual(scene["p_values"], [[p_values[i][j] for j in permutation] for i in permutation])

    def test_general_numeric_validation_and_scene_mapping(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"]["values"] = [[10.0, 12.0, 11.0], [8.0, 9.0, 12.0], [11.0, 12.0, -8.0]]
        scene = to_scene_config(config)
        self.assertEqual(scene["matrix_mode"], "general_numeric")
        self.assertEqual(scene["matrix_semantics"]["color_mapping"], "diverging")
        self.assertEqual(scene["matrix_semantics"]["palette"], "RdBu")
        self.assertEqual(scene["matrix_semantics"]["color_limits"], [-8.0, 12.0])
        self.assertEqual(scene["matrix"][0][0], 10.0)
        self.assertEqual(len(scene["cell_colors"]), 9)

    def test_general_numeric_one_sided_and_explicit_limits(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"]["values"] = [[2.0, 3.0, 4.0], [3.0, 2.0, 4.0], [4.0, 4.0, 2.0]]
        config["matrix"]["col_limits"] = [1.0, 5.0]
        scene = to_scene_config(config)
        self.assertEqual(scene["matrix_semantics"]["color_mapping"], "sequential")
        self.assertEqual(scene["matrix_semantics"]["color_limits"], [1.0, 5.0])
        self.assertEqual(scene["matrix"][0][0], 2.0)

    def test_general_numeric_csv_requires_explicit_limits(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"] = {
            "kind": "matrix_csv", "path": "values.csv", "delimiter": ",",
            "header": True, "row_labels": True,
        }
        config["matrix"]["col_limits"] = [-10.0, 10.0]
        validate_config(config)
        config["matrix"].pop("col_limits")
        with self.assertRaisesRegex(ValueError, "CSV-backed general_numeric requires explicit limits"):
            validate_config(config)

    def test_general_numeric_selects_legacy_sequential_palette_for_positive_and_negative(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"]["values"] = [[1.0, 2.0, 2.0], [2.0, 1.0, 3.0], [2.0, 3.0, 3.0]]
        positive = to_scene_config(config)
        self.assertEqual(positive["matrix_semantics"]["palette"], "YlOrBr")
        self.assertEqual(positive["cell_colors"][0], "#ffffe5")
        self.assertEqual(positive["cell_colors"][-1], "#662506")
        config["input"]["values"] = [[-3.0, -2.0, -2.0], [-2.0, -3.0, -1.0], [-2.0, -1.0, -1.0]]
        negative = to_scene_config(config)
        self.assertEqual(negative["matrix_semantics"]["palette"], "YlOrBr")
        self.assertEqual(negative["cell_colors"][0], "#ffffe5")
        self.assertEqual(negative["cell_colors"][-1], "#662506")

    def test_general_numeric_negative_only_and_constant_domains(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"]["values"] = [[-4.0, -3.0, -2.0], [-3.0, -4.0, -2.0], [-2.0, -2.0, -4.0]]
        scene = to_scene_config(config)
        self.assertEqual(scene["matrix_semantics"]["color_mapping"], "sequential")
        self.assertEqual(scene["matrix_semantics"]["normalization"]["intercept"], 4.0)
        self.assertEqual(scene["matrix_semantics"]["normalization"]["zoom"], 0.5)
        self.assertEqual(scene["matrix_semantics"]["palette"], "YlOrBr")
        config["input"]["values"] = [[5.0, 5.0, 5.0], [5.0, 5.0, 5.0], [5.0, 5.0, 5.0]]
        scene = to_scene_config(config)
        self.assertEqual(scene["matrix_semantics"]["color_limits"], [4.5, 5.5])

    def test_general_numeric_rejects_all_missing_and_uncovered_limits(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["matrix"]["kind"] = "general_numeric"
        config["input"]["values"] = [[None, None, None], [None, None, None], [None, None, None]]
        with self.assertRaisesRegex(ValueError, "all-missing"):
            validate_config(config)
        config["input"]["values"] = [[2.0, 3.0, 4.0], [3.0, 2.0, 4.0], [4.0, 4.0, 2.0]]
        config["matrix"]["col_limits"] = [0.0, 3.0]
        with self.assertRaisesRegex(ValueError, "cover every observed"):
            validate_config(config)

    def test_schema_keeps_correlation_range_but_allows_general_values(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        config["input"]["values"][0][1] = 1.2
        config["input"]["values"][1][0] = 1.2
        self.assertTrue(list(Draft202012Validator(SCHEMA).iter_errors(config)))
        config["matrix"]["kind"] = "general_numeric"
        self.assertFalse(list(Draft202012Validator(SCHEMA).iter_errors(config)))

    def test_canonical_json_is_deterministic(self):
        config = json.loads(EXAMPLES[0].read_text(encoding="utf-8"))
        self.assertEqual(canonical_json(config), canonical_json(config))
        self.assertEqual(json.loads(canonical_json(config)), config)


if __name__ == "__main__":
    unittest.main()
