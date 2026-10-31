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

The Rust CLI and R/rextendr spike are not yet linked to one library crate. Issue #173 therefore proves the smallest real shared-crate integration and its packaging/maintenance feasibility; it no longer asks whether duplicated implementations are a desirable long-term architecture. Do not migrate the legacy R package until the proof and release gates pass.

C++/Rcpp remains useful as an R-only integration comparison and fallback only if Rust's R packaging constraints prove unacceptable. It is not the preferred repository-wide core because it would introduce a second native implementation/integration path for the Rust CLI and future consumers.

## Evidence reviewed

The evaluation issues and merged implementation PRs provide the following evidence:

| Area | Evidence | Finding |
|---|---|---|
| CRAN source-package policy | [#111 / PR #118](https://github.com/vsimko/corrplot-ng/pull/118), `docs/cran-rust-cpp-comparison.md` | Both approaches are feasible. C++/Rcpp passed three-platform `R CMD check --as-cran`; Rust/rextendr passed offline source-package checks on three platforms but reported two NOTEs (vendor metadata and non-API `R_NamespaceRegistry`). Oldest-toolchain compatibility and actual CRAN acceptance remain unproven. |
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
- The current CRAN-style checks report two NOTEs that need resolution or explicit maintainer/CRAN disposition.
- The oldest supported Cargo/rustc versions and actual CRAN acceptance have not been verified.
- The vendoring, lockfile, license inventory, generated rextendr scaffold, and offline archive add ongoing maintenance work.
- No production Rust library is yet shared between the CLI and R binding; that reuse must be demonstrated before it is counted as a benefit.

**Assessment:** viable candidate for a future multi-consumer native core; not justified as an immediate R-package migration.

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
- Future Python and TypeScript consumers should read and validate scene-v1 JSON/Schema. They do not need to embed the R binding or inherit the native core language.
- Do not use the prototype benchmark to claim a production performance win.

## Conditions for production rollout

The strategic language decision is Rust. The remaining gate is implementation viability: demonstrate that at least two real consumers (initially the standalone CLI and a disposable R/rextendr binding) link to the same library crate and invoke the same operation. This is an integration and release-readiness gate, not a reconsideration of whether shared semantics should be duplicated.

Before making the shared Rust crate a production dependency:

- Resolve or obtain explicit disposition for the two current `R CMD check --as-cran` NOTEs.
- Verify a conservative minimum Cargo/rustc toolchain against the oldest supported CRAN build environments.
- Keep dependency vendoring, offline compilation, license notices, Cargo parallelism, and source-package size as release gates.
- Prototype one shared Rust library crate consumed by both the CLI and the R binding; do not count JSON fixture interoperability as compiled-library reuse.

If Rust's R packaging or CRAN/toolchain burden cannot be brought within the maintainer budget, keep the Rust core authoritative for CLI and other consumers and evaluate a narrowly scoped R-only C++/Rcpp or pure-R adapter as an explicit exception. Such an exception must conform to the same versioned contracts and shared fixtures; it must not silently redefine common semantics.

## Rollback and exit strategy

No production package depends on either spike yet, so the shared-library rollout can be staged without a breaking migration. When a future implementation is attempted:

1. Keep the language-specific implementation behind the existing scene-v1 and R adapter boundaries.
2. Require semantic parity and legacy compatibility before switching the default path.
3. Retain the previous implementation until the new path passes supported-platform package checks and binary/source installation tests.
4. If Rust's CRAN/toolchain conditions cannot be met within the maintainer budget, do not force an unsafe or unmaintainable Rust dependency into the R package. Keep Rust as the authoritative core for consumers it can serve, and document any R-only adapter exception against the same conformance fixtures. The scene-v1 schema and fixtures remain unchanged.
5. If C++ cannot serve the required non-R consumers without a second implementation, do not force it into the shared-core role; retain the language-neutral interchange boundary and revisit the architecture.

## Revisit when

- A second production consumer needs the same visualization semantics and a shared compiled library would materially remove duplicated work.
- CRAN toolchain policy, Rcpp/rextendr maintenance costs, or binary distribution constraints change.
- A full compatibility and performance comparison exists for representative corrplot workloads rather than only the current scene-v1 prototype.
