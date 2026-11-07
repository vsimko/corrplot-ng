# Repository layout and ownership

This document defines current ownership and the accepted ADR 0003 boundaries. It distinguishes production ecosystem roots from retained non-production experiments and transitional paths.

## Top-level directories

| Path | Ownership and contents |
| --- | --- |
| `rust/` | Rust workspace root for the shared core, CLI, Node-API and browser-WASM crates. The disposable R/rextendr adapter is kept under `rust/bindings/r/` but excluded from the workspace with its own audited lockfile. |
| `npm/` | Private npm workspace, lockfile, installable packages, platform-specific native package manifests, and cross-backend npm tests. Rust crate sources live under `rust/bindings/`. |
| `tools/` | Repository-wide orchestration only, under `tools/repository/`. |
| `implementations/` | Language-specific product packages and adapters. The R package owns R APIs, legacy compatibility, graphics-device behavior, and package-specific tests. |
| `prototypes/` | A transitional home for three distinct asset classes: normative versioned contracts, reference implementations, and disposable experiments. Do not treat every child directory as production code. |
| `tests/` | Cross-language fixtures, compatibility oracles, and repository-level tests that are not owned by one runtime package. |
| `scripts/` | Maintainer and CI utilities that operate across packages or validate generated artifacts. |
| `benchmarks/` | Opt-in performance and resource measurements. Benchmarks are not part of routine correctness checks unless a specific issue requires them. |
| `docs/` | Architecture decisions, contracts and workflow documentation, testing guidance, and the roadmap. |
| `.github/` | GitHub Actions workflows, issue templates, and repository automation. |

Root-level manifests and configuration files remain at the repository root when their tools expect them there.

## Categories currently grouped under `prototypes/`

The remaining transitional paths are retained until their owning migration issues update package manifests, CI filters, and documented commands. The categories have different stability expectations:

- **Normative, versioned contracts:** `normalized-matrix-v1/`, `publication-config-v1/`, `publication-svg-v1/`, the scene-v1 schema and fixtures, and `renderer-capabilities-v1.schema.json`. Their schemas and compatibility rules are normative; their current location under `prototypes/` is historical, not a statement that the contracts are throwaway.
- **Reference implementation:** `shared-svg/` is the Python scene/SVG reference implementation. It is useful for semantic examples and conformance work, but it is not the selected production shared core.

Future work may move these categories to clearer top-level locations such as `contracts/`, `references/`, and `experiments/`. Do not move them until a separate staged change inventories and updates every relative link, package path, CI path filter, fixture consumer, and developer command.

## Rust package boundary

- `rust/crates/corrplot-core/` owns reusable, language-neutral Rust semantics. It currently provides shared matrix validation, a parser for the matrix-bearing scene-v1 subset, normalized-matrix-to-scene construction, and typed-scene SVG/RGBA rendering. Full scene-v1 presentation semantics remain incomplete.
- `rust/tools/corrplot-cli/` owns CLI parsing, file and process I/O, command orchestration, and executable packaging.
- `rust/bindings/r/rextendr-prototype/` owns only the disposable R-binding proof and its generated package workflow; it is intentionally excluded from the main Rust workspace to preserve its separately audited CRAN/offline lockfile.
- `implementations/r/` owns the production R-facing API and legacy compatibility behavior. Do not migrate it to rextendr as part of a layout cleanup.
- `rust/bindings/node/` is the experimental Node-API adapter. It exposes matrix validation, normalized-matrix scene construction, typed-scene SVG/RGBA rendering, and matrix-bearing scene-v1 compatibility rendering without duplicating semantics. `rust/bindings/wasm/` exposes the corresponding browser-WASM operations over the same core. The runtime-isolated facade in `npm/packages/typescript/` selects the appropriate backend. The optional React adapter in `npm/packages/react/` and Vue adapter in `npm/packages/vue/` consume only the browser WASM package and keep their frameworks as peer dependencies. Neither runtime binding or framework adapter may copy core algorithms.

Rust is the accepted strategic shared-core implementation in ADR 0001. The scene-v1 JSON contract remains the language-neutral interchange and conformance format, not a competing implementation of the shared semantics. Production R adoption remains gated by CRAN disposition for the retained Cargo vendor-metadata NOTE, compatibility with the intended CRAN toolchain, and evidence from the actual production package artifact. The C++/Rcpp maintainer-effort comparison was discontinued by #269.

## Shared fixture ownership

The canonical sources and package copies are:

| Canonical source | Package copy | Policy |
| --- | --- | --- |
| `prototypes/normalized-matrix-v1/fixtures/valid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/valid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `prototypes/normalized-matrix-v1/fixtures/invalid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/invalid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `tests/legacy-visual/cases.json` | `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json` | Exact byte-for-byte package-test copy. |
| `tests/fixtures/general-matrix-cases.json` | `implementations/r/inst/extdata/general-matrix-cases.json` | The package copy is a declared projection: it omits the canonical `all-missing` invalid render case. Keep all other content identical. |

`tests/test_fixture_integrity.py` checks the exact copies and the declared general-matrix projection. Update that test and this table together if the ownership or projection changes. Do not edit package copies independently.

The renderer capability declarations are intentionally backend-specific, not duplicate fixtures: the Python reference declaration is in `prototypes/shared-svg/renderer-capabilities.json`, while the Rust CLI declaration is in `rust/tools/corrplot-cli/renderer-capabilities.json`. They must be validated against the shared schema, but their capability values are expected to differ.

## Change rules

1. Put shared semantic behavior in `rust/crates/corrplot-core`, not in CLI or binding adapters.
2. Keep R-specific API compatibility and graphics-device side effects in `implementations/r`.
3. Put cross-runtime conformance fixtures in `tests/` or alongside the normative contract they define. A package copy is allowed only when packaging or package-local tests require it.
4. Give every package copy an explicit source-of-truth and an automated drift check.
5. Keep reference code and disposable spikes clearly labeled; do not present them as production backends.
6. Before moving files, inventory references and update workflows, manifests, scripts, tests, documentation, and examples in the same staged change.
