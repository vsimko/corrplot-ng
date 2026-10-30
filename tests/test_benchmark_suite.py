import importlib.util
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "benchmarks" / "run_shared_core.py"
SPEC = importlib.util.spec_from_file_location("run_shared_core", MODULE_PATH)
benchmark = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(benchmark)


class BenchmarkSuiteTests(unittest.TestCase):
    def test_workloads_cover_initial_milestone(self):
        cases = benchmark.make_cases()
        self.assertEqual(set(cases), {"B01", "B02", "B05", "B07"})
        self.assertEqual(len(cases["B01"]["labels"]), 20)
        self.assertEqual(len(cases["B02"]["labels"]), 100)
        self.assertEqual(cases["B05"]["matrix"][0][0], 1.0)
        self.assertEqual(cases["B07"]["labels"][0], "v099")

    def test_generated_cases_are_deterministic(self):
        self.assertEqual(benchmark.make_cases(), benchmark.make_cases())

    def test_missingness_is_symmetric_and_approximately_ten_percent(self):
        matrix = benchmark.make_cases()["B05"]["matrix"]
        missing = sum(value is None for row in matrix for value in row)
        self.assertEqual(missing % 2, 0)
        self.assertAlmostEqual(missing / (100 * 99), 0.10, delta=0.002)

    def test_runner_emits_all_initial_workloads(self):
        result = benchmark.run(20)
        self.assertEqual(
            [item["workload"] for item in result["results"]],
            ["B01", "B02", "B05", "B07", "B08"],
        )
        self.assertTrue(all(item["status"] == "passed" for item in result["results"]))
        self.assertTrue(all(len(item["samples_ms"]) == 20 for item in result["results"]))


if __name__ == "__main__":
    unittest.main()
