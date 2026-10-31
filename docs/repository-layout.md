# Repository layout and ownership

This document defines where corrplot-ng code, contracts, fixtures, tests, and release assets belong. It describes the current layout; it does not authorize a broad directory migration.

## Top-level directories

| Path | Ownership and contents |
| --- | --- |
| `crates/` | Reusable Rust libraries that own shared implementation semantics. `crates/corrplot-core` is the authoritative Rust shared-core crate; it currently provides matrix validation and typed-scene SVG/RGBA rendering. Keep consumer-specific command handling and bindings out of this directory. |
| `bindings/` | Runtime-specific native/WASM adapters over reusable Rust crates. `bindings/node/` contains the experimental Node-API addon and its JavaScript/TypeScript surface; browser WASM remains future work. |
| `tools/` | Executable developer and user tools. `tools/corrplot-ng` is the standalone Rust CLI and should remain a thin application adapter over shared crates. |
| `implementations/` | Language-specific product packages and adapters. The R package owns R APIs, legacy compatibility, graphics-device behavior, and package-specific tests. |
| `prototypes/` | A transitional home for three distinct asset classes: normative versioned contracts, reference implementations, and disposable experiments. Do not treat every child directory as production code. |
| `tests/` | Cross-language fixtures, compatibility oracles, and repository-level tests that are not owned by one runtime package. |
| `scripts/` | Maintainer and CI utilities that operate across packages or validate generated artifacts. |
| `benchmarks/` | Opt-in performance and resource measurements. Benchmarks are not part of routine correctness checks unless a specific issue requires them. |
| `docs/` | Architecture decisions, contracts and workflow documentation, testing guidance, and the roadmap. |
| `.github/` | GitHub Actions workflows, issue templates, and repository automation. |
| `.cargo/` | Shared Cargo configuration and dependency-vendoring metadata. |

Root-level manifests and configuration files remain at the repository root when their tools expect them there.

## Categories currently grouped under `prototypes/`

The current path layout is retained to avoid breaking references, package manifests, CI filters, and documented commands. The categories have different stability expectations:

- **Normative, versioned contracts:** `normalized-matrix-v1/`, `publication-config-v1/`, `publication-svg-v1/`, the scene-v1 schema and fixtures, and `renderer-capabilities-v1.schema.json`. Their schemas and compatibility rules are normative; their current location under `prototypes/` is historical, not a statement that the contracts are throwaway.
- **Reference implementation:** `shared-svg/` is the Python scene/SVG reference implementation. It is useful for semantic examples and conformance work, but it is not the selected production shared core.
- **Disposable integration experiments:** `rust-rextendr/` and `cpp-rcpp/` are evaluation spikes. The Rust spike demonstrates that an R binding and the CLI can call the same crate; it is not the production R package.

Future work may move these categories to clearer top-level locations such as `contracts/`, `references/`, and `experiments/`. Do not move them until a separate staged change inventories and updates every relative link, package path, CI path filter, fixture consumer, and developer command.

## Rust package boundary

- `crates/corrplot-core/` owns reusable, language-neutral Rust semantics. It currently provides shared matrix validation plus typed-scene SVG and direct RGBA rendering; the complete scene-v1 parser and matrix-to-scene builder are not yet implemented.
- `tools/corrplot-ng/` owns CLI parsing, file and process I/O, command orchestration, and executable packaging.
- `prototypes/rust-rextendr/` owns only the disposable R-binding proof and its generated package workflow.
- `implementations/r/` owns the production R-facing API and legacy compatibility behavior. Do not migrate it to rextendr as part of a layout cleanup.
- `bindings/node/` is the initial Node-API adapter. It exposes validation and typed-scene SVG/RGBA rendering without duplicating semantics; the complete scene-v1 parser/builder and platform-specific prebuilt publishing remain open. Future browser WebAssembly consumers should use the same crate through a thin adapter. Neither adapter may copy core algorithms.

Rust is the accepted strategic shared-core implementation in ADR 0001. The scene-v1 JSON contract remains the language-neutral interchange and conformance format, not a competing implementation of the shared semantics. Production R rollout still requires CRAN guidance and measured maintainer effort.

## Shared fixture ownership

The canonical sources and package copies are:

| Canonical source | Package copy | Policy |
| --- | --- | --- |
| `prototypes/normalized-matrix-v1/fixtures/valid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/valid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `prototypes/normalized-matrix-v1/fixtures/invalid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/invalid-cases.json` | Exact byte-for-byte copy; required in the installed R package. |
| `tests/legacy-visual/cases.json` | `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json` | Exact byte-for-byte package-test copy. |
| `tests/fixtures/general-matrix-cases.json` | `implementations/r/inst/extdata/general-matrix-cases.json` | The package copy is a declared projection: it omits the canonical `all-missing` invalid render case. Keep all other content identical. |

`tests/test_fixture_integrity.py` checks the exact copies and the declared general-matrix projection. Update that test and this table together if the ownership or projection changes. Do not edit package copies independently.

The renderer capability declarations are intentionally backend-specific, not duplicate fixtures: the Python reference declaration is in `prototypes/shared-svg/renderer-capabilities.json`, while the Rust CLI declaration is in `tools/corrplot-ng/renderer-capabilities.json`. They must be validated against the shared schema, but their capability values are expected to differ.

## Change rules

1. Put shared semantic behavior in `crates/corrplot-core`, not in CLI or binding adapters.
2. Keep R-specific API compatibility and graphics-device side effects in `implementations/r`.
3. Put cross-runtime conformance fixtures in `tests/` or alongside the normative contract they define. A package copy is allowed only when packaging or package-local tests require it.
4. Give every package copy an explicit source-of-truth and an automated drift check.
5. Keep reference code and disposable spikes clearly labeled; do not present them as production backends.
6. Before moving files, inventory references and update workflows, manifests, scripts, tests, documentation, and examples in the same staged change.
