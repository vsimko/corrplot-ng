import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_root_policy import validate_paths


class RootPolicyTests(unittest.TestCase):
    def test_current_migration_baseline_allows_experiments_and_warns_only_for_benchmarks(self):
        paths = [
            ".gitattributes",
            ".gitignore",
            "CITATION.cff",
            "README.md",
            "docs/architecture.md",
            ".github/workflows/test-foundation.yml",
            "tools/repository/check_root_policy.py",
            "npm/package.json",
            "npm/package-lock.json",
            "npm/packages/native-node/package.json",
            "npm/.changeset/config.json",
            "python/reference/requirements-dev.txt",
            "rust/crates/corrplot-core/Cargo.toml",
            "r/corrplotng/DESCRIPTION",
            "experiments/wasm-r/README.md",
            "benchmarks/README.md",
            "python/reference/prototypes/publication-config-v1/README.md",
        ]
        errors, warnings = validate_paths(paths)
        self.assertEqual(errors, [])
        self.assertFalse(any("package.json" in warning for warning in warnings))
        self.assertFalse(any("requirements-dev.txt" in warning for warning in warnings))
        self.assertFalse(any("experiments/" in warning for warning in warnings))
        self.assertFalse(any("prototypes/" in warning for warning in warnings))
        self.assertEqual(len(warnings), 1)
        self.assertTrue(warnings[0].startswith("benchmarks/"))

    def test_adr_approved_experiments_root_is_allowed(self):
        errors, warnings = validate_paths(["experiments/wasm-r/README.md"])
        self.assertEqual(errors, [])
        self.assertEqual(warnings, [])

    def test_unknown_root_entry_fails(self):
        errors, _ = validate_paths(["README.md", "random-workspace/package.json"])
        self.assertTrue(any("random-workspace" in error for error in errors))

    def test_root_cargo_manifest_fails(self):
        errors, _ = validate_paths(["Cargo.toml"])
        self.assertTrue(any("Cargo.toml" in error for error in errors))

    def test_generated_dependency_directories_fail_at_any_depth(self):
        errors, _ = validate_paths([
            "npm/packages/example/node_modules/example/index.js",
            "rust/crates/example/target/debug/example",
            "python/reference/.venv/bin/python",
            "r/corrplotng/renv/library/R-4.4/example/DESCRIPTION",
        ])
        self.assertEqual(len(errors), 4)

    def test_generated_bytecode_and_native_addons_fail(self):
        errors, _ = validate_paths([
            "python/reference/__pycache__/module.pyc",
            "npm/packages/native-node/prebuilds/addon.node",
        ])
        self.assertEqual(len(errors), 3)

    def test_new_root_directory_fails(self):
        errors, _ = validate_paths(["README.md", "new-product-root/source.rs"])
        self.assertTrue(any("new-product-root" in error for error in errors))


if __name__ == "__main__":
    unittest.main()


class RemovedRootTestsDirectoryTests(unittest.TestCase):
    def test_root_tests_directory_is_no_longer_a_transition_exception(self):
        errors, warnings = validate_paths(["tests/test_fixture_integrity.py"])
        self.assertTrue(any(error.startswith("tests: unreviewed repository-root entry") for error in errors))
        self.assertFalse(any(warning.startswith("tests/") for warning in warnings))
