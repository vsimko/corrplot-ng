---
okf_version: "0.2"
type: Engineering Strategy
title: Testing strategy
description: Describe the test layers, current CI checks, and evidence limits for corrplot-ng.
tags:
  - testing
  - compatibility
  - quality
status: active
---

# Testing strategy

## Purpose

Protect the behavior that exists today and make the limits of each test
explicit. The repository has several independent test layers. Passing one
layer does not prove full compatibility with the original `corrplot`
package, shared compiled-core parity, or visual equivalence across renderers.

The current implementation includes:

- A provisional R package with a legacy-compatible delegation path and an
  experimental scene-v1 preparation path.
- A language-neutral normalized-matrix v1 input contract with R and file
  adapters.
- A Python scene/SVG reference prototype and an R scene-v1 consumer.
- A Rust shared core with matrix validation, an initial matrix-bearing
  scene-v1 parser/builder, and typed-scene SVG/RGBA rendering. The CLI,
  Node-API, browser-WASM, and disposable R/rextendr consumers use this core
  for their implemented operations.
- An experimental Node-API binding and browser-WASM binding, plus an initial
  TypeScript facade with runtime-specific backend selection.
- Experimental C++/Rcpp and Rust/rextendr R-integration candidates; the
  rextendr adapter is disposable evaluation infrastructure, not the
  production R package.

## Test layers

### 1. Shared fixtures and normalized-matrix v1

The general fixtures in `tests/fixtures/correlation-cases.json` describe
language-neutral matrix cases. The normalized-matrix v1 contract in
`prototypes/normalized-matrix-v1/` defines the canonical input shape,
including square dimensions, exact Unicode labels, zero-based source-index
mapping, and JSON `null` for missing values. Zero remains a numeric value.

The contract suite checks the schema and semantic rules using valid and
invalid fixtures. The R package tests exercise matrix and file adapters.
They cover R matrix input and CSV/TSV/normalized-matrix JSON file input,
including label handling, missing values, and source-index preservation.
The adapter parity test sends equivalent labelled matrix data through the R
matrix and CSV adapters and compares normalized representations while
allowing source provenance to differ.

Run the contract tests from the repository root:

```sh
python -m unittest discover -s tests -v
python -m unittest discover -s prototypes/normalized-matrix-v1 -p 'test_*.py' -v
```

The R adapter tests run through `R CMD check implementations/r`.

This layer validates already-computed matrices. It does not calculate
correlations from raw observations and does not test rendering.

### 2. R compatibility and statistical helpers

The default exported `corrplot_ng()` path delegates to the pinned upstream
`corrplot` package. The R package check verifies package installation and
its automated tests against the declared legacy reference version
(`corrplot` 0.95).

The saved-raster regression suite compares 15 selected 900×900 PNG outputs
with committed reference images. It verifies the R compatibility adapter,
not the independent Python SVG renderer or a future production engine.
Pixel equality is meaningful only in the pinned CI graphics environment;
it is not a cross-platform rendering contract.

The R package also tests its experimental scene path, normalized input
adapters, `cor.mtest()`, and exported palette/color-legend helpers. The
statistical helper computes pairwise p-values and confidence intervals; it
does not move statistical calculations into the renderer.

The legacy semantic oracle is a separate foundation workflow job. It installs
the pinned upstream source, exports representative legacy matrix semantics,
and checks the scene bridge. This is targeted evidence, not exhaustive
coverage of every public argument or option interaction.

See [the regression test guide](../tests/README.md) for the pinned reference,
snapshot regeneration procedure, fixture inventory, and platform limitations.

Run the package check from the repository root:

```sh
R CMD check implementations/r
```

### 3. Scene-v1 contract and renderer-neutral behavior

The Python scene/SVG prototype has its own tests under
`prototypes/shared-svg/`. They cover scene construction, validation,
layout, labels, annotations, missing values, deterministic SVG output, and
the seven prototype glyph methods. The R scene consumer checks that the R
adapter can consume and produce the documented scene-v1 representation.
The legacy-to-scene bridge uses actual matrix output from the pinned legacy
oracle to check selected semantic edge cases.

Run the Python prototype tests with:

```sh
python -m pip install jsonschema
python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v
Rscript prototypes/shared-svg/test_r_scene_consumer.R
```

These checks establish contract and targeted semantic conformance for the
Python and R consumers. The Rust-backed CLI, Node, and WASM integrations use
the same compiled core for the operations they expose; their dedicated tests
are described below. None of these checks establishes complete scene-v1
presentation fidelity or complete legacy API compatibility.

### 4. SVG structure, export, and interoperability

`scripts/validate_svg.py` checks the supported self-contained SVG subset:
well-formed XML, namespace and dimensions, viewBox geometry, unique IDs,
local and accessibility references, and the absence of scripts, event
handlers, remote assets, and external CSS dependencies.

The foundation workflow validates a generated Python SVG and rasterizes it
with librsvg. The Rust CLI workflow generates a native SVG artifact and runs
the same validator on Ubuntu. The broader SVG prototype tests check
structure, geometry, glyph-specific elements, labels, annotations, and
deterministic serialization.

Run the validator against an artifact with:

```sh
python scripts/validate_svg.py figure.svg
```

A manual LaTeX embedding smoke test is documented in
[the regression test guide](../tests/README.md). SVG validation checks a
documented subset; it is not complete SVG 2 conformance and does not promise
identical rendering in every viewer.

Renderer capability declarations are checked against
`prototypes/renderer-capabilities-v1.schema.json`. The shared-SVG prototype
tests strict rejection of unsupported required features and permissive
omission only when the caller explicitly marks a feature optional. Each
omission returns a structured warning. The native CLI exposes its narrow
machine-readable declaration through `corrplot-ng capabilities`. See
[renderer capability negotiation](renderer-capabilities.md).

The diagnostic-v1 schema defines stable codes, severity, stage, and optional
source locations. Tests check serialized diagnostic fields for representative
R, Python, renderer, and CLI failures. See the
[diagnostic catalogue](diagnostics.md).

### 5. Shared Rust core and native CLI

The dependency-free `crates/corrplot-core` library is the accepted strategic
shared implementation under ADR 0001. It currently tests matrix validation,
the matrix-bearing scene-v1 parser and normalized-matrix-to-scene builder,
and typed-scene SVG/RGBA rendering. The CLI, Node, browser-WASM, and
experimental R/rextendr integrations call this crate for their implemented
operations. This is real shared compiled implementation, but not full
scene-v1 semantic or presentation parity.

The shared-core workflow verifies the repository-local rextendr dependency,
compiles the core and its tests for `wasm32-unknown-unknown`, runs native core
tests, and tests the declared Rust 1.78 MSRV. The CLI workflow runs locked,
offline core/CLI tests and builds on Ubuntu, macOS, and Windows for
comprehensive runs; PR validation uses Ubuntu. On Ubuntu it also renders and
validates an SVG and tests the installed executable outside the checkout.

Run local core and CLI tests from the repository root:

```sh
cargo test --locked --offline --manifest-path crates/corrplot-core/Cargo.toml
cargo test --locked --offline --manifest-path tools/corrplot-ng/Cargo.toml
cargo build --locked --offline --release --manifest-path tools/corrplot-ng/Cargo.toml
```

The standalone CLI remains a thin application adapter. Production R rollout
is separately gated by the CRAN and maintainer-workflow evidence in ADR 0001.

### 6. Node, browser-WASM, and TypeScript integrations

The native Node-API binding tests matrix validation, typed-scene SVG/RGBA,
and matrix-bearing scene-v1 to SVG/RGBA operations against the shared Rust
core. Its pull-request matrix is Linux x64 GNU on Node 24. Comprehensive
runs cover Linux x64 GNU, macOS x64, macOS arm64, and Windows x64 on Node
22, 24, and 26, including platform-package assembly checks.

The browser-WASM binding compiles the same core for `wasm32-unknown-unknown`.
Its Node-based binding tests and browser ESM build run on Node 24 for PRs;
comprehensive runs use Node 24 and 26. Browser smoke coverage runs on Node
26 against Chromium, Firefox, and WebKit. Weekly and manual runs also collect
informational WASM size/initialization/rendering baselines; these are not
per-PR performance gates.

The initial TypeScript facade has tests in the foundation workflow. It
selects the runtime-specific Node or WASM backend and currently exposes
validation, matrix-to-SVG/RGBA rendering, capabilities, diagnostics, and
disposal. Broader API completeness, direct Canvas, and React/Vue adapters
remain unfinished. The binding READMEs document their operation and runtime
boundaries.

### Local command entry point

Use `bash scripts/dev-check.sh` from the repository root to run the same
component checks locally without copying long CI command sequences:

- `quick`: fixture integrity, capability-schema validation, normalized
  matrix, shared SVG, publication-config, publication-SVG, and SVG artifact
  checks used by the foundation workflow.
- `test-r` and `test-r-legacy`: R package checks and the pinned legacy
  semantic-oracle bridge.
- `test-rust-cli`, `test-node`, and `test-wasm`: focused native CLI,
  Node-API, and browser-WASM checks.
- `test-all`: combines the locally supported correctness suites.
- `build-cli` and `build-node`: release CLI and native Node addon builds.
- `benchmarks` and `benchmarks-stress`: explicit regular and stress
  measurement commands; neither is part of the quick or full correctness path.

The script checks for required tools and reports actionable installation
hints when a prerequisite is missing. It uses Bash and is supported on Linux,
macOS, WSL, and Git Bash; native PowerShell is not supported. The separate
WASM/R experiment needs its pinned Wasmtime C API and is not folded into
`test-all`.

The pull-request and comprehensive workflow mapping is maintained in
[CI validation tiers](ci-validation-tiers.md). Workflow YAML is linted
independently by `test-workflows.yml`, including manual-only workflows.

### 7. CI execution policy

Routine pull-request CI is correctness-focused and path-filtered where
appropriate. Workflow concurrency cancels superseded runs for the same PR.

| Workflow | Trigger and scope | Main checks |
| --- | --- | --- |
| [Test foundation](../.github/workflows/test-foundation.yml) | All PRs, pushes to `main`, and manual dispatch; component jobs run when foundation paths match | Shared fixture/schema and Python prototype checks, TypeScript facade tests, SVG validation and librsvg interoperability, pinned legacy oracle and legacy-to-scene bridge |
| [R package checks](../.github/workflows/test-r.yml) | All PRs, pushes to `main`, and manual dispatch; R, scene-consumer, fixture, and schema paths are filtered | Pinned `corrplot` 0.95 reference, R scene consumer, and `R CMD check` |
| [Shared Rust core](../.github/workflows/test-shared-rust-core.yml) | PRs and pushes to `main` for core, CLI, rextendr, or workflow paths; manual dispatch | Repository-local rextendr consumer, native core tests, WASM target compile, and Rust 1.78 MSRV checks |
| [Native Rust CLI](../.github/workflows/test-rust-cli.yml) | PRs and pushes to `main` for CLI/core/publication-config/validator paths; manual dispatch | Core and CLI offline tests, Ubuntu PR build, comprehensive Ubuntu/macOS/Windows build, SVG validation, installed-outside-checkout smoke |
| [Native Node binding](../.github/workflows/test-node-native.yml) | PRs and pushes to `main` for Node binding/core/workflow paths; manual dispatch | Linux x64 GNU + Node 24 on PRs; comprehensive Linux/macOS/Windows architecture matrix on Node 22/24/26; native tests and package assembly |
| [Browser WASM binding](../.github/workflows/test-wasm-binding.yml) | PRs and pushes to `main` for WASM binding/core/workflow paths; manual dispatch and weekly schedule | Node 24 PR tests/build; Node 24/26 comprehensive tests, browser ESM package checks, Chromium/Firefox/WebKit smoke on Node 26; scheduled/manual informational baseline artifact |

The foundation workflow's component jobs run only when their detected paths
change (or on manual dispatch); its path filter also includes the initial
TypeScript facade tests. The R workflow similarly filters package-related
paths. Shared-core, CLI, Node, and WASM workflows have explicit path filters.
The PR tier uses representative fast checks; comprehensive matrices and
scheduled WASM baseline collection are intentionally broader. The detailed
path-to-check mapping is maintained in [CI validation tiers](ci-validation-tiers.md).

Performance measurements are not part of routine pull-request CI. The
dedicated **Performance benchmarks** workflow is started manually using
`workflow_dispatch`; it has no scheduled trigger.
Large B03/B04 stress workloads and the three-platform native package smoke
matrix are separate opt-in inputs for manual runs. See
[benchmark operations](../benchmarks/README.md). The native CLI correctness
workflow does not run a baseline-versus-candidate performance comparison on
pull requests.

### 8. Future test extensions

The initial TypeScript, Node native, and browser-WASM slices now have test
coverage, but their broader APIs and production support are not complete.
Direct Canvas, worker orchestration, React/Vue adapters, full scene-v1
presentation coverage, and wider cross-backend conformance remain future
work tracked under issues #139–#146.

When one of these interfaces is implemented:

- Reuse normalized-matrix and scene-v1 fixtures instead of creating a second
  definition of matrix or scene semantics.
- Add a consumer-specific contract test for each supported runtime.
- Keep contract conformance separate from shared compiled-core parity.
- Test renderer structure and behavior independently from pixel comparisons.
- Use reviewed tolerances for cross-renderer visual comparisons; do not
  weaken semantic assertions to accommodate font or rasterizer differences.
- Add platform and packaging checks only for platforms the interface claims
  to support.

Property-based tests may be added when they provide coverage beyond the
existing deterministic edge-case fixtures. Generated tests must use fixed
seeds and report reproducible failing inputs.

## Evidence and compatibility claims

Use precise terms when reporting results:

- **Contract conformance:** an implementation accepts and emits the
  documented versioned data shape and semantics for the tested cases.
- **Adapter parity:** equivalent inputs through two adapters produce
  equivalent normalized representations, apart from explicitly
  source-specific provenance.
- **Semantic compatibility:** selected behavior matches a named legacy
  reference for the covered cases.
- **Structural SVG conformance:** output passes the documented SVG subset
  validator and structural assertions.
- **Pixel regression:** output matches saved raster references in the pinned
  graphics environment.
- **Shared implementation reuse:** multiple interfaces call the same
  compiled implementation for a stated operation. The CLI, Node, browser-WASM,
  and disposable R/rextendr adapter currently reuse `crates/corrplot-core` for
  the operations implemented by each consumer. This does not establish
  complete semantic parity across their public APIs or renderers.
- **Cross-runtime semantic parity:** independently exercised consumers
  produce equivalent results and diagnostics for the same fixtures. Coverage
  is limited to the operations and fixtures actually tested; full parity is
  not established.

Track compatibility per behavior:

- `verified`: an automated test passes against a named reference or
  contract for the stated scope.
- `partial`: only selected inputs or behavior are covered.
- `changed`: behavior changed intentionally and the change is documented.
- `untested`: no adequate automated evidence exists.

The current legacy visual corpus is representative, not exhaustive. Passing
fixtures or a workflow does not establish full compatibility with the
original package. Link each compatibility claim to its test and reference
version. Test coverage percentage alone is not proof of compatibility.

## Acceptance criteria

- Each current test layer is described as implemented, with its evidence and
  limitations.
- Planned TypeScript/native/WASM/Canvas work is not described as running.
- Contract conformance, adapter parity, semantic compatibility, structural
  SVG conformance, pixel regression, and shared implementation parity are
  distinct claims.
- CI triggers and checks agree with the workflow files.
- Performance benchmarks remain manual and outside ordinary PR checks.
- The documented commands and links point to existing files and workflows.
