# ADR 0001: Shared-core architecture for corrplot-ng

- **Status:** Accepted — Rust is the sole selected native implementation; production R adoption remains gated by CRAN/toolchain and package-footprint evidence
- **Date:** 2026-10-31
- **Decision owners:** corrplot-ng maintainers
- **Related:** [Issue #12](https://github.com/vsimko/corrplot-ng/issues/12), [Rust/rextendr spike #13](https://github.com/vsimko/corrplot-ng/pull/13), [C++/Rcpp spike #14](https://github.com/vsimko/corrplot-ng/pull/14), [language evaluation #116](https://github.com/vsimko/corrplot-ng/issues/116)

## Context

corrplot-ng should preserve the established R `corrplot` interface while enabling a modern API and possible wrappers for R, Python, and TypeScript. Maintainer capacity is limited, so the architecture must minimize duplicated semantics and ongoing release/CI work.

The repository has a Python scene reference, Rust/rextendr and C++/Rcpp R-binding spikes, a standalone Rust CLI, and a versioned renderer-neutral scene-v1 contract. Both native integrations build across Windows, macOS, and Linux. Merged PR #180 additionally proves that the CLI and rextendr adapter consume one compiled Rust library for matrix validation; this remains a focused integration proof rather than a production R migration.

## Decision

**Use one authoritative Rust implementation for shared correlation/visualization semantics and core scene operations, with thin language-specific adapters. Do not maintain independent production implementations of those semantics in R, CLI, Node, Python, or browser code.**

This is the project's target architecture, not a claim that the shared library is already implemented. The existing Rust CLI makes Rust the coherent core language across native CLI, R, Node, and WebAssembly consumers. The reason is broader than raw speed: one implementation is maintained and tested once, consumers receive the same semantic behavior, fixes apply consistently, and performance-critical algorithms do not need to be reimplemented in each language.

The Rust CLI and R/rextendr spike now call the same local `corrplot-core` library for matrix validation, as demonstrated by merged PR #180. Issue #173 continues to track the remaining CRAN/vendor-metadata disposition and production-R package evidence; this focused proof does not authorize migration of the production R package.

C++/Rcpp was evaluated and explicitly rejected. The project will use Rust for the shared native implementation and will not maintain a parallel C++ implementation or an Rcpp fallback. R integration work must use the Rust/rextendr path or a clearly documented non-C++ alternative if a future constraint requires one.

## Shared-crate proof: Issue #173

Merged PR #180 extracts validate_matrix into the dependency-free
rust/crates/corrplot-core library. Both the native CLI and the disposable
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
cross-language interchange boundary. The proposed human maintainer-time
comparison against C++/Rcpp was discontinued by issue #269 on 2026-11-04;
no comparative timings were collected. The historical 1–2 h versus 2–4 h
figures remain unmeasured planning estimates, not rollout criteria.
## Scene-construction progress

Merged PR #237 and PR #243 establish the shared Rust scene-v1 parser and normalized-matrix-to-scene construction path, including strict diagnostics for invalid display options. The active follow-up PR #236 extends the same renderer-neutral `Scene` with deterministic label wrapping, multiline SVG/RGBA text, rotation in both SVG and RGBA, document descriptions, and semantic roles for figure text and cell annotations. CLI, experimental Rust/rextendr, Node-API, and browser-WASM tests consume the same scene-v1 construction fixture.

This remains an incremental implementation rather than a claim of full scene-v1 presentation fidelity. Plotmath is explicitly unsupported and rejected with a path-specific diagnostic; the RGBA bitmap renderer does not reproduce SVG font-family, weight, or style attributes. The document description and text roles are preserved in the scene and exposed in SVG accessibility attributes. PR #236 remains in progress until its full acceptance criteria and current CI are verified.

## Evidence reviewed

The evaluation issues and merged implementation PRs provide the following evidence:

| Area | Evidence | Finding |
|---|---|---|
| CRAN source-package policy | [#111 / PR #118](https://github.com/vsimko/corrplot-ng/pull/118), `docs/cran-rust-cpp-comparison.md` | Both approaches are feasible. C++/Rcpp passed three-platform `R CMD check --as-cran`. Rust/rextendr 0.9.0 passed offline source-package checks on three platforms with one documented NOTE for Cargo vendor metadata; the earlier `R_NamespaceRegistry` NOTE was removed by the upstream 0.9.0 fix. Rust 1.78/Cargo 1.78 is declared and covered by CI. Actual CRAN acceptance remains unproven. |
| Historical maintainer workflow | [#113 / PR #119](https://github.com/vsimko/corrplot-ng/pull/119), `docs/native-integration-maintenance-comparison.md`, resolution of #269 | The report retains 1–2 h versus 2–4 h planning estimates. The proposed comparison was discontinued; no human-effort measurements were collected, and these estimates are not current decision evidence. |
| Binary installation | [Rust PR #16](https://github.com/vsimko/corrplot-ng/pull/16), [C++ PR #17](https://github.com/vsimko/corrplot-ng/pull/17) | Both Windows binaries installed and passed smoke tests with the relevant compiler/toolchain removed from the restricted `PATH`. This addresses binary end-user installation, not source-build or CRAN acceptance. |
| Scene independence | [#115 / PR #121](https://github.com/vsimko/corrplot-ng/pull/121) | Python, R, and the standalone Rust CLI consume the versioned scene-v1 JSON fixture. The Rust CLI validates the data contract independently of R. This demonstrates a language-neutral interchange boundary, not reuse of one compiled native library. |
| Parity and performance | [#114 / PR #122](https://github.com/vsimko/corrplot-ng/pull/122), [follow-up PR #124](https://github.com/vsimko/corrplot-ng/pull/124), `docs/historical-cross-language-benchmark.md` | Shared fixtures and validation cover row-major cells, missing values, annotations, dimensions, invalid values, and NA-versus-NaN behavior. Existing Linux-only scene-v1 benchmarks show Rust ahead of C++ in the tested workloads, but the gap is implementation- and workload-specific and is not sufficient to select a production language. |
| Standalone CLI | [#98](https://github.com/vsimko/corrplot-ng/issues/98), [PR #120](https://github.com/vsimko/corrplot-ng/pull/120) | Rust stable, edition 2021, was selected for the standalone CLI. Its dependency-free binary is built and audited separately; this is a useful alignment factor for a future Rust shared core, but it does not by itself prove that the R package should adopt rextendr. |

### Interpretation

The evidence supports Rust as the shared-core target. Rust is the sole selected native implementation. C++/Rcpp is not retained as an alternative. The remaining question is whether the shared Rust crate can satisfy R packaging and toolchain requirements sustainably.

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

**Assessment:** selected as the repository-wide shared-core implementation. The shared-library proof and tested MSRV are complete; production R adoption remains gated by CRAN disposition for the vendor-metadata NOTE, intended toolchain compatibility, and evidence from the actual production package artifact and its footprint.

### C++ + Rcpp

**Assessment: rejected.** The project will not maintain the C++/Rcpp prototype, its CI workflow, or an Rcpp fallback. The Rust shared core is the only selected native implementation. This is an explicit project direction, not a claim that every Rust/R packaging gate is already resolved.

### Keep Python as the production core

Useful as a reference implementation and semantic oracle, but not selected as a production runtime dependency. Requiring Python for users who only need the R package or standalone CLI would add a separate deployment requirement.

### Separate implementations per language

Rejected. Duplicating matrix normalization, ordering, and visualization semantics would increase regression risk and exceed the project's maintenance capacity. Language-specific adapters may remain separate, but they must conform to the same versioned scene contract.

## Consequences and implementation boundaries

- Scene-v1 remains the stable experimental interchange boundary. It describes plot meaning and geometry, not a language-specific object graph or renderer API.
- Keep the Python scene prototype as the reference producer and shared fixtures as the semantic compatibility gate.
- Keep the Rust/rextendr integration isolated until its production R rollout gates are met. Retire the C++/Rcpp spike; do not introduce a parallel native implementation.
- Preserve the legacy R API, defaults, return values, graphics-device behavior, and compatibility tests. No breaking migration is implied.
- Keep the standalone CLI in Rust as already decided in #98. This decision does not require the CLI to depend on the rextendr package.
- Python may continue to act as a reference implementation and test oracle during migration. Production consumers that need shared semantics should call the Rust core through an appropriate thin binding; scene-v1 remains the language-neutral interchange and conformance format.
- Do not use the prototype benchmark to claim a production performance win.

## Measured release-readiness results

The disposable Rust/rextendr package generator's build-time linker-stripping mode was measured on 2026-10-31:

| Platform | Installed package | Native library | Source tarball | R CMD check |
|---|---:|---:|---:|---|
| Linux | 2,112,721 bytes | 2,100,280 bytes | 1,049,181 bytes | 1 NOTE |
| macOS | 1,805,047 bytes | 1,792,600 bytes | 1,048,308 bytes | 1 NOTE |
| Windows | 1,176,796 bytes | 1,164,288 bytes | 1,041,008 bytes | 1 NOTE |

Linux uses GNU `--strip-debug`, macOS uses Apple's `-S`, and Windows remains unstripped pending a validated PE/COFF approach. The offline install, R smoke test, source-tarball build, and `R CMD check --as-cran` passed on all three platforms.

The remaining NOTE reports hidden Cargo `.cargo-checksum.json` and `.cargo_vcs_info.json` metadata. These files are retained for offline dependency integrity and source provenance; do not remove them merely to silence the NOTE. The source archives are about 1.0 MiB on all three platforms. These are disposable-prototype measurements, not CRAN acceptance or production-package measurements. The CRAN disposition is tracked in #268.

## R-host WASM distribution experiment — 2026-11-05

The completed manual R-host matrix in workflow run [#41981917849](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849) passed all nine runner jobs (three repetitions each on Ubuntu Linux, Intel macOS, and Windows). Node and R invoked matching WASI and embedded-ABI module bytes within each platform, and all six fixture responses matched across Node, R/Wasmtime, R/V8, native Rust, and the native-R reference. The detailed toolchain/platform matrix, module hashes, raw artifact links, and machine-readable results are recorded in [the R-host WASM benchmark report](../experiments/wasm-r/results/benchmark-2026-11-05.md).

At the tested JSON validation boundary, the native Rust R binding had the lowest median latency on all six fixtures and all three platforms. For the 512×512 fixture it measured 5.19 ms/call on Ubuntu, 9.42 ms/call on Intel macOS, and 5.90 ms/call on Windows. Embedded Wasmtime initialization was 45.9–59.0 ms median and first-call latency 15.1–20.8 ms median. Its shared library measured 65.5–102.7 MiB, compared with a 144 KiB embedded ABI module and a 0.63–2.22 MiB native Rust R binding library. V8 and Wasmtime did not have a universal relative throughput winner.

The measurements distinguish current process RSS snapshots, process-lifetime peak RSS, and WASM linear memory. Peak RSS includes R and Wasmtime allocations; it is not a WASM-only metric. Darwin peak RSS is explicitly unavailable and represented as JSON null rather than inferred from an RSS snapshot. WASM linear memory after the 512×512 call was 10.75 MiB.

**Experiment recommendation: retain native Rust as the R integration direction and WASM for Node/browser consumers.** Process-based WASI remains a portability/feasibility baseline, not the preferred high-throughput R interface. These synthetic validation microbenchmarks are not corrplot rendering or end-to-end package benchmarks, do not establish CRAN acceptance, and do not authorize migration or release of the production R package. The production package, full dependency/license closure, offline installation, R CMD check --as-cran, supported-platform behavior, and the CRAN vendor-metadata disposition in #268 remain separate rollout gates.

## Conditions for production rollout

The strategic language decision (Rust only; C++/Rcpp rejected) and the shared-library integration gate are complete: the CLI and disposable R/rextendr binding link to `rust/crates/corrplot-core` and invoke the same tested validation operation. Remaining gates concern CRAN disposition for the vendor-metadata NOTE, compatibility with the intended CRAN toolchain, and validation of the actual production R package artifact and its footprint. The proposed C++/Rcpp human-effort comparison was discontinued by #269 and is not a gate.

Before making the shared Rust crate a production R dependency:

- Retain the single Cargo vendor metadata NOTE only with an explicit CRAN-comments explanation: `.cargo-checksum.json` is required for offline vendored-source integrity and `.cargo_vcs_info.json` preserves source provenance. Do not delete these files merely to silence `R CMD check`.
- Keep Rust 1.78/Cargo 1.78 as the declared and CI-tested MSRV, and confirm the intended minimum against the actual CRAN build environment before submission.
- Record the generated source-tarball and installed-package sizes and assess the installed size against rextendr's 5 MB caution.
- Keep dependency vendoring, offline compilation, license notices, Cargo parallelism, and source-package size as release gates. The C++/Rcpp comparison was discontinued by explicit project decision; no human-effort measurements were claimed.

If Rust's R packaging or CRAN/toolchain burden cannot be brought within the maintainer budget, keep Rust authoritative for the CLI and other consumers and evaluate a narrowly scoped pure-R adapter as an explicit exception. C++/Rcpp is not an approved fallback. Any alternative adapter must conform to the same versioned contracts and shared fixtures; it must not silently redefine common semantics.

## Rollback and exit strategy

No production package depends on either spike yet, so the shared-library rollout can be staged without a breaking migration. When a future implementation is attempted:

1. Keep the language-specific implementation behind the existing scene-v1 and R adapter boundaries.
2. Require semantic parity and legacy compatibility before switching the default path.
3. Retain the previous implementation until the new path passes supported-platform package checks and binary/source installation tests.
4. If Rust's CRAN/toolchain conditions cannot be met within the maintainer budget, do not force an unsafe or unmaintainable Rust dependency into the R package. Keep Rust as the authoritative core for consumers it can serve, and document any R-only adapter exception against the same conformance fixtures. The scene-v1 schema and fixtures remain unchanged.
5. C++/Rcpp is not an option under the current project decision. Retain the language-neutral interchange boundary if a future adapter constraint arises.

## Revisit the rollout plan when

- The shared-crate proof identifies a better crate boundary or binding strategy.
- CRAN toolchain policy, rextendr maintenance costs, or binary distribution constraints change.
- A full compatibility and performance comparison exists for representative corrplot workloads rather than only the current scene-v1 prototype.


## 2026-11-13 clarification: full visualization-engine target

This ADR's Rust selection is unchanged and now applies explicitly to the complete visualization engine, not only matrix validation or the current scene-v1 operations. Rust is the sole production authority for shared visualization semantics, layout, and rendering. The R package's upstream corrplot delegation is a temporary migration fallback and test oracle, not a permanent second renderer.

The project has two public API surfaces: a compatibility adapter for the complete pinned corrplot 0.95 behavior and a separate modern composable API. Both compile into one general, versioned, renderer-neutral visualization model and use the same Rust implementation. R-specific argument conversion, statistical helpers, return objects, and graphics-device integration remain in a thin R adapter where they are host-specific.

The current scene-v1 contract is not assumed to be sufficiently general. A new model version or distinct visualization-model contract must be designed explicitly, while preserving existing scene-v1 compatibility. See [ADR 0004](0004-unified-visualization-model-and-dual-api.md) and [#430](https://github.com/vsimko/corrplot-ng/issues/430).
