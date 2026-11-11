# Repository root policy

This policy implements the root-directory rules in [ADR 0003](adr/0003-monorepo-layout.md). The root is for repository-wide governance, documentation entry points, shared editor/ignore settings and thin orchestration. Ecosystem dependency graphs and generated environments belong to their owning project roots.

## Permanent root allowlist

- Governance and discovery files: `README.md`, `CITATION.cff`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, and `LICENSE` where present.
- Git/editor configuration: `.gitignore`, `.gitattributes`, and shared editor configuration.
- `.github/` for workflows and repository automation.
- `docs/` for ADRs and repository-wide documentation.
- Target ecosystem roots defined by ADR 0003: `contracts/`, `fixtures/`, `rust/`, `npm/`, `r/`, `python/`, and `packaging/`.
- `experiments/` for isolated, non-production research and cross-runtime proofs with independent build/test boundaries, as explicitly allowed by ADR 0003.
- `tools/` only for repository-wide orchestration under `tools/repository/`; ecosystem executables belong with their ecosystem.

## Transitional root entries

None. The npm workspace, lockfile, Changesets metadata, and JavaScript packages have moved under `npm/` in #367. The `experiments/` directory is a permanent ADR-approved root exception. The checker fails on every unreviewed root entry and on new root dependency manifests.

## Generated paths that must never be committed

The checker rejects tracked occurrences of:

- JavaScript dependencies: `node_modules/`.
- Rust build output: `target/`.
- Python environments/caches: `.venv/`, `venv/`, `__pycache__/`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, `*.pyc`, `*.pyo`.
- Local R state: `.Rproj.user/`, `Rlibs/`, `renv/library/`, `renv/staging/`, and generated R package directories.
- Built native Node addons (`*.node`) and generated browser-WASM package output (`npm/packages/wasm/pkg/`).

Build artifacts and caches belong in CI artifacts or local ignored directories, never in Git.

## Check

Run from a clean checkout:

```sh
python3 tools/repository/check_root_policy.py
python3 -m unittest discover -s tools/repository/tests -p 'test_*.py' -v
```

The foundation CI job runs both commands. When a migration removes a transitional root entry, remove its exception from `tools/repository/check_root_policy.py` and this table in the same PR.
