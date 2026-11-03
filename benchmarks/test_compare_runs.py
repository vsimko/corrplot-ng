import json
import tempfile
import unittest
from pathlib import Path

from compare_runs import SUPPORTED_SUITE, compare_runs


class CompareRunsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.environment = {
            "python": "3.12.2 (main, Feb 1 2026)",
            "implementation": "CPython",
            "platform": "Linux-6.8.0-x86_64-with-glibc2.39",
            "machine": "x86_64",
            "processor": "x86_64",
            "cpu_count": 4,
        }
        self.inputs = {"B01": {"labels": ["a"], "matrix": [[1.0]]}}

    def tearDown(self):
        self.temp.cleanup()

    def write_run(self, name, samples, *, environment=None, inputs=None, seed=20261029):
        payload = {
            "schema_version": 1,
            "suite": SUPPORTED_SUITE,
            "seed": seed,
            "iterations": len(samples),
            "measured_at": "2026-11-03T00:00:00Z",
            "source_commit": name,
            "environment": environment or self.environment,
            "inputs": inputs or self.inputs,
            "results": [{
                "workload": "B01",
                "status": "passed",
                "measurement": "core_only_scene_construction",
                "samples_ms": samples,
            }],
        }
        path = self.root / f"{name}.json"
        path.write_text(json.dumps(payload), encoding="utf-8")
        return path

    def test_compares_compatible_runs_and_preserves_raw_samples(self):
        first = self.write_run("baseline", [10.0, 11.0, 9.0])
        second = self.write_run("latest", [12.0, 13.0, 11.0])
        report = compare_runs([first, second])
        observations = report["comparisons"][0]["observations"]
        self.assertEqual(observations[0]["samples_ms"], [10.0, 11.0, 9.0])
        self.assertEqual(observations[1]["samples_ms"], [12.0, 13.0, 11.0])
        self.assertAlmostEqual(observations[1]["change_vs_first_percent"], 20.0)

    def test_rejects_different_machine_environment(self):
        first = self.write_run("baseline", [10.0])
        second = self.write_run("latest", [11.0], environment=dict(self.environment, machine="arm64"))
        with self.assertRaisesRegex(ValueError, "incompatible benchmark environments"):
            compare_runs([first, second])

    def test_rejects_different_seed(self):
        first = self.write_run("baseline", [10.0])
        second = self.write_run("latest", [11.0], seed=7)
        with self.assertRaisesRegex(ValueError, "generator seed"):
            compare_runs([first, second])

    def test_omits_workload_when_input_fingerprint_changes(self):
        first = self.write_run("baseline", [10.0])
        second = self.write_run(
            "latest", [11.0],
            inputs={"B01": {"labels": ["b"], "matrix": [[1.0]]}},
        )
        report = compare_runs([first, second])
        self.assertEqual(report["comparisons"], [])

    def test_requires_two_runs(self):
        first = self.write_run("baseline", [10.0])
        with self.assertRaisesRegex(ValueError, "at least two"):
            compare_runs([first])


if __name__ == "__main__":
    unittest.main()
