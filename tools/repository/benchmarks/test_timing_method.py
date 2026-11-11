import importlib.util
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parent / "run_python_scene_v1_reference.py"
SPEC = importlib.util.spec_from_file_location("run_python_scene_v1_reference", MODULE_PATH)
benchmark = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(benchmark)


class CalibratedTimingTests(unittest.TestCase):
    def test_measurement_uses_calibrated_batches_and_raw_samples(self):
        calls = 0

        def operation():
            nonlocal calls
            calls += 1

        result = benchmark.measure(operation)
        self.assertGreaterEqual(result["batch_size"], 1)
        self.assertEqual(result["sample_count"], 30)
        self.assertEqual(len(result["samples_ms"]), 30)
        self.assertEqual(result["warmup_iterations"], 5)
        self.assertGreater(calls, result["batch_size"] * result["sample_count"])


if __name__ == "__main__":
    unittest.main()
