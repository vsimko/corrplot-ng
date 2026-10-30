import importlib.util
import unittest
from pathlib import Path

path = Path(__file__).with_name("run_scalability.py")
spec = importlib.util.spec_from_file_location("run_scalability", path)
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class ScalabilityTests(unittest.TestCase):
    def test_regular_profile(self):
        cases = benchmark.make_workloads()
        self.assertEqual(set(cases), {"B01", "B02", "B05", "B06"})
        for config in cases.values():
            benchmark.validate(config)

    def test_stress_is_opt_in(self):
        cases = benchmark.make_workloads(stress=True)
        self.assertEqual(len(cases["B03"]["labels"]), 500)
        self.assertEqual(len(cases["B04"]["labels"]), 1000)

    def test_missing_values_and_long_labels(self):
        cases = benchmark.make_workloads()
        self.assertTrue(any(v is None for row in cases["B05"]["matrix"] for v in row))
        self.assertGreater(max(map(len, cases["B06"]["labels"])), 24)

    def test_asymmetric_matrix_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "not symmetric"):
            benchmark.validate({"labels": ["a", "b"],
                                "matrix": [[1.0, 0.2], [0.3, 1.0]]})


if __name__ == "__main__":
    unittest.main()
