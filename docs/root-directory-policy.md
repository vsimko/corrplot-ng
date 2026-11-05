# Repository root policy

This policy implements the root-directory rules in [ADR 0003](adr/0003-monorepo-layout.md). The root is for repository-wide governance, documentation entry points, shared editor/ignore settings and thin orchestration. Ecosystem dependency graphs and generated environments belong to their owning project roots.

## Permanent root allowlist

- Governance and discovery files: `README.md`, `CITATION.cff`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, and `LICENSE` where present.
- Git/editor configuration: `.gitignore`, `.gitattributes`, and shared editor configuration.
- `.github/` for workflows and repository automation.
- `docs/` for ADRs and repository-wide documentation.
- Target ecosystem roots defined by ADR 0003: `contracts/`, `fixtures/`, `rust/`, `npm/`, `r/`, `python/`, and `packaging/`.
- `tools/` only for repository-wide orchestration under `tools/repository/`; ecosystem executables belong with their ecosystem.

## Explicit migration exceptions

The following current entries are temporary and must be removed or relocated by the linked work. They are warnings in the checker so the policy can be enforced before the structural moves finish; no new root-level package manager is permitted.

| Current root entry | Reason / destination |
|---|---|
| `package.json` (and future generated `package-lock.json`) | Existing private npm workspace; move both to `npm/` under #367. #265 must generate the lockfile with npm rather than hand-authoring it. |
| `requirements-dev.txt` | Current repository Python test dependency; move under `tools/repository/` or the owning Python project as part of #369/#372. |
| `.changeset/` | npm release metadata; move to `npm/.changeset/` under #367. |
| `benchmarks/` | Mixed-runtime harness; split by ownership under #364. |
| `bindings/`, `crates/` | Mixed npm/Rust roots; split under #366/#367. |
| `implementations/` | Current R package; move to `r/corrplotng/` under #368. |
| `packages/` | Current npm workspace packages; move to `npm/packages/` under #367. |
| `prototypes/` | Mixed contracts, Python reference code and Rust/rextendr experiment; split under #366/#369/#370. |
| `scripts/`, `tests/` | Mixed repository and runtime utilities/fixtures; split under #364/#370. |
| `experiments/` | Cross-runtime WASM/R experiment. ADR 0003's target tree does not yet show this root; #364 records it as a deliberate exception that must be explicitly retained or assigned another reviewed home before migration. |
| `tools/corrplot-ng/` | Current Rust CLI; move to `rust/tools/corrplot-cli/` under #366. The eventual `tools/` root remains for repository orchestration only. |

Do not remove these exceptions until the owning move and its CI/documentation updates have merged. The checker fails on unreviewed root entries and on new root dependency manifests outside the explicit transitional set.

## Generated paths that must never be committed

The checker rejects tracked occurrences of:

- JavaScript dependencies: `node_modules/`.
- Rust build output: `target/`.
- Python environments/caches: `.venv/`, `venv/`, `__pycache__/`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, `*.pyc`, `*.pyo`.
- Local R state: `.Rproj.user/`, `Rlibs/`, `renv/library/`, `renv/staging/`, and generated R package directories.
- Built native Node addons (`*.node`) and generated browser-WASM package output (`bindings/wasm/pkg/` or its future npm location).

Build artifacts and caches belong in CI artifacts or local ignored directories, never in Git.

## Check

Run from a clean checkout:

```sh
python3 tools/repository/check_root_policy.py
python3 -m unittest tools/repository/tests/test_root_policy.py -v
```

The foundation CI job runs both commands. When a migration removes a transitional root entry, remove its exception from `tools/repository/check_root_policy.py` and this table in the same PR.
