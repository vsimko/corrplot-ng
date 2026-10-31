# ADR 0001: Shared-core architecture for corrplot-ng

- **Status:** Accepted — Rust is the strategic shared-core implementation; production rollout is gated by integration and maintenance proof
- **Date:** 2026-10-31
- **Decision owners:** corrplot-ng maintainers
- **Related:** [Issue #12](https://github.com/vsimko/corrplot-ng/issues/12), [Rust/rextendr spike #13](https://github.com/vsimko/corrplot-ng/pull/13), [C++/Rcpp spike #14](https://github.com/vsimko/corrplot-ng/pull/14), [language evaluation #116](https://github.com/vsimko/corrplot-ng/issues/116)

## Context

corrplot-ng should preserve the established R `corrplot` interface while enabling a modern API and possible wrappers for R, Python, and TypeScript. Maintainer capacity is limited, so the architecture must minimize duplicated semantics and ongoing release/CI work.

The repository has a Python scene reference, Rust/rextendr and C++/Rcpp R-binding spikes, a standalone Rust CLI, and a versioned renderer-neutral scene-v1 contract. The spikes establish that both native integrations can build and run on Windows, macOS, and Linux. They do not yet establish that one compiled core is actually reused by two production consumers.

## Decision

**Use one authoritative Rust implementation for shared correlation/visualization semantics and core scene operations, with thin language-specific adapters. Do not maintain independent production implementations of those semantics in R, CLI, Node, Python, or browser code.**

This is the project's target architecture, not a claim that the shared library is already implemented. The existing Rust CLI makes Rust the coherent core language across native CLI, R, Node, and WebAssembly consumers. The reason is broader than raw speed: one implementation is maintained and tested once, consumers receive the same semantic behavior, fixes apply consistently, and performance-critical algorithms do not need to be reimplemented in each language.

The Rust CLI and disposable R/rextendr binding now link to the same reusable crate. Issue #173 has established the compiled-library boundary; the remaining work is to verify its minimum toolchain, package footprint, and release/maintenance implications. Do not migrate the legacy R package until the remaining release gates pass.

C++/Rcpp remains useful as an R-only integration comparison and fallback only if Rust's R packaging constraints prove unacceptable. It is not the preferred repository-wide core because it would introduce a second native implementation/integration path for the Rust CLI and future consumers.

## Shared-crate proof: Issue #173

The first real shared-library integration is implemented in
`crates/corrplot-core`. Its `validate_matrix` operation is consumed directly
by both:

- `tools/corrplot-ng`, after CSV/JSON parsing and before publication output;
- `prototypes/rust-rextendr`, before constructing scene-v1 JSON.

Both Cargo manifests use a path dependency to the same crate source and
version. There is no copied validation implementation in either consumer.
The core has no third-party dependencies. The CLI therefore retains a
third-party-dependency-free runtime while sharing application semantics.

The shared operation validates non-empty square dimensions, label uniqueness,
missing-value representation, finite observed values, and correlation range,
symmetry, and diagonal rules. Its input is row-major `Option<f64>`: R's
`NA_REAL` maps to `None`, while an ordinary NaN remains `Some(NaN)` and is
rejected. Error codes and cell coordinates are stable and zero-based.

The new cross-platform workflow runs the core and CLI test suites and compiles
the rextendr consumer on Linux, macOS, and Windows. It verifies that both
consumers resolve `corrplot-core` from the same repository path. A generated
R package is installed and exercised on Linux, including a valid correlation
matrix and a non-symmetric matrix rejected by the shared core.

This proves reuse of one compiled operation; it does not prove that the whole
visualization engine has been migrated or that a production R package is ready
for CRAN. Oldest-toolchain compatibility, full R CMD check across platforms,
Windows R-package installation, and production migration remain gated work.
The renderer-neutral scene-v1 JSON contract remains the cross-language
interchange boundary.

## Evidence reviewed

The evaluation issues and merged implementation PRs provide the following evidence:

| Area | Evidence | Finding |
|---|---|---|
| CRAN source-package policy | [#111 / PR #118](https://github.com/vsimko/corrplot-ng/pull/118), `docs/cran-rust-cpp-comparison.md` | Both approaches are feasible. C++/Rcpp passed three-platform `R CMD check --as-cran`; Rust/rextendr passes offline source-package installation and `R CMD check --as-cran` on Linux, macOS, and Windows. After upgrading to extendr-api 0.9.0, the non-API `R_NamespaceRegistry` NOTE is removed; one NOTE remains for Cargo vendor metadata files that must be included for offline integrity. The metadata NOTE is accepted for this disposable spike with an explicit CRAN-comments rationale; actual CRAN acceptance remains unproven. |
| Maintainer workflow | [#113 / PR #119](https://github.com/vsimko/corrplot-ng/pull/119), `docs/native-integration-maintenance-comparison.md` | C++/Rcpp has the shorter routine and release workflow for an R-only extension. The report's planning estimates are 1–2 h for C++ versus 2–4 h for Rust release-grade validation; these are estimates, not measured timings. |
| Binary installation | [Rust PR #16](https://github.com/vsimko/corrplot-ng/pull/16), [C++ PR #17](https://github.com/vsimko/corrplot-ng/pull/17) | Both Windows binaries installed and passed smoke tests with the relevant compiler/toolchain removed from the restricted `PATH`. This addresses binary end-user installation, not source-build or CRAN acceptance. |
| Scene independence | [#115 / PR #121](https://github.com/vsimko/corrplot-ng/pull/121) | Python, R, and the standalone Rust CLI consume the versioned scene-v1 JSON fixture. The Rust CLI validates the data contract independently of R. This demonstrates a language-neutral interchange boundary, not reuse of one compiled native library. |
| Parity and performance | [#114 / PR #122](https://github.com/vsimko/corrplot-ng/pull/122), [follow-up PR #124](https://github.com/vsimko/corrplot-ng/pull/124), `benchmarks/CROSS-LANGUAGE.md` | Shared fixtures and validation cover row-major cells, missing values, annotations, dimensions, invalid values, and NA-versus-NaN behavior. Existing Linux-only scene-v1 benchmarks show Rust ahead of C++ in the tested workloads, but the gap is implementation- and workload-specific and is not sufficient to select a production language. |
| Standalone CLI | [#98](https://github.com/vsimko/corrplot-ng/issues/98), [PR #120](https://github.com/vsimko/corrplot-ng/pull/120) | Rust stable, edition 2021, was selected for the standalone CLI. Its dependency-free binary is built and audited separately; this is a useful alignment factor for a future Rust shared core, but it does not by itself prove that the R package should adopt rextendr. |

### Interpretation

The evidence supports Rust as the shared-core target. C++/Rcpp remains lower-risk only for an isolated R-only native extension; it is not a competing repository-wide architecture. The remaining question is whether the shared Rust crate can satisfy R packaging and toolchain requirements sustainably.

The measured performance difference does not override these operational considerations. The benchmark covers a small scene-v1 workload set on one Linux runner; it is not a full corrplot workload or a cross-platform performance comparison. The R-independence work proves that the *data contract* can cross language boundaries, not that a Rust or C++ library is already reused by Python or TypeScript.

## Options considered

### Rust + rextendr

**Strengths**
- Aligns with the already-selected Rust standalone CLI toolchain.
- Offers a plausible route to one native core used by the CLI and R binding.
- Has demonstrated Windows/macOS/Linux source builds, offline vendoring, and Windows binary installation without Rust on the end user's PATH.
- Provides strong typing and a compact, measured scene-v1 implementation.

**Conditions and risks**
- The current CRAN-style check reports one NOTE for hidden Cargo vendor metadata. The files are required for offline Cargo checksum/provenance integrity and are retained intentionally; document this in any CRAN submission comments.
- Rust 1.78/Cargo 1.78 is the declared and CI-tested minimum for the shared consumers. This is a project MSRV, not a claim that every older CRAN build image provides it; actual CRAN acceptance remains unproven.
- The vendoring, lockfile, license inventory, generated rextendr scaffold, and offline archive add ongoing maintenance work.
- The CLI and R spike now share `corrplot-core`; the remaining operational question is whether its package footprint and maintainer workflow fit the production R release budget.

**Assessment:** selected as the repository-wide shared-core implementation; production R integration remains gated by CRAN/toolchain and maintenance validation.

### C++ + Rcpp

**Strengths**
- Mature R integration and the lower incremental CRAN/release risk.
- Shorter local build, binding-generation, and release workflow for an R-only extension.
- Three-platform source-tarball checks and Windows binary installation without Rtools on the end user's PATH have been demonstrated.

**Conditions and risks**
- C++ is not the implementation language of the standalone CLI. Selecting it for the shared core would either require a second native integration in the CLI or a separately designed cross-language ABI.
- Generated Rcpp wrappers and compiler/R ABI compatibility still require maintenance.
- The current evidence does not demonstrate a shared C++ core consumed by Python or TypeScript.

**Assessment:** preferred fallback if a concrete requirement calls for an R-only native extension; not selected as the repository-wide shared core.

### Keep Python as the production core

Useful as a reference implementation and semantic oracle, but not selected as a production runtime dependency. Requiring Python for users who only need the R package or standalone CLI would add a separate deployment requirement.

### Separate implementations per language

Rejected. Duplicating matrix normalization, ordering, and visualization semantics would increase regression risk and exceed the project's maintenance capacity. Language-specific adapters may remain separate, but they must conform to the same versioned scene contract.

## Consequences and implementation boundaries

- Scene-v1 remains the stable experimental interchange boundary. It describes plot meaning and geometry, not a language-specific object graph or renderer API.
- Keep the Python scene prototype as the reference producer and shared fixtures as the semantic compatibility gate.
- Keep the Rust/rextendr and C++/Rcpp spikes isolated. Do not add either binding to the production R package yet.
- Preserve the legacy R API, defaults, return values, graphics-device behavior, and compatibility tests. No breaking migration is implied.
- Keep the standalone CLI in Rust as already decided in #98. This decision does not require the CLI to depend on the rextendr package.
- Python may continue to act as a reference implementation and test oracle during migration. Production consumers that need shared semantics should call the Rust core through an appropriate thin binding; scene-v1 remains the language-neutral interchange and conformance format.
- Do not use the prototype benchmark to claim a production performance win.

## Measured release-readiness results

The current disposable R/rextendr source package was measured by the Ubuntu 24.04 CI job with R 4.6.2 and GCC 13.3.0:

| Measurement | Result |
|---|---:|
| Compressed R source tarball | 1,314,420 bytes (about 1.25 MiB) |
| Installed package | 6.0 MB |
| Installed `libs` directory | 5.9 MB |
| `R CMD check --as-cran` | 1 NOTE |

The single remaining NOTE is limited to Cargo's hidden vendor metadata (`.cargo-checksum.json` and `.cargo_vcs_info.json`). The checksum files are required for Cargo to verify vendored sources during offline builds; the VCS files preserve source provenance. They are intentionally retained and must be explained in any CRAN submission comments. Upgrading to extendr-api 0.9.0 removed the non-API `R_NamespaceRegistry` NOTE. Pruning unused `lazy_static/tests` fixtures removed the non-portable-path NOTE.

The shared consumers declare Rust 1.78/Cargo 1.78. CI compiles the core, CLI, and R/rextendr consumer on Rust 1.78; the stable cross-platform matrix covers Ubuntu, macOS, and Windows. The tested native prerequisites are a C toolchain plus R development headers on Linux, Xcode Command Line Tools on macOS, and Rtools/MSVC on Windows. The Windows binary-install workflow verifies that end users can install the built package without Rust on PATH.

The source-package workflow is now reproducible and documented, but controlled elapsed maintainer effort versus the C++/Rcpp adapter has not been measured. Existing time estimates are planning estimates, not observed timings.

## Conditions for production rollout

The strategic language decision is Rust. The shared-library integration gate is satisfied: the CLI and disposable R/rextendr binding link to `crates/corrplot-core` and invoke the same tested validation operation. The remaining gates concern source-package policy, minimum toolchain, package footprint, and maintainer cost.

Before making the shared Rust crate a production dependency:

- Retain the single Cargo-vendor-metadata NOTE only with a CRAN-comments explanation: `.cargo-checksum.json` is required for Cargo's vendored-source integrity checks and `.cargo_vcs_info.json` preserves source provenance; do not delete these files merely to silence `R CMD check`.
- Verify Rust 1.78/Cargo 1.78 on Linux, macOS, and Windows and document platform build prerequisites (Rtools/MSVC on Windows, Xcode Command Line Tools on macOS, and a C toolchain/R headers on Linux).
- Record the actual generated source-tarball and installed-package sizes from the offline R package workflow.
- Keep dependency vendoring, offline compilation, license notices, Cargo parallelism, and source-package size as release gates. The controlled elapsed maintainer-time comparison against C++/Rcpp has not yet been measured; current estimates are planning estimates only.
- Do not migrate the production R package as part of this proof.

If Rust's R packaging or CRAN/toolchain burden cannot be brought within the maintainer budget, keep the Rust core authoritative for CLI and other consumers and evaluate a narrowly scoped R-only C++/Rcpp or pure-R adapter as an explicit exception. Such an exception must conform to the same versioned contracts and shared fixtures; it must not silently redefine common semantics.

## Rollback and exit strategy

No production package depends on either spike yet, so the shared-library rollout can be staged without a breaking migration. When a future implementation is attempted:

1. Keep the language-specific implementation behind the existing scene-v1 and R adapter boundaries.
2. Require semantic parity and legacy compatibility before switching the default path.
3. Retain the previous implementation until the new path passes supported-platform package checks and binary/source installation tests.
4. If Rust's CRAN/toolchain conditions cannot be met within the maintainer budget, do not force an unsafe or unmaintainable Rust dependency into the R package. Keep Rust as the authoritative core for consumers it can serve, and document any R-only adapter exception against the same conformance fixtures. The scene-v1 schema and fixtures remain unchanged.
5. If C++ cannot serve the required non-R consumers without a second implementation, do not force it into the shared-core role; retain the language-neutral interchange boundary and revisit the architecture.

## Revisit the rollout plan when

- The shared-crate proof identifies a better crate boundary or binding strategy.
- CRAN toolchain policy, Rcpp/rextendr maintenance costs, or binary distribution constraints change.
- A full compatibility and performance comparison exists for representative corrplot workloads rather than only the current scene-v1 prototype.
