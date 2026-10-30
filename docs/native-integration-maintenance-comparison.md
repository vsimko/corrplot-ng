# Maintainer workflow and CI cost: Rust/rextendr vs C++/Rcpp

**Issue:** [#113](https://github.com/vsimko/corrplot-ng/issues/113)  
**Snapshot reviewed:** `main` at `91981304c6f5509e919f2d086cb8d74e33bbe255`  
**Scope:** the isolated scene-v1 spikes in `prototypes/rust-rextendr/` and `prototypes/cpp-rcpp/`. This is a workflow comparison, not a language or production-architecture decision.

## Executive summary

Both candidates implement the same narrow R-callable scene-v1 JSON builder. Both can be generated, compiled, installed, and exercised by the existing cross-language workflow on Ubuntu, macOS, and Windows. That establishes basic integration feasibility, not equal release readiness.

- **C++/Rcpp has the shorter routine package workflow.** Rcpp attributes generate the R/C++ registration wrappers, then ordinary R package installation compiles the C++ source. Its main maintenance cost is keeping generated wrappers in sync and diagnosing the platform's C++ compiler/R toolchain.
- **Rust/rextendr has more build and distribution machinery.** The current package-generation script creates a temporary R package, pins the generated package to extendr 0.8.2, adapts the lockfile and generated Makevars, and adds R wrappers, help, namespace, and tests. Its dedicated CI also checks offline vendoring, source-tarball contents, dependency licenses, and a Windows binary install without Rust on PATH.
- **Current CI investment is asymmetric.** The shared manual cross-language workflow covers both candidates on all three operating systems, but the extra offline-package, dependency-audit, and Windows binary workflows currently cover Rust only. This is evidence of more Rust validation work, not proof that C++ has no corresponding release risks.
- **Neither spike is production-ready.** Keep both isolated until the same distribution and maintenance acceptance is applied to each.

## What is being compared

| Property | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Prototype entry point | `build_scene_json_cpp(labels, matrix, annotation)` | `build_scene_json_rust(labels, matrix, annotation)` |
| Native implementation | `prototypes/cpp-rcpp/src/scene_v1.cpp` | `prototypes/rust-rextendr/src/{lib.rs,scene_v1.rs}` |
| R package metadata | `prototypes/cpp-rcpp/DESCRIPTION`, Imports/LinkingTo Rcpp | `prototypes/rust-rextendr/DESCRIPTION`, generated package metadata |
| Binding generator | `Rcpp::compileAttributes()` | `rextendr::use_extendr()` via `prepare-r-package.R` |
| Native build system | R package build + platform C++ compiler | Cargo build invoked through generated rextendr package |
| Dependency pinning in spike | R package dependency declaration; R package resolution | Committed `Cargo.lock`; extendr API pinned to `=0.8.2` |
| Package output | Install the spike package directly | Generate disposable `.ci-r-package`, then install it |
| Scope boundary | Experiment only; no production R package edits | Experiment only; no production R package edits |

Both implementations produce the same scene-v1 shape, preserve missing matrix values as JSON `null`, emit row-major cells, and use the same geometry/style constants. The C++ spike additionally validates dimensions and coefficient bounds in its implementation. The Rust spike is an equivalent candidate behind the R binding. The shared parity harness is the appropriate place to judge semantic equivalence rather than comparing source-code size.

## Reproducible local workflow

Run these commands from the repository root. They describe the current repository scripts and package metadata; they are not a claim that every maintainer machine has already run them.

### C++ / Rcpp

**Prerequisites**

- Git and R 4.x.
- A working C++ compiler compatible with the local R installation: Rtools on Windows, Xcode Command Line Tools/Clang on macOS, or the distribution compiler toolchain on Linux.
- R package `Rcpp`.

**One-time setup**

```sh
R -q -e 'install.packages("Rcpp", repos = "https://cloud.r-project.org")'
```

**Generate wrappers, build, and install**

```sh
Rscript -e 'Rcpp::compileAttributes("prototypes/cpp-rcpp")'
R CMD INSTALL prototypes/cpp-rcpp
```

The attribute generator scans the C++ source annotations and creates the R wrapper and native registration source. In this checkout those generated `RcppExports` files are not present in the tracked prototype directories, so regeneration is part of the reproducible build workflow rather than a manual source edit.

**Exercise it**

After installation, call `corrplotngcppspike::build_scene_json_cpp()` with a small square numeric matrix and labels. For a fair comparison, use the shared fixtures and harness in `benchmarks/run_native_scene_v1.R` and `benchmarks/test_native_scene_v1.R`, not a hand-picked input alone.

### Rust / rextendr

**Prerequisites**

- Git and R 4.x.
- Stable Rust toolchain (`rustc` and `cargo`).
- R packages `rextendr`, `usethis`, `devtools`, and `testthat`; the shared benchmark workflow also installs `jsonlite`.
- A supported native linker/build toolchain for the host platform.

Install Rust using the official rustup installer for the operating system, then install the R dependencies:

```sh
R -q -e 'install.packages(c("rextendr", "usethis", "devtools", "testthat"), repos = "https://cloud.r-project.org")'
```

**Generate, build, and install**

```sh
Rscript prototypes/rust-rextendr/prepare-r-package.R
R CMD INSTALL prototypes/rust-rextendr/.ci-r-package
```

**Exercise it**

```sh
Rscript -e 'library(corrplotngrustspike); source("prototypes/rust-rextendr/test-r-integration.R")'
```

The generator deletes and recreates `.ci-r-package`. It calls `rextendr::use_extendr()`, pins the generated `extendr-api` dependency to 0.8.2, adapts the committed lockfile to the generated package identity, copies the Rust implementation, adjusts generated Makevars/target handling, and creates the R wrapper, namespace export, help page, and testthat smoke test. Do not hand-edit generated files as the source of truth; change the prototype or generator and regenerate.

**Additional release-oriented/offline path**

The dedicated workflow shows the extra steps currently needed to validate a self-contained source package:

1. Generate the temporary package.
2. Check the extendr license notice is included.
3. Resolve dependencies, enforce the license policy with `cargo-deny`, and vendor locked crates.
4. Configure Cargo offline mode and build with the lockfile.
5. Install the generated package without network access.
6. Build the actual R source tarball, restore the vendored source tree after the generated clean target, and inspect required archive entries.
7. Run `R CMD check --as-cran` on that tarball.

The exact workflow is `.github/workflows/rust-rextendr-offline-package.yml`. Its complexity is real and should be counted as maintainer work if Rust becomes the production route.

## CI and platform comparison

### Existing shared CI

`.github/workflows/cross-language-benchmarks.yml` is manually dispatched (`workflow_dispatch`), not an ordinary pull-request gate.

- The `cross-language` job runs on `ubuntu-latest`, installs Python 3.12, R, stable Rust, and the R packages Rcpp/jsonlite/rextendr/usethis. It exports shared fixtures, regenerates Rcpp wrappers, installs the C++ candidate, generates and installs the Rust candidate, and benchmarks both alongside Python/Pure R.
- The optional `package-smoke` job runs the same package build and semantic parity test on `ubuntu-latest`, `macos-latest`, and `windows-latest`. It is enabled only when the workflow input `package_smoke` is selected.
- The optional stress workloads are separate and do not change the routine package build path.

This is the current symmetric cross-platform evidence for both spikes. Because the workflow is manual and the package matrix is opt-in, it should not be described as continuous PR protection.

### Additional CI currently specific to Rust

| Workflow | Trigger | Platforms | Additional maintenance it exercises |
|---|---|---|---|
| `rust-rextendr-offline-package.yml` | PR paths for the Rust spike/workflow, plus manual dispatch; also configured for the named experiment branch | Ubuntu, macOS, Windows | Generated package install, locked vendoring, offline Cargo build, actual source tarball, `R CMD check --as-cran`, license notices |
| `rust-cran-dependency-audit.yml` | PR on Rust dependency inputs; weekly schedule; manual dispatch | Ubuntu | `cargo metadata --locked`, `cargo-deny`, dependency inventory, vendoring, offline build |
| `test-rust-windows-binary.yml` | PR paths for Rust spike/workflow and audit report; manual dispatch | Windows | Build binary ZIP, then install with a restricted PATH containing R/Windows system paths but no Rust toolchain; run R smoke test |

There is no dedicated C++-only workflow in the current workflow directory. C++ is exercised by the shared manual benchmark/package-smoke workflow. A production C++ package would still need explicit source-tarball checks, CRAN checks, and a clean Windows binary-install test; the absence of those jobs is a coverage gap, not a free pass.

### Platform-specific concerns

| Platform | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Windows | Rtools must match the R toolchain for source builds; compiler/linker and Rcpp registration errors are the main debugging surface. A prebuilt R binary should avoid requiring users to install Rtools, but the current shared matrix alone does not prove that. | Rust stable target/linker setup must work in the package build; the dedicated workflow separately builds a binary ZIP and proves install/smoke-test with Rust removed from PATH. Source-package and offline-vendor path are also tested. |
| macOS | Xcode Command Line Tools/Clang and the local R SDK/compiler setup are required for source builds. | Stable Rust plus the platform linker are required; the offline package workflow includes macOS. |
| Linux | Distribution compiler toolchain and R headers are required for source builds. | Stable Rust plus linker/build tools are required; offline package workflow includes Linux. |
| All | Generated Rcpp registration files must agree with annotated C++ exports; compiler diagnostics and ABI/toolchain mismatch are the likely failure modes. | Generated package scaffolding, Cargo lockfile, vendor configuration, generated Makevars, and package archive contents must agree; dependency and build failures can occur before R-level tests run. |

The workflows use hosted runner labels rather than pinning exact compiler/Xcode/Rtools versions in this comparison. Runner images evolve; record tool versions in any production decision experiment before treating a passing run as a reproducible toolchain specification.

## Generated files and manual maintenance

| Concern | C++ / Rcpp | Rust / rextendr |
|---|---|---|
| Generated binding files | `R/RcppExports.R` and `src/RcppExports.cpp` from `Rcpp::compileAttributes()` (not present in the tracked prototype snapshot) | Generated package tree under `.ci-r-package/`, including rextendr scaffolding and generated native bindings |
| Source of truth | Annotated C++ exports and package metadata | Rust source, `Cargo.toml`, `Cargo.lock`, license notices, and `prepare-r-package.R` |
| Build artifacts | R package compilation output; generated wrappers | Cargo target output, generated package, vendored crate tree/archive, R source tarball |
| Manual review | Re-run attribute generation after changing exports; inspect generated diff; ensure Rcpp imports/namespace and native registration match | Review lockfile changes, license inventory/notices, vendor config and archive size; check generator changes against rextendr output; inspect source tarball and offline behavior |
| Dependency upgrade | Update Rcpp package requirement when needed; test compatible R/Rcpp/compiler versions | Deliberate crate upgrade, lockfile refresh, license-policy review, vendoring/offline audit, generated-package rebuild, and CRAN re-check |
| Release preparation | Standard R package source/binary build and CRAN checks; ensure generated bindings are present in the source package | Same R package release work plus Rust source/toolchain declaration, vendored source inclusion, offline Cargo configuration, crate notices, resource-bounded build and source-tarball verification |

The current Rust generator contains explicit compatibility adjustments around rextendr-generated Makevars and lockfile identity. That is a maintenance burden today, but it is also documented and automated rather than left as an undocumented manual recipe.

## Debugging and upgrade risks

### C++ / Rcpp

- **Stale generated bindings:** changing an Rcpp export without re-running `compileAttributes()` can leave the R wrapper/native registration out of sync.
- **Compiler and R ABI coupling:** source installation depends on a compiler toolchain compatible with the installed R version and platform.
- **Native diagnostics:** template/compiler errors and C++ ABI/linker failures can be more difficult to interpret than R-level errors.
- **Dependency behavior:** Rcpp is a mature R dependency, but the package still needs to state supported R/Rcpp versions and validate its compiler matrix.
- **Distribution proof missing:** the shared package-smoke matrix proves runner-side source installation; it does not prove a user can install a released Windows binary without Rtools or that a CRAN source tarball is self-contained.

### Rust / rextendr

- **Generated scaffold drift:** rextendr output can change across versions. The generator currently pins extendr-api 0.8.2 after scaffolding and makes targeted Makevars/lockfile adjustments; upgrades need explicit review.
- **Cargo dependency graph:** the lockfile makes the resolved graph reproducible, but each crate upgrade can change transitive dependencies, licenses, source archive size, or platform build behavior.
- **Offline packaging complexity:** Cargo vendoring and path/config handling must survive R package build/clean steps. The workflow currently performs archive extraction/restoration to verify the actual source tarball.
- **Toolchain and linker failures:** Rust compiler, target, static-library and linker diagnostics add a separate toolchain to investigate.
- **License obligations:** transitive crate notices need an auditable inventory and inclusion in the generated R package. The generator copies the pinned extendr MIT notice explicitly.
- **Windows install boundary:** source builds require Rust, while the Windows binary test is specifically designed to prove that installation of the built binary does not require Rust on the user's PATH.

## Estimated maintainer effort

These are planning estimates for one maintainer familiar with R packaging, inferred from the steps and scripts in this repository. They are not measured timings or CI wall-clock measurements. They exclude feature implementation and the shared scene-contract work.

| Activity | C++ / Rcpp | Rust / rextendr | Basis |
|---|---:|---:|---|
| First local setup | 15–30 min | 30–60 min | Rcpp install + compiler vs. Rust toolchain + R dependencies |
| Regenerate, build, install after a small native change | 5–15 min | 10–25 min | One attribute-generation step vs. generated package script + Cargo build |
| Diagnose a typical binding/build failure | 15–60 min | 30–90 min | Rcpp wrapper/compiler surface vs. rextendr scaffold, Cargo, vendor and linker layers |
| Dependency upgrade with CI evidence | 15–45 min | 45–120 min | R package dependency/compiler validation vs. lockfile, license, vendor and offline checks |
| Prepare a release-grade source/binary validation | 1–2 h | 2–4 h | Standard R source/binary checks vs. additional vendor, offline archive, and license verification |

**Interpretation:** C++/Rcpp currently has lower expected maintenance cost for an R-only native extension. Rust/rextendr has a larger initial and release-validation surface, particularly because CRAN-style source distribution needs a self-contained Cargo dependency strategy. Rust may still be justified if the same core is genuinely reused by non-R consumers; that benefit is not measured by this issue and should not be assumed.

## Fair comparison gaps and recommended follow-up

The current CI is not fully symmetric. Before using this report to select a production language:

1. Add equivalent C++ evidence for building the source tarball and running `R CMD check --as-cran` on supported platforms.
2. Add a C++ Windows binary install/smoke test under a restricted PATH, analogous to the Rust workflow.
3. Run both package-smoke paths from clean checkouts and retain exact-head logs; the existing cross-language workflow is manually dispatched and its platform matrix is optional.
4. Record actual `R --version`, compiler, Rust/Cargo, Rcpp and rextendr versions in the evidence.
5. Measure wall-clock maintainer time for one controlled change (one exported argument or validation rule) and one dependency upgrade on each spike.
6. Compare compressed source package size and installation behavior on a clean user machine, not only on CI runners.

Until these are complete, the evidence supports a limited conclusion: **Rcpp is operationally simpler in the current prototype; Rust has a more thoroughly exercised source/offline and Windows-binary distribution path.** It does not establish the overall production winner.

## Evidence index

- [C++ spike README](../prototypes/cpp-rcpp/README.md), [DESCRIPTION](../prototypes/cpp-rcpp/DESCRIPTION), [implementation](../prototypes/cpp-rcpp/src/scene_v1.cpp).
- [Rust spike README](../prototypes/rust-rextendr/README.md), [Cargo manifest](../prototypes/rust-rextendr/Cargo.toml), [R package generator](../prototypes/rust-rextendr/prepare-r-package.R), [integration smoke test](../prototypes/rust-rextendr/test-r-integration.R).
- [Shared cross-language workflow](../.github/workflows/cross-language-benchmarks.yml).
- [Rust offline package workflow](../.github/workflows/rust-rextendr-offline-package.yml), [dependency audit workflow](../.github/workflows/rust-cran-dependency-audit.yml), [Windows binary workflow](../.github/workflows/test-rust-windows-binary.yml).
- [CRAN Rust readiness review](cran-rust-readiness.md) and [dependency audit results](cran-rust-dependency-audit-results.md).
- [ADR 0001](adr/0001-shared-core-architecture.md), which defers the production implementation-language choice.
