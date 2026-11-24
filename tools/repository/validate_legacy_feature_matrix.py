#!/usr/bin/env python3
"""Validate the evidence-backed corrplot 0.95 feature matrix."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SUPPORT = {"verified", "unsupported", "unknown", "not_applicable"}
CONTRACT = {"supported", "contract-extension", "r-adapter", "deferred", "undocumented", "not-applicable"}


def derive_classification(legacy: str, rust: str, parity_verified: bool = False) -> str | None:
    """Derive the four-way support class; unknown/N/A is deliberately unclassified."""
    if legacy == "verified" and rust == "verified":
        return "both" if parity_verified else None
    return {
        ("verified", "unsupported"): "legacy-only",
        ("unsupported", "verified"): "rust-only",
        ("unsupported", "unsupported"): "neither",
    }.get((legacy, rust))


def _read_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def validate_document(document: dict[str, Any], repo_root: Path) -> list[str]:
    errors: list[str] = []
    reference = document.get("reference", {})
    inventory_path = repo_root / reference.get("inventory_path", "")
    visual_path = repo_root / reference.get("visual_fixture_path", "")

    if document.get("schema_version") != 1:
        errors.append("schema_version must be 1")
    if reference.get("commit") != "62532f700e6c9d04b93ffbf0bf56e264b0e0368e":
        errors.append("reference commit must remain pinned to corrplot 0.95")
    if not inventory_path.is_file():
        errors.append(f"missing inventory: {reference.get('inventory_path')}")
        inventory = {}
    else:
        inventory = _read_json(inventory_path)
    if not visual_path.is_file():
        errors.append(f"missing visual fixture manifest: {reference.get('visual_fixture_path')}")
        visual_ids: set[str] = set()
    else:
        visual = _read_json(visual_path)
        visual_ids = {case["id"] for case in visual.get("cases", [])}

    functions = {
        item["name"]: set(item.get("formals", []))
        for item in inventory.get("functions", [])
    }
    if inventory.get("reference", {}).get("ref") != reference.get("commit"):
        errors.append("matrix and inventory refer to different upstream commits")
    for key in ("adr_path", "scene_v1_contract_path", "visualization_model_contract_path", "rust_contract_path"):
        contract_path = repo_root / reference.get(key, "")
        if not contract_path.is_file():
            errors.append(f"missing architecture/contract reference: {reference.get(key)}")

    rows = document.get("rows", [])
    if not rows:
        errors.append("rows must be a non-empty array")
    seen: set[str] = set()
    for index, row in enumerate(rows):
        prefix = f"rows[{index}]"
        feature_id = row.get("feature_id")
        if not isinstance(feature_id, str) or not feature_id:
            errors.append(f"{prefix}: feature_id must be a non-empty string")
            feature_id = f"<row-{index}>"
        if feature_id in seen:
            errors.append(f"{prefix}: duplicate feature_id {feature_id}")
        seen.add(feature_id)

        legacy = row.get("legacy_support")
        rust = row.get("rust_support")
        if legacy not in SUPPORT:
            errors.append(f"{prefix}: invalid legacy_support {legacy!r}")
        if rust not in SUPPORT:
            errors.append(f"{prefix}: invalid rust_support {rust!r}")
        parity_status = row.get("parity_status")
        parity_evidence = row.get("parity_evidence", [])
        comparative_evidence = any(item.get("kind") == "comparative_test" for item in parity_evidence)
        expected = (
            derive_classification(legacy, rust, parity_status == "verified" and comparative_evidence)
            if legacy in SUPPORT and rust in SUPPORT else None
        )
        if row.get("classification") != expected:
            errors.append(
                f"{prefix}: classification must be derived as {expected!r}, "
                f"not {row.get('classification')!r}"
            )

        contract = row.get("scene_v1_contract_status")
        if contract not in CONTRACT:
            errors.append(f"{prefix}: invalid scene_v1_contract_status {contract!r}")
        if not row.get("behavior") or not row.get("layer") or not row.get("disposition"):
            errors.append(f"{prefix}: behavior, layer and disposition are required")
        if not row.get("notes"):
            errors.append(f"{prefix}: notes must explain scope or limitations")

        refs = row.get("inventory_refs", [])
        for ref in refs:
            function = ref.get("function")
            if function not in functions:
                errors.append(f"{prefix}: unknown inventory function {function!r}")
                continue
            for argument in ref.get("arguments", []):
                if argument not in functions[function]:
                    errors.append(
                        f"{prefix}: {function} has no pinned formal argument {argument!r}"
                    )

        evidence = row.get("evidence", {})
        for side, status in (("legacy", legacy), ("rust", rust)):
            items = evidence.get(side, [])
            if status == "verified" and not items:
                errors.append(f"{prefix}: {side}=verified requires executable test evidence")
            if status == "unsupported" and not any(
                item.get("kind") == "test" and any(
                    word in item.get("test", "").lower()
                    for word in ("reject", "unsupported", "not support", "error")
                )
                for item in items
            ):
                errors.append(f"{prefix}: {side}=unsupported requires an executable negative test")
            if status in {"unknown", "not_applicable"} and items:
                errors.append(f"{prefix}: {side}={status} must not carry support evidence")
            for item in items:
                if item.get("kind") != "test":
                    errors.append(f"{prefix}: support evidence must be an executable test")
                    continue
                test_path = repo_root / item.get("path", "")
                test_name = item.get("test", "")
                if not test_path.is_file():
                    errors.append(f"{prefix}: missing evidence test file {item.get('path')}")
                elif not test_name or test_name not in test_path.read_text(encoding="utf-8"):
                    errors.append(
                        f"{prefix}: test identifier {test_name!r} not found in {item.get('path')}"
                    )
                fixture_ids = item.get("fixture_ids", [])
                fixture_path = item.get("fixture_path")
                if fixture_ids and fixture_path != reference.get("visual_fixture_path"):
                    errors.append(f"{prefix}: fixture IDs must point to the pinned visual fixture manifest")
                for fixture_id in fixture_ids:
                    if fixture_id not in visual_ids:
                        errors.append(f"{prefix}: unknown legacy visual fixture ID {fixture_id!r}")

        if parity_status == "verified":
            if legacy != "verified" or rust != "verified":
                errors.append(f"{prefix}: parity can only be verified when both implementations are verified")
            comparative_items = [item for item in parity_evidence if item.get("kind") == "comparative_test"]
            if not comparative_items:
                errors.append(f"{prefix}: verified parity requires comparative_test evidence")
            for item in comparative_items:
                test_path = repo_root / item.get("path", "")
                test_name = item.get("test", "")
                if not test_path.is_file():
                    errors.append(f"{prefix}: missing comparative evidence test file {item.get('path')}")
                elif not test_name or test_name not in test_path.read_text(encoding="utf-8"):
                    errors.append(f"{prefix}: comparative test identifier {test_name!r} not found in {item.get('path')}")
        elif parity_status == "not-claimed":
            if parity_evidence:
                errors.append(f"{prefix}: not-claimed parity must not carry parity evidence")
        else:
            errors.append(f"{prefix}: parity_status must be 'verified' or 'not-claimed'")

    row_by_id = {row.get("feature_id"): row for row in rows}
    requirements = document.get("scene_v1_requirement_coverage", [])
    if not requirements:
        errors.append("scene_v1_requirement_coverage must be a non-empty array")
    requirement_ids: set[str] = set()
    for index, requirement in enumerate(requirements):
        prefix = f"scene_v1_requirement_coverage[{index}]"
        requirement_id = requirement.get("id")
        if not requirement_id or requirement_id in requirement_ids:
            errors.append(f"{prefix}: requirement id must be non-empty and unique")
        requirement_ids.add(requirement_id)
        status = requirement.get("status")
        if status not in {"verified", "gap", "not-in-contract"}:
            errors.append(f"{prefix}: invalid coverage status {status!r}")
        feature_ids = requirement.get("feature_ids", [])
        if not feature_ids:
            errors.append(f"{prefix}: at least one feature_id is required")
        linked = []
        for feature_id in feature_ids:
            row = row_by_id.get(feature_id)
            if row is None:
                errors.append(f"{prefix}: unknown feature row {feature_id!r}")
            else:
                linked.append(row)
        if status == "verified" and linked and not all(
            row.get("rust_support") in {"verified", "unsupported"}
            and row.get("evidence", {}).get("rust")
            for row in linked
        ):
            errors.append(f"{prefix}: verified scene-v1 requirement needs executable Rust conformance evidence")
        if status == "gap" and linked and not any(
            row.get("rust_support") in {"unknown", "not_applicable"}
            for row in linked
        ):
            errors.append(f"{prefix}: gap status requires at least one missing Rust evidence row")
        if status == "not-in-contract" and linked and not all(
            row.get("scene_v1_contract_status") in {"r-adapter", "deferred", "not-applicable"}
            for row in linked
        ):
            errors.append(f"{prefix}: not-in-contract rows must be marked as adapter/deferred/out-of-contract")

    return errors


def validate_file(matrix_path: Path, repo_root: Path) -> tuple[list[str], int]:
    document = _read_json(matrix_path)
    return validate_document(document, repo_root), len(document.get("rows", []))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--matrix",
        type=Path,
        default=Path("r/corrplotng/tests/testthat/fixtures/legacy-api-feature-matrix.json"),
    )
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    args = parser.parse_args()
    errors, count = validate_file(args.matrix, args.repo_root)
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        print(f"Feature matrix validation failed: {len(errors)} error(s), {count} row(s).")
        return 1
    print(f"Feature matrix validation passed: {count} rows; pinned source and executable evidence verified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
