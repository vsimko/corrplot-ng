# Repository layout and ownership

This document defines current ownership and the accepted ADR 0003 boundaries. It distinguishes production ecosystem roots from retained non-production experiments and transitional paths.

## Top-level directories

| Path | Ownership and contents |
| --- | --- |
| `rust/` | Rust workspace root for the shared core, CLI, Node-API and browser-WASM crates. The disposable R/rextendr adapter is kept under `rust/bindings/r/` but excluded from the workspace with its own audited lockfile. |
| `npm/` | Private npm workspace, lockfile, installable packages, platform-specific native package manifests, and cross-backend npm tests. Rust crate sources live under `rust/bindings/`. |
| `tools/` | Repository-wide orchestration only, under `tools/repository/`. |
| `r/` | Production-facing R package at `r/corrplotng/`; owns R APIs, legacy compatibility, graphics-device behavior, and package tests. |
| `python/` | Python reference project under `python/reference/`; non-production and not a distributable package. |
| `experiments/` | Disposable cross-runtime experiments; non-production and not supported package backends. |
| `prototypes/` | Retained Python reference implementations and validators for normalized-matrix and publication configuration/SVG prototypes. Normative schemas live under `contracts/`; canonical shared fixtures live under `fixtures/`. |
| `tests/` | Cross-language fixtures, compatibility oracles, and repository-level tests that are not owned by one runtime package. |
| `scripts/` | Maintainer and CI utilities that operate across packages or validate generated artifacts. |
| `benchmarks/` | Opt-in performance and resource measurements. Benchmarks are not part of routine correctness checks unless a specific issue requires them. |
| `docs/` | Architecture decisions, contracts and workflow documentation, testing guidance, and the roadmap. |
| `.github/` | GitHub Actions workflows, issue templates, and repository automation. |

Root-level manifests and configuration files remain at the repository root when their tools expect them there.

## Retained Python prototypes under `prototypes/`

The remaining `prototypes/` tree contains executable Python reference code and tests, not canonical contract definitions:

- `normalized-matrix-v1/` contains the Python semantic validator and its tests. Its normative schema is `contracts/normalized-matrix-v1/`; canonical valid/invalid fixtures are `fixtures/normalized-matrix-v1/`.
- `publication-config-v1/` contains the Python publication-configuration prototype, examples, and tests. Its normative schema and specification are under `contracts/publication-config-v1/`.
- `publication-svg-v1/` contains the Python publication-SVG prototype, examples, and tests. Its normative schema and specification are under `contracts/publication-svg-v1/`.

The Python scene/SVG reference is separately owned by `python/reference/`. Do not treat prototype implementation code as the source of truth for versioned schemas or shared fixtures.

## Rust package boundary

- `rust/crates/corrplot-core/` owns reusable, language-neutral Rust semantics. It currently provides shared matrix validation, a parser for the matrix-bearing scene-v1 subset, normalized-matrix-to-scene construction, and typed-scene SVG/RGBA rendering. Full scene-v1 presentation semantics remain incomplete.
- `rust/tools/corrplot-cli/` owns CLI parsing, file and process I/O, command orchestration, and executable packaging.
- `rust/bindings/r/rextendr-prototype/` owns only the disposable R-binding proof and its generated package workflow; it is intentionally excluded from the main Rust workspace to preserve its separately audited CRAN/offline lockfile.
- `r/corrplotng/` owns the production R-facing API and legacy compatibility behavior. Do not migrate it to rextendr as part of a layout cleanup.
- `rust/bindings/node/` is the experimental Node-API adapter. It exposes matrix validation, normalized-matrix scene construction, typed-scene SVG/RGBA rendering, and matrix-bearing scene-v1 compatibility rendering without duplicating semantics. `rust/bindings/wasm/` exposes the corresponding browser-WASM operations over the same core. The runtime-isolated facade in `npm/packages/typescript/` selects the appropriate backend. The optional React adapter in `npm/packages/react/` and Vue adapter in `npm/packages/vue/` consume only the browser WASM package and keep their frameworks as peer dependencies. Neither runtime binding or framework adapter may copy core algorithms.

Rust is the accepted strategic shared-core implementation in ADR 0001. The scene-v1 JSON contract remains the language-neutral interchange and conformance format, not a competing implementation of the shared semantics. Production R adoption remains gated by CRAN disposition for the retained Cargo vendor-metadata NOTE, compatibility with the intended CRAN toolchain, and evidence from the actual production package artifact. The C++/Rcpp maintainer-effort comparison was discontinued by #269.

## Shared fixture ownership

The canonical sources and package copies are:

| Canonical source | Package copy | Policy |
| --- | --- | --- |
| `fixtures/normalized-matrix-v1/valid-cases.json` | `r/corrplotng/inst/extdata/normalized-matrix-v1/valid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `fixtures/normalized-matrix-v1/invalid-cases.json` | `r/corrplotng/inst/extdata/normalized-matrix-v1/invalid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `tests/legacy-visual/cases.json` | `r/corrplotng/tests/testthat/fixtures/legacy-visual-cases.json` | Exact byte-for-byte package-test copy. |
| `fixtures/matrix-input-v1/correlation-cases.json` | `r/corrplotng/inst/extdata/correlation-cases.json` | Exact byte-for-byte copy; required by installed-package scene-adapter tests. |
| `fixtures/matrix-input-v1/general-matrix-cases.json` | `r/corrplotng/inst/extdata/general-matrix-cases.json` | The package copy is a declared projection: it omits the canonical `all-missing` invalid render case. Keep all other content identical. |

`tests/test_fixture_integrity.py` checks the exact copies and the declared general-matrix projection. Update that test and this table together if the ownership or projection changes. Do not edit package copies independently.

The renderer capability declarations are intentionally backend-specific, not duplicate fixtures: the Python reference declaration is in `python/reference/renderer-capabilities.json`, while the Rust CLI declaration is in `rust/tools/corrplot-cli/renderer-capabilities.json`. They must be validated against the shared schema, but their capability values are expected to differ.

## Change rules

1. Put shared semantic behavior in `rust/crates/corrplot-core`, not in CLI or binding adapters.
2. Keep R-specific API compatibility and graphics-device side effects in `r/corrplotng`.
3. Put cross-runtime conformance fixtures in `tests/` or alongside the normative contract they define. A package copy is allowed only when packaging or package-local tests require it.
4. Give every package copy an explicit source-of-truth and an automated drift check.
5. Keep reference code and disposable spikes clearly labeled; do not present them as production backends.
6. Before moving files, inventory references and update workflows, manifests, scripts, tests, documentation, and examples in the same staged change.
