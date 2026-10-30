# CRAN readiness: Rust integration

**Status:** preliminary audit of the isolated `prototypes/rust-rextendr` scene-v1 benchmark spike. This is not a CRAN submission approval.

## Scope and current evidence

- Rust remains isolated under `prototypes/rust-rextendr/`; the production R package does not import or link to Rust.
- The current prototype implements the renderer-neutral scene-v1 JSON builder used for cross-language comparison. It is no longer the earlier cell-count-only smoke test.
- `.github/workflows/test-rust-windows-binary.yml` builds a Windows binary package, then installs and smoke-tests it with a restricted PATH that excludes the Rust toolchain. This is useful evidence about binary installation, not proof of CRAN source-package readiness.
- `.github/workflows/cross-language-benchmarks.yml` includes an optional package-smoke matrix for Ubuntu, macOS, and Windows, as well as the cross-language benchmark workflow.

## CRAN requirements relevant to a production Rust package

1. **Ship source, not prebuilt executable code.** CRAN source packages may not contain binary executable code. Include the Rust sources and third-party crate sources, or fetch exact versions from a secure and reliable location and verify them with checksums. CRAN's Rust guidance strongly prefers vendoring.
2. **Declare toolchain requirements.** If Rust enters the production package, declare `SystemRequirements: Cargo (Rust's package manager), rustc`. Keep minimum versions conservative and justified by the dependency graph. Configure scripts should locate common Rust installations and report tool versions.
3. **Keep builds resource-bounded.** Cargo's default parallelism can exceed CRAN's limits. Constrain builds to at most two jobs, and preferably one where required by the build environment.
4. **Avoid network-dependent installation.** Prefer `cargo vendor` and configure Cargo to use only the vendored directory during package compilation. Include the vendored sources in the source tarball.
5. **Account for all copyrights and licenses.** Inventory every direct and transitive crate, preserve license and copyright notices, and manually review unknown, custom, or incompatible licenses.
6. **Use portable linking.** Verify static-linking requirements on Windows and macOS and build the actual source package on supported R platforms.
7. **Pass release checks.** Build a source tarball and run `R CMD check --as-cran` with current R-devel. Resolve warnings and significant notes or document why they are acceptable.

References:
- CRAN Repository Policy: https://stat.ethz.ch/CRAN/web/packages/policies.html
- Using Rust in CRAN packages: https://mirrors.nics.utk.edu/cran/web/packages/using_rust.html
- Writing R Extensions: https://www.stat.ethz.ch/CRAN/doc/FAQ/R-exts.html

## Findings in the current repository snapshot

| Area | Finding | Assessment |
|---|---|---|
| Production isolation | Rust remains under `prototypes/rust-rextendr/`; the production R package has no Rust dependency | Good boundary |
| Prototype scope | Rust implements the scene-v1 JSON builder used by the Python and C++/Rcpp candidates | Useful experiment; not production code |
| Windows binary install | A restricted-PATH binary-install and smoke-test workflow is present | Workflow coverage exists; source-package readiness is not established |
| Cross-platform builds | The cross-language workflow has an optional Ubuntu, macOS, and Windows package-smoke matrix | Coverage is defined; release-grade CRAN checks remain outstanding |
| Dependency lock | No committed `Cargo.lock` is present in the repository snapshot | Dependency set is not yet pinned for a reproducible audit |
| Vendoring | No committed `cargo vendor` output or vendor archive is present | CRAN distribution approach not yet addressed |
| System requirements | The production R package does not yet declare Cargo/rustc | Must be addressed only if Rust enters production |
| Build parallelism | The prototype does not yet establish a CRAN-safe explicit Cargo job limit | Must be checked and constrained |
| License inventory | No lockfile/vendor tree is available to enumerate transitive crates | Audit cannot be completed yet |
| Maintainer metadata | The production R package still has placeholder maintainer metadata | Must be replaced before any real CRAN submission |

## Required next experiment

Do not add Rust to the production package yet. First make the isolated source-build experiment self-contained:

1. Commit a lockfile generated from the current Rust prototype.
2. Generate a complete crate/license inventory from that lockfile; flag unknown, custom, or incompatible licenses for manual review.
3. Vendor the locked crates with `cargo vendor`, and configure Cargo to use only the vendored directory during package compilation.
4. Add a source-package CI job that builds from a clean checkout without network access after dependencies are prepared.
5. Set Cargo parallelism to at most two jobs and capture the rustc/Cargo versions in the build log.
6. Run Linux and Windows source builds, then `R CMD check`; keep macOS in the validation matrix.
7. Measure the compressed source tarball size and confirm all license notices are included.

**Decision gate:** reconsider Rust for production only after the vendored source package builds reproducibly on supported R platforms, the dependency license audit is clean, and CRAN checks are acceptable. The Windows binary-install test remains a separate distribution check.
