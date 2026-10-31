# Repository structure and ownership

This document defines the purpose and ownership of the repository's top-level
paths. It also distinguishes normative contracts, reference implementations,
disposable experiments, and package-local copies. It is the directory map for
contributors; it does not change any public API or file path.

## Top-level directory map

| Path | Purpose and ownership | What belongs here |
| --- | --- | --- |
| `.github/` | Repository automation and policy | GitHub Actions workflows, issue/PR templates, and repository configuration |
| `crates/` | Reusable Rust libraries | Shared semantics that must not depend on a CLI, R, Node, or browser adapter; currently `corrplot-core` |
| `tools/` | Standalone developer and user-facing tools | Native CLI applications and their command-specific configuration/assets |
| `implementations/` | Language-specific packages and adapters | The provisional R package, R API/compatibility code, package metadata, and package-local tests/data |
| `prototypes/` | Transitional home for versioned contracts, reference implementations, and disposable integration experiments | Normative versioned contract specifications/fixtures, the Python scene/SVG reference, and isolated Rust/rextendr and C++/Rcpp spikes; see the lifecycle categories below |
| `tests/` | Repository-level tests and canonical shared fixtures | Cross-language fixture sources, fixture-integrity tests, and pinned legacy visual case inventory |
| `benchmarks/` | Performance and resource measurements | Benchmark harnesses, measurement protocols, and benchmark-specific workflows/data |
| `scripts/` | Repository maintenance and validation helpers | Snapshot regeneration, artifact validation, fixture maintenance, and other build/test tooling |
| `docs/` | Authoritative project knowledge | Architecture, requirements, ADRs, compatibility notes, testing strategy, and this directory map |

Top-level files such as `README.md`, `LICENSE`, and Cargo workspace/package
manifests describe the repository as a whole. They do not own implementation
logic.

## Lifecycle categories

The current `prototypes/` path contains different asset types. Their
lifecycle is defined by what they are, not by the directory name:

1. **Normative versioned contracts.** These define language-neutral formats
   and compatibility rules. Current examples include
   `normalized-matrix-v1/`, `publication-config-v1/`,
   `publication-svg-v1/`, `renderer-capabilities-v1.schema.json`, and
   the scene-v1 specification/schema. Contract files and their canonical
   fixtures are not disposable merely because they currently live below
   `prototypes/`. Changes require compatibility review and versioning.
2. **Reference implementations.** `shared-svg/` is the Python reference
   scene/SVG implementation. It is useful for semantic examples and tests,
   but it is not the production shared core and must not become a runtime
   dependency of the native CLI or R package.
3. **Disposable integration experiments.** `rust-rextendr/` and
   `cpp-rcpp/` are isolated binding/package experiments. They provide
   evidence about build, packaging, and adapter costs. They are not
   production R backends; generated package/build artifacts remain
   uncommitted.

The Rust shared library belongs in `crates/`, not inside `tools/` or a
language-specific adapter. The CLI in `tools/corrplot-ng/` and the
rextendr experiment in `prototypes/rust-rextendr/` are consumers of
`crates/corrplot-core`. The current proof covers shared matrix validation;
it does not mean that the full visualization engine has already moved into
the crate.

## Shared fixture ownership

A shared fixture has one canonical source. A package copy is permitted only
when it is needed by package tests or must be installed with the package.
The repository-level `tests/test_fixture_integrity.py` checks byte-for-byte
identity for every declared package mirror during the foundation workflow.

| Canonical source | Required R package copy | Reason |
| --- | --- | --- |
| `prototypes/normalized-matrix-v1/fixtures/valid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/valid-cases.json` | R adapter tests load the contract cases through `system.file()`; the file must be installed with the R package |
| `prototypes/normalized-matrix-v1/fixtures/invalid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/invalid-cases.json` | Package-installed invalid-input cases remain available to R tests and downstream package diagnostics |
| `tests/fixtures/general-matrix-cases.json` | `implementations/r/inst/extdata/general-matrix-cases.json` | R scene-adapter tests load the reference cases from the installed package |
| `tests/legacy-visual/cases.json` | `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json` | The R package test suite needs its case inventory when run from the package |

Update a package mirror from its canonical source; do not edit both copies
independently. The general-matrix R mirror is now byte-identical to the
canonical fixture, including the all-missing invalid case. The R test loops
through all canonical valid and invalid cases.

`tests/fixtures/correlation-cases.json` is a repository-level shared fixture
with no package mirror. The 15 legacy PNG baselines live under the R package's
test fixtures because they are consumed by the R visual-regression tests;
their case inventory remains canonical in `tests/legacy-visual/cases.json`.

## Renderer capability manifests are not duplicates

`prototypes/shared-svg/renderer-capabilities.json` and
`tools/corrplot-ng/renderer-capabilities.json` are intentionally different
backend declarations. They share the schema in
`prototypes/renderer-capabilities-v1.schema.json`, but describe different
renderers, supported cell methods, output formats, features, and negotiation
modes. Do not copy one over the other to make the files identical. The
fixture-integrity tests assert their distinct renderer identities and
backend-specific capability sets; renderer-specific behavior must be updated
in the corresponding manifest and implementation together.

## Path migration policy

No broad directory migration is part of this audit. Moving the normative
contracts from `prototypes/` to a top-level `contracts/`, moving the Python
reference to `references/`, or moving disposable spikes to
`experiments/` could improve discoverability, but it would also change
schema/fixture paths, package copy sources, CI path filters, scripts, links,
and documented commands.

The current taxonomy makes lifecycle and ownership explicit without
introducing that compatibility risk. If a later change moves these paths,
first inventory every tracked and generated reference, then use a staged
migration with compatibility redirects or synchronized transition paths,
and update all CI filters and package-generation scripts in the same reviewed
change. Do not move files solely for aesthetic consistency.
