# CRAN readiness: Rust integration

**Status:** preliminary audit of the isolated `prototypes/rust-rextendr` spike. This is not a CRAN submission approval.

## Scope and current evidence

- The Rust spike is isolated from `implementations/r/`; the production R package does not currently import or link to Rust.
- The Windows binary-install workflow passed on commit `c2e0f1317e5072faafdc0cfa063ec13e75db3173`, including installation and smoke testing on a runner whose PATH was restricted to R and Windows system directories.
- That test proves a built Windows binary can be installed without a Rust toolchain. It does **not** prove that a source package can be built on CRAN infrastructure or that CRAN will accept the dependency packaging model.

## CRAN requirements relevant to a production Rust package

1. **Ship source, not prebuilt executable code.** CRAN source packages may not contain binary executable code. The package must contain the Rust sources, including third-party crate sources, or fetch an exact version from a secure and reliable location and verify it with a checksum. CRAN's Rust guidance strongly prefers vendoring.
2. **Declare toolchain requirements.** A Rust package should declare `SystemRequirements: Cargo (Rust's package manager), rustc`. Any minimum versions must be conservative and justified by the dependency graph. Configure scripts should locate both PATH and common per-user Rust installations and report versions.
3. **Keep builds resource-bounded.** Cargo's default parallelism can exceed CRAN's limit. Invoke Cargo with `-j 1` or `-j 2`.
4. **Avoid network-dependent installation.** CRAN advises avoiding downloads where possible; GitHub is not considered a sufficiently reliable long-term source for crate downloads. Prefer `cargo vendor` and include the vendored sources in the source tarball, compressed with xz.
5. **Account for all copyrights and licenses.** The R `DESCRIPTION` must identify authorship and copyright for Rust code and bundled Rust dependencies. Every crate's license and copyright notices must be preserved and compatible with the package's distribution rights.
6. **Use portable, static linking where required.** CRAN policy requires static libraries on Windows and macOS. The actual source package must build and pass checks on at least two major R platforms.
7. **Pass release checks.** Before submission, build a source tarball and run `R CMD check --as-cran` with current R-devel; warnings and significant notes need to be resolved or explained.

References:
- CRAN Repository Policy: https://stat.ethz.ch/CRAN/web/packages/policies.html
- Using Rust in CRAN packages: https://mirrors.nics.utk.edu/cran/web/packages/using_rust.html
- Writing R Extensions: https://www.stat.ethz.ch/CRAN/doc/FAQ/R-exts.html

## Findings in the current repository snapshot

| Area | Finding | Assessment |
|---|---|---|
| Production isolation | Rust remains under `prototypes/rust-rextendr/`; `implementations/r/DESCRIPTION` has no Rust dependency | Good boundary |
| Windows user install | Restricted-PATH binary install and smoke test pass | Proven for this prototype |
| Source package | Current CI generates a disposable package at build time; this is not yet a CRAN-ready source package | Not proven |
| Dependency lock | No committed `Cargo.lock` is present in the repository snapshot | Dependency set is not yet pinned for reproducible audit |
| Vendoring | No committed `cargo vendor` output or vendor archive is present | CRAN distribution requirement not yet addressed |
| System requirements | The production R package does not yet declare Cargo/rustc; the temporary generated package is only a CI fixture | Must be added if Rust enters production |
| Build parallelism | Current prototype build does not establish a CRAN-safe explicit Cargo job limit | Must be checked and constrained |
| License inventory | No lockfile/vendor tree exists to enumerate transitive crates and their licenses | Cannot complete yet |
| Maintainer metadata | The current production DESCRIPTION contains a placeholder maintainer address | Must be replaced before any real CRAN submission |

## Required next experiment

Do not add Rust to the production package yet. First make a self-contained source-build experiment on a separate branch:

1. Commit a lockfile generated from the actual Rust package.
2. Generate a complete crate/license inventory from that lockfile; flag unknown, custom, or incompatible licenses for manual review.
3. Vendor the locked crates with `cargo vendor`, and configure Cargo to use only the vendored directory during package compilation.
4. Add a source-package CI job that builds from a clean checkout with network access disabled after dependencies are prepared.
5. Set Cargo parallelism to at most two jobs and capture the rustc/Cargo versions in the installation log.
6. Run Linux and Windows source builds, then `R CMD check`; add macOS when the initial source-build path is stable.
7. Measure the compressed source tarball size and confirm all license notices are included.

**Decision gate:** Rust can be reconsidered for production only after the vendored source package builds reproducibly on the supported R platforms, the dependency license audit is clean, and the CRAN check results are acceptable. The successful Windows binary-install test remains a separate, positive distribution result.
