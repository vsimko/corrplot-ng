> **Archive — historical workflow comparison, not current instructions.** This report describes the disposable R-binding candidates as they existed at the snapshot below. Rust is now the sole selected native implementation. The C++/Rcpp prototype and its CI were retired by PR #332. The proposed human-effort comparison was not performed and was discontinued by issue #269 on 2026-11-04; the estimates below are unmeasured historical planning estimates, not current rollout gates. Do not recreate the retired candidate or use this report as a current build recipe.

# Maintainer workflow and CI cost: Rust/rextendr vs C++/Rcpp

**Issue:** [#113](https://github.com/vsimko/corrplot-ng/issues/113)  
**Snapshot reviewed:** `main` at `91981304c6f5509e919f2d086cb8d74e33bbe255`  
**Scope:** historical comparison of the disposable scene-v1 R-binding spikes at the reviewed snapshot. This is an archived workflow assessment, not a current language or production-architecture decision. The companion CRAN policy assessment is handled separately by [issue #111](https://github.com/vsimko/corrplot-ng/issues/111).

## Executive summary

At the reviewed snapshot, both candidates implemented the same narrow R-callable scene-v1 JSON builder and could be generated, compiled, installed, and exercised by the then-current cross-language workflow on Ubuntu, macOS, and Windows. That historical evidence established basic integration feasibility, not equal release readiness. The C++/Rcpp candidate has since been removed; only the Rust/rextendr adapter remains as disposable evaluation infrastructure.

- **C++/Rcpp has the shorter routine package workflow.** Rcpp attributes generate the R/C++ registration wrappers, then ordinary R package installation compiles the C++ source. Its main maintenance cost is keeping generated wrappers in sync and diagnosing the platform's C++ compiler/R toolchain.
- **Rust/rextendr has more build and distribution machinery.** The current package-generation script creates a temporary R package, pins the generated package to extendr 0.9.0, adapts the lockfile and generated Makevars, and adds R wrappers, help, namespace, and tests. Its dedicated CI also checks offline vendoring, source-tarball contents, dependency licenses, and a Windows binary install without Rust on PATH.
- **Current CI investment is asymmetric.** The shared manual cross-language workflow covers both candidates on all three operating systems, but the extra offline-package, dependency-audit, and Windows binary workflows currently cover Rust only. This is evidence of more Rust validation work, not proof that C++ has no corresponding release risks.
- **Current project direction:** Rust is the sole selected native implementation. The C++/Rcpp prototype and workflow were retired by PR #332. The Rust/rextendr adapter remains disposable evaluation infrastructure; it is not the production R package and does not authorize migration.

## What is being compared

| Property | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Prototype entry point | `build_scene_json_cpp(labels, matrix, annotation)` | `build_scene_json_rust(labels, matrix, annotation)` |
| Native implementation | Retired C++/Rcpp spike (removed by PR #332) | Disposable Rust/rextendr spike (historical snapshot) |
| R package metadata | Retired C++/Rcpp package metadata (historical) | Disposable Rust/rextendr package metadata |
| Binding generator | `Rcpp::compileAttributes()` | `rextendr::use_extendr()` via `prepare-r-package.R` |
| Native build system | R package build + platform C++ compiler | Cargo build invoked through generated rextendr package |
| Dependency pinning in spike | R package dependency declaration; R package resolution | Committed `Cargo.lock`; extendr API pinned to `=0.9.0` |
| Package output | Install the spike package directly | Generate disposable `.ci-r-package`, then install it |
| Scope boundary | Experiment only; no production R package edits | Experiment only; no production R package edits |

At the reviewed snapshot, both implementations produced the same scene-v1 shape, preserved missing matrix values as JSON `null`, emitted row-major cells, and used the same geometry/style constants. The C++ spike additionally validated dimensions and coefficient bounds. These statements describe historical prototype behavior only; the C++ implementation is retired and no longer part of the supported benchmark or test matrix.

## Historical local workflow (not current build instructions)

At the reviewed snapshot, the C++/Rcpp candidate was generated with `Rcpp::compileAttributes()` and installed as an ordinary R package; the Rust/rextendr candidate was generated into a disposable package and installed from that generated tree. The C++ source tree and its dedicated workflow were removed by PR #332, so the former C++ commands are intentionally not reproduced here and cannot be run on current `main`.

The disposable Rust/rextendr adapter remains available for evaluation. Its current setup and package-generation instructions are maintained in the [Rust/rextendr spike README](../prototypes/rust-rextendr/README.md). Do not treat this historical report as a production R-package installation guide.

## CI and platform comparison

### Historical shared CI at the reviewed snapshot

The cross-language workflow was manually dispatched (`workflow_dispatch`), not an ordinary pull-request gate. At the snapshot reviewed, its `cross-language` and optional `package-smoke` jobs installed and compared the C++/Rcpp and Rust/rextendr candidates alongside Python and pure R. That description is historical: PR #332 subsequently removed the C++ implementation and its dedicated workflow. The active benchmark harness is being reconciled separately; do not infer current support for the retired candidate from this archived comparison.

This is the current symmetric cross-platform evidence for both spikes. Because the workflow is manual and the package matrix is opt-in, it should not be described as continuous PR protection.

### Historical additional CI specific to Rust

| Workflow | Trigger | Platforms | Additional maintenance it exercises |
|---|---|---|---|
| `rust-rextendr-offline-package.yml` | PR paths for the Rust spike/workflow, plus manual dispatch; also configured for the named experiment branch | Ubuntu, macOS, Windows | Generated package install, locked vendoring, offline Cargo build, actual source tarball, `R CMD check --as-cran`, license notices |
| `rust-cran-dependency-audit.yml` | PR on Rust dependency inputs; weekly schedule; manual dispatch | Ubuntu | `cargo metadata --locked`, `cargo-deny`, dependency inventory, vendoring, offline build |
| `test-rust-windows-binary.yml` | PR paths for Rust spike/workflow and audit report; manual dispatch | Windows | Build binary ZIP, then install with a restricted PATH containing R/Windows system paths but no Rust toolchain; run R smoke test |

At the reviewed snapshot there was no dedicated C++-only workflow; the C++ spike was exercised by the shared manual benchmark/package-smoke workflow. PR #332 later retired the C++ spike and its dedicated CRAN workflow. The historical absence of C++ release checks is not a current coverage gap because C++/Rcpp is no longer a project candidate.

### Platform-specific concerns

| Platform | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Windows | Rtools must match the R toolchain for source builds; compiler/linker and Rcpp registration errors are the main debugging surface. A prebuilt R binary should avoid requiring users to install Rtools, but the current shared matrix alone does not prove that. | Rust stable target/linker setup must work in the package build; the dedicated workflow separately builds a binary ZIP and proves install/smoke-test with Rust removed from PATH. Source-package and offline-vendor path are also tested. |
| macOS | Xcode Command Line Tools/Clang and the local R SDK/compiler setup are required for source builds. | Stable Rust plus the platform linker are required; the offline package workflow includes macOS. |
| Linux | Distribution compiler toolchain and R headers are required for source builds. | Stable Rust plus linker/build tools are required; offline package workflow includes Linux. |
| All | Generated Rcpp registration files must agree with annotated C++ exports; compiler diagnostics and ABI/toolchain mismatch are the likely failure modes. | Generated package scaffolding, Cargo lockfile, vendor configuration, generated Makevars, and package archive contents must agree; dependency and build failures can occur before R-level tests run. |

The workflows use hosted runner labels rather than pinning exact compiler/Xcode/Rtools versions in this comparison. Runner images evolve; record tool versions in any production decision experiment before treating a passing run as a reproducible toolchain specification.

## Historical generated files and manual maintenance

| Concern | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Generated binding files | `R/RcppExports.R` and `src/RcppExports.cpp` from `Rcpp::compileAttributes()` (not present in the tracked prototype snapshot) | Generated package tree under `.ci-r-package/`, including rextendr scaffolding and generated native bindings |
| Source of truth | Annotated C++ exports and package metadata | Rust source, `Cargo.toml`, `Cargo.lock`, license notices, and `prepare-r-package.R` |
| Build artifacts | R package compilation output; generated wrappers | Cargo target output, generated package, vendored crate tree/archive, R source tarball |
| Manual review | Re-run attribute generation after changing exports; inspect generated diff; ensure Rcpp imports/namespace and native registration match | Review lockfile changes, license inventory/notices, vendor config and archive size; check generator changes against rextendr output; inspect source tarball and offline behavior |
| Dependency upgrade | Update Rcpp package requirement when needed; test compatible R/Rcpp/compiler versions | Deliberate crate upgrade, lockfile refresh, license-policy review, vendoring/offline audit, generated-package rebuild, and CRAN re-check |
| Release preparation | Standard R package source/binary build and CRAN checks; ensure generated bindings are present in the source package | Same R package release work plus Rust source/toolchain declaration, vendored source inclusion, offline Cargo configuration, crate notices, resource-bounded build and source-tarball verification |

The current Rust generator contains explicit compatibility adjustments around rextendr-generated Makevars and lockfile identity. That is a maintenance burden today, but it is also documented and automated rather than left as an undocumented manual recipe.

## Historical debugging and upgrade risks

### C++ / Rcpp

- **Stale generated bindings:** changing an Rcpp export without re-running `compileAttributes()` can leave the R wrapper/native registration out of sync.
- **Compiler and R ABI coupling:** source installation depends on a compiler toolchain compatible with the installed R version and platform.
- **Native diagnostics:** template/compiler errors and C++ ABI/linker failures can be more difficult to interpret than R-level errors.
- **Dependency behavior:** Rcpp is a mature R dependency, but the package still needs to state supported R/Rcpp versions and validate its compiler matrix.
- **Distribution proof missing:** the shared package-smoke matrix proves runner-side source installation; it does not prove a user can install a released Windows binary without Rtools or that a CRAN source tarball is self-contained.

### Rust / rextendr

- **Generated scaffold drift:** rextendr output can change across versions. The generator currently pins extendr-api 0.9.0 after scaffolding and makes targeted Makevars/lockfile adjustments; upgrades need explicit review.
- **Cargo dependency graph:** the lockfile makes the resolved graph reproducible, but each crate upgrade can change transitive dependencies, licenses, source archive size, or platform build behavior.
- **Offline packaging complexity:** Cargo vendoring and path/config handling must survive R package build/clean steps. The workflow currently performs archive extraction/restoration to verify the actual source tarball.
- **Toolchain and linker failures:** Rust compiler, target, static-library and linker diagnostics add a separate toolchain to investigate.
- **License obligations:** transitive crate notices need an auditable inventory and inclusion in the generated R package. The generator copies the pinned extendr MIT notice explicitly.
- **Windows install boundary:** source builds require Rust, while the Windows binary test is specifically designed to prove that installation of the built binary does not require Rust on the user's PATH.

## Historical maintainer-effort estimates (not measured)

These are planning estimates for one maintainer familiar with R packaging, inferred from the steps and scripts in the historical repository snapshot. They are not measured timings or CI wall-clock measurements. They exclude feature implementation and the shared scene-contract work. Issue #269 closed as not planned on 2026-11-04; the proposed human-effort comparison was discontinued and these ranges are not current decision evidence.

| Activity | C++ / Rcpp | Rust / rextendr | Basis |
|---|---:|---:|---|
| First local setup | 15–30 min | 30–60 min | Rcpp install + compiler vs. Rust toolchain + R dependencies |
| Regenerate, build, install after a small native change | 5–15 min | 10–25 min | One attribute-generation step vs. generated package script + Cargo build |
| Diagnose a typical binding/build failure | 15–60 min | 30–90 min | Rcpp wrapper/compiler surface vs. rextendr scaffold, Cargo, vendor and linker layers |
| Dependency upgrade with CI evidence | 15–45 min | 45–120 min | R package dependency/compiler validation vs. lockfile, license, vendor and offline checks |
| Prepare a release-grade source/binary validation | 1–2 h | 2–4 h | Standard R source/binary checks vs. additional vendor, offline archive, and license verification |

**Interpretation:** C++/Rcpp currently has lower expected maintenance cost for an R-only native extension. Rust/rextendr has a larger initial and release-validation surface, particularly because CRAN-style source distribution needs a self-contained Cargo dependency strategy. Rust may still be justified if the same core is genuinely reused by non-R consumers; that benefit is not measured by this issue and should not be assumed.

## Historical CI measurements from the shared-core proof

The PR #180 offline source-package workflow ran on all three operating systems.
From the completed GitHub Actions job logs, end-to-end job wall times were:

| Platform | Elapsed CI job time |
|---|---:|
| Ubuntu | 2 min 33 sec |
| macOS | 2 min 19 sec |
| Windows | 3 min 10 sec |

The Linux source tarball measured 1,270,634 bytes and the installed package
reported 6.0 MB; the Windows source tarball measured 1,260,465 bytes. These
are historical extendr 0.8.2 baseline measurements, superseded by the later
extendr 0.9.0 and build-time stripping evidence below. CI wall time is not
human maintainer effort and cannot be used as a substitute for it.
## Updated Rust/rextendr package measurements (2026-10-31)

PR #215 added opt-in build-time linker stripping to the disposable package generator. The three-platform offline workflow measured the following results on 2026-10-31:

| Platform | Installed package | Native library | Source tarball | R CMD check |
| --- | ---: | ---: | ---: | --- |
| Linux | 2,112,721 bytes | 2,100,280 bytes | 1,049,181 bytes | 1 NOTE |
| macOS | 1,805,047 bytes | 1,792,600 bytes | 1,048,308 bytes | 1 NOTE |
| Windows | 1,176,796 bytes | 1,164,288 bytes | 1,041,008 bytes | 1 NOTE |

Linux uses the GNU linker `--strip-debug` option, macOS uses Apple's `-S` option, and Windows remains unstripped because a PE/COFF approach has not been validated. The full offline install, R smoke test, source-tarball build, and `R CMD check --as-cran` passed on all three platforms.

The remaining NOTE reports Cargo's hidden `.cargo-checksum.json` and `.cargo_vcs_info.json` vendor metadata. These files are retained for offline dependency integrity and source provenance; they must not be deleted merely to silence the NOTE. The source archives are about 1.0 MiB on all three platforms. These are prototype measurements, not CRAN acceptance or production-package measurements. Obtain CRAN/maintainer guidance before any production R adoption.

## Discontinued maintainer-time comparison

The estimates above were not measured. No controlled human-effort comparison was performed. Issue #269 closed as not planned on 2026-11-04 after the project selected Rust as the sole native implementation and retired C++/Rcpp. Do not perform or report a C++/Rcpp maintainer-time comparison; do not infer human effort from CI wall-clock measurements.

## Evidence index

- [C++/Rcpp retirement PR #332](https://github.com/vsimko/corrplot-ng/pull/332) and the historical assessment preserved in this report. The retired source tree is not linked as a current repository path.
- [Rust spike README](../prototypes/rust-rextendr/README.md), [Cargo manifest](../prototypes/rust-rextendr/Cargo.toml), [R package generator](../prototypes/rust-rextendr/prepare-r-package.R), [integration smoke test](../prototypes/rust-rextendr/test-r-integration.R).
- [Shared cross-language workflow](../.github/workflows/cross-language-benchmarks.yml).
- [Rust offline package workflow](../.github/workflows/rust-rextendr-offline-package.yml), [dependency audit workflow](../.github/workflows/rust-cran-dependency-audit.yml), [Windows binary workflow](../.github/workflows/test-rust-windows-binary.yml).
- [CRAN Rust readiness review](cran-rust-readiness.md) and [dependency audit results](cran-rust-dependency-audit-results.md).
- [ADR 0001](adr/0001-shared-core-architecture.md), which selects Rust as the shared-core target while keeping production R migration gated by release evidence.
