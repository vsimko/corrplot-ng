# ADR 0001: Shared-core architecture for corrplot-ng

- **Status:** Accepted — Rust is the strategic shared-core implementation; production rollout is gated by integration and maintenance proof
- **Date:** 2026-10-31
- **Decision owners:** corrplot-ng maintainers
- **Related:** [Issue #12](https://github.com/vsimko/corrplot-ng/issues/12), [Rust/rextendr spike #13](https://github.com/vsimko/corrplot-ng/pull/13), [C++/Rcpp spike #14](https://github.com/vsimko/corrplot-ng/pull/14), [language evaluation #116](https://github.com/vsimko/corrplot-ng/issues/116)

## Context

corrplot-ng should preserve the established R `corrplot` interface while enabling a modern API and possible wrappers for R, Python, and TypeScript. Maintainer capacity is limited, so the architecture must minimize duplicated semantics and ongoing release/CI work.

The repository has a Python scene reference, Rust/rextendr and C++/Rcpp R-binding spikes, a standalone Rust CLI, and a versioned renderer-neutral scene-v1 contract. Both native integrations build across Windows, macOS, and Linux. Merged PR #180 additionally proves that the CLI and rextendr adapter consume one compiled Rust library for matrix validation; this remains a focused integration proof rather than a production R migration.

## Decision

**Use one authoritative Rust implementation for shared correlation/visualization semantics and core scene operations, with thin language-specific adapters. Do not maintain independent production implementations of those semantics in R, CLI, Node, Python, or browser code.**

This is the project's target architecture, not a claim that the shared library is already implemented. The existing Rust CLI makes Rust the coherent core language across native CLI, R, Node, and WebAssembly consumers. The reason is broader than raw speed: one implementation is maintained and tested once, consumers receive the same semantic behavior, fixes apply consistently, and performance-critical algorithms do not need to be reimplemented in each language.

The Rust CLI and R/rextendr spike now call the same local `corrplot-core` library for matrix validation, as demonstrated by merged PR #180. Issue #173 continues to track the remaining source-package, toolchain, distribution, and maintenance evidence; this focused proof does not authorize migration of the production R package.

C++/Rcpp remains useful as an R-only integration comparison and fallback only if Rust's R packaging constraints prove unacceptable. It is not the preferred repository-wide core because it would introduce a second native implementation/integration path for the Rust CLI and future consumers.

## Shared-crate proof: Issue #173

Merged PR #180 extracts validate_matrix into the dependency-free
crates/corrplot-core library. Both the native CLI and the disposable
R/rextendr adapter call this exact path dependency; neither consumer keeps a
parallel implementation of the shared validation operation.

The operation validates non-empty square dimensions, unique non-empty labels,
finite observed values, and correlation range, symmetry, and unit diagonal.
Its row-major Option<f64> input distinguishes explicit missing values from
ordinary NaN. The R adapter maps NA_REAL to None; NaN remains an observed
non-finite value and is rejected. Errors expose stable codes and zero-based
cell coordinates.

The CLI and R consumers passed shared-core tests/build checks on Linux, macOS,
and Windows. The generated R package also passed offline installation,
binding smoke tests, source-tarball inspection, and R CMD check --as-cran
on all three operating systems. The Windows binary-install test passed with
Rust removed from the user's PATH.

The R integration is pinned to extendr-api 0.9.0. Upstream removed its
R_NamespaceRegistry binding in that release, resolving the corresponding
non-API call NOTE from 0.8.2. See the
[extendr 0.9.0 changelog](https://github.com/extendr/extendr/blob/main/CHANGELOG.md)
and [upstream change #1069](https://github.com/extendr/extendr/pull/1069).

The one remaining R CMD check --as-cran NOTE reports hidden Cargo vendor
metadata (.cargo-checksum.json and .cargo_vcs_info.json) inside the vendored
crate sources. The checksum manifests are retained to preserve Cargo's
integrity checks for reproducible offline source builds. This NOTE is
explicitly accepted for the isolated spike; it is not a claim of CRAN
acceptance. The source archive and installed-package sizes are measured in
the CI evidence recorded with issue #173.

The manifests declare Rust 1.78 as the minimum supported version. Cargo 1.78
is the minimum as well because the committed lockfiles use format version 4;
the selected extendr release itself requires Rust 1.77. A dedicated CI job
tests the core, CLI, and R consumer on Rust/Cargo 1.78.

This is proof of one real shared compiled operation and its packaging
feasibility, not migration of the full visualization engine or production R
package. The renderer-neutral scene-v1 JSON contract remains the
cross-language interchange boundary. A controlled comparison of actual
maintainer elapsed time against the C++/Rcpp-only path has not yet been run;
the existing 1–2 h versus 2–4 h figures remain planning estimates, not
measured effort.
## Evidence reviewed

The evaluation issues and merged implementation PRs provide the following evidence:

| Area | Evidence | Finding |
|---|---|---|
| CRAN source-package policy | [#111 / PR #118](https://github.com/vsimko/corrplot-ng/pull/118), `docs/cran-rust-cpp-comparison.md` | Both approaches are feasible. C++/Rcpp passed three-platform `R CMD check --as-cran`. Rust/rextendr 0.9.0 passed offline source-package checks on three platforms with one documented NOTE for Cargo vendor metadata; the earlier `R_NamespaceRegistry` NOTE was removed by the upstream 0.9.0 fix. Rust 1.78/Cargo 1.78 is declared and covered by CI. Actual CRAN acceptance remains unproven. |
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
- The current CRAN-style check reports one NOTE for Cargo vendor checksum/provenance metadata. These files are retained for offline integrity and provenance and require an explicit explanation in any CRAN submission comments.
- Rust 1.78/Cargo 1.78 is the declared and CI-tested minimum for the shared consumers. This does not establish that every older CRAN build image provides it; actual CRAN acceptance remains unproven.
- The vendoring, lockfile, license inventory, generated rextendr scaffold, and offline archive add ongoing maintenance work.
- The CLI and disposable R/rextendr spike now share `corrplot-core`; the remaining operational questions are package footprint, CRAN disposition, and maintainer cost.

**Assessment:** selected as the repository-wide shared-core implementation. The shared-library proof and tested MSRV are complete; production R integration remains gated by CRAN disposition, package footprint, and maintenance validation.

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

The 2026-10-31 Ubuntu 24.04 / R 4.6.2 / GCC 13.3.0 offline source-package job measured:

| Measurement | Result |
|---|---:|
| Compressed R source tarball | 1,314,220 bytes (about 1.25 MiB) |
| Installed package | 6.0 MB |
| Installed `libs` directory | 5.9 MB |
| `R CMD check --as-cran` | 1 NOTE |

The remaining NOTE is limited to Cargo's hidden vendor checksum/provenance
metadata. The checksum manifests are required for Cargo to verify vendored
sources during offline builds; the VCS metadata preserves source provenance.
They are intentionally retained and must be explained in any CRAN submission
comments. The extendr 0.9.0 upgrade removed the non-API
`R_NamespaceRegistry` NOTE. Pruning unused vendored test/example/benchmark
files removed the non-portable-path NOTE.

The compressed source archive is below CRAN's general 10 MB source-package
guideline, while the installed package exceeds rextendr's 5 MB caution and
needs explicit CRAN guidance before production R use. These are prototype
measurements, not CRAN acceptance. The workflow verifies Rust/Cargo 1.78 and
stable builds on Linux, macOS, and Windows. A controlled measurement of human
maintainer time against C++/Rcpp has not been run; the existing 1–2 h versus
2–4 h figures remain planning estimates, not observed effort.

## Conditions for production rollout

The strategic language decision and the shared-library integration gate are complete: the CLI and disposable R/rextendr binding link to `crates/corrplot-core` and invoke the same tested validation operation. Remaining gates concern source-package policy, package footprint, and sustainable maintainer workflow.

Before making the shared Rust crate a production R dependency:

- Retain the single Cargo vendor metadata NOTE only with an explicit CRAN-comments explanation: `.cargo-checksum.json` is required for offline vendored-source integrity and `.cargo_vcs_info.json` preserves source provenance. Do not delete these files merely to silence `R CMD check`.
- Keep Rust 1.78/Cargo 1.78 as the declared and CI-tested MSRV, and confirm the intended minimum against the actual CRAN build environment before submission.
- Record the generated source-tarball and installed-package sizes and assess the installed size against rextendr's 5 MB caution.
- Keep dependency vendoring, offline compilation, license notices, Cargo parallelism, and source-package size as release gates. The controlled elapsed maintainer-time comparison against C++/Rcpp remains unmeasured; current effort ranges are planning estimates only.

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
