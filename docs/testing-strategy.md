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
- A Python scene/SVG prototype and an R scene-v1 consumer.
- A standalone Rust CLI that consumes publication configuration and exports
  SVG without Python or R at runtime.
- Experimental C++/Rcpp and Rust/rextendr benchmark candidates. These are not
  production shared-core bindings.

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

These checks establish contract and targeted semantic conformance. They do
not establish that Python, R, Rust, C++, and future TypeScript/WASM
implementations share one compiled implementation. Nor do they establish
complete legacy API compatibility.

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

### 5. Native Rust CLI

The standalone CLI is tested independently from the Python and R runtimes.
Its Cargo manifest has no third-party dependencies. The CLI workflow runs
locked, offline Rust tests and builds the release executable on Ubuntu,
macOS, and Windows. On Ubuntu it also renders the example publication
configuration and validates the resulting SVG.

The release workflow separately builds platform artifacts and runs
executable smoke checks. Release packaging checks are not the same as the
routine PR correctness matrix.

Run the local CLI tests and build from the repository root:

```sh
cargo test --locked --offline --manifest-path tools/corrplot-ng/Cargo.toml
cargo build --locked --offline --release --manifest-path tools/corrplot-ng/Cargo.toml
```

The CLI is a standalone application. Its implementation does not make Rust
the selected production shared-core language; ADR 0001 defers that decision.

### 6. CI execution policy

Routine pull-request CI is correctness-focused and path-filtered where
appropriate. Workflow concurrency cancels superseded runs for the same PR.

| Workflow | Trigger and scope | Main checks |
| --- | --- | --- |
| [Test foundation](../.github/workflows/test-foundation.yml) | Pull requests, pushes to `main`, and manual dispatch; foundation paths are filtered | Shared fixture integrity, normalized-matrix contract, Python shared SVG and publication-config/SVG tests, SVG artifact validation, librsvg interoperability, and pinned legacy semantic bridge |
| [R package checks](../.github/workflows/test-r.yml) | Pull requests, pushes to `main`, and manual dispatch; R package and related paths are filtered | Pinned `corrplot` reference, R scene consumer, and `R CMD check` |
| [Native Rust CLI](../.github/workflows/test-rust-cli.yml) | Pull requests and pushes to `main` for CLI-relevant paths, plus manual dispatch | Offline Rust tests, release build on Ubuntu/macOS/Windows, and native SVG validation on Ubuntu |

The foundation workflow runs fixture, SVG, and legacy-oracle jobs only when
the relevant paths change (or on manual dispatch). The R workflow similarly
filters package-related paths. The native CLI matrix is path-filtered and
keeps the three supported CI operating systems.

Performance measurements are not part of routine pull-request CI. The
dedicated **Performance benchmarks** workflow runs its regular suites on a
daily schedule and can also be started manually using `workflow_dispatch`.
Large B03/B04 stress workloads and the three-platform native package smoke
matrix are separate opt-in inputs for manual runs. See
[benchmark operations](../benchmarks/README.md). The native CLI correctness
workflow does not run a baseline-versus-candidate performance comparison on
pull requests.

### 7. Future test extensions

Future TypeScript, Node native bindings, browser WASM, and Canvas work is
tracked separately under issues #139–#146. These are not current test
runners or supported production interfaces.

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
- **Shared implementation parity:** multiple interfaces use the same
  compiled implementation. The current repository does not claim this.

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
