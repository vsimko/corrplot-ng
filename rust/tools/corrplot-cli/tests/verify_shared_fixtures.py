#!/usr/bin/env python3
"""Exercise the native CLI with the repository's shared matrix fixtures.

This is an integration test: it checks the installed-style executable rather
than only validating the fixture JSON itself.
"""

import argparse
import csv
import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
FIXTURES = (
    (ROOT / "fixtures" / "matrix-input-v1" / "correlation-cases.json", "correlation", "correlation_plot"),
    (ROOT / "fixtures" / "matrix-input-v1" / "general-matrix-cases.json", "general", "general_matrix_plot"),
)


def run(binary: Path, *arguments: str) -> str:
    result = subprocess.run(
        [str(binary), *arguments],
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if result.returncode != 0:
        raise AssertionError(
            f"{binary.name} {' '.join(arguments)} failed with {result.returncode}: "
            f"{result.stderr.strip()}"
        )
    return result.stdout


def write_csv(path: Path, labels: list[str], matrix: list[list[float | None]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(["", *labels])
        for label, row in zip(labels, matrix, strict=True):
            writer.writerow([label, *("NA" if value is None else repr(value) for value in row)])


def check_case(binary: Path, case: dict, kind: str, scene_kind: str) -> None:
    matrix = case["matrix"]
    size = len(matrix)
    labels = case.get("labels") or [f"variable-{index + 1}" for index in range(size)]
    assert len(labels) == size, f"{case['id']}: labels do not match matrix dimensions"

    with tempfile.TemporaryDirectory(prefix="corrplot-cli-fixture-") as temporary:
        input_path = Path(temporary) / "matrix.csv"
        write_csv(input_path, labels, matrix)

        run(binary, "validate", "--input", str(input_path), "--kind", kind)

        normalized = json.loads(
            run(
                binary,
                "render",
                "--input",
                str(input_path),
                "--kind",
                kind,
                "--format",
                "normalized-json",
            )
        )
        assert normalized["schema_version"] == 1, case["id"]
        assert normalized["kind"] == "normalized_matrix", case["id"]
        assert normalized["labels"] == labels, case["id"]
        assert normalized["values"] == matrix, case["id"]
        assert normalized["source"]["format"] == "csv", case["id"]
        assert normalized["source_indices"] == list(range(size)), case["id"]

        scene = json.loads(
            run(
                binary,
                "render",
                "--input",
                str(input_path),
                "--kind",
                kind,
                "--format",
                "scene-json",
            )
        )
        assert scene["schema_version"] == 1, case["id"]
        assert scene["kind"] == scene_kind, case["id"]
        assert [label["text"] for label in scene["labels"]] == labels, case["id"]
        assert len(scene["cells"]) == size * size, case["id"]

        rendered_values = [[None] * size for _ in range(size)]
        for cell in scene["cells"]:
            rendered_values[cell["row"]][cell["column"]] = cell["value"]
        assert rendered_values == matrix, case["id"]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    arguments = parser.parse_args()
    binary = arguments.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"native CLI binary does not exist: {binary}")

    total = 0
    for fixture_path, kind, scene_kind in FIXTURES:
        with fixture_path.open(encoding="utf-8") as source:
            fixture = json.load(source)
        for case in fixture["cases"]:
            check_case(binary, case, kind, scene_kind)
            total += 1

    print(f"Native CLI passed shared matrix fixture parity for {total} cases.")


if __name__ == "__main__":
    main()
