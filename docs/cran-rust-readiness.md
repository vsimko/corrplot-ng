# CRAN readiness: Rust integration

**Status (2026-10-31):** the isolated `prototypes/rust-rextendr` scene-v1 spike has a reproducible, cross-platform source-package validation path. It is **not** the production R package and has **not** been accepted by CRAN. The current result is **PASS WITH TWO NOTEs / CONDITIONS**, not a clean CRAN check.

## Current validation evidence

The completed work for [issue #45](https://github.com/vsimko/corrplot-ng/issues/45) is recorded in the comparative review at [`cran-rust-cpp-comparison.md`](cran-rust-cpp-comparison.md) and the merged implementation PRs:

- [PR #55](https://github.com/vsimko/corrplot-ng/pull/55) established the three-platform vendored/offline source-package workflow.
- [PR #118](https://github.com/vsimko/corrplot-ng/pull/118) added the side-by-side C++ CRAN source-package checks and consolidated the current CRAN policy comparison.
- [PR #124](https://github.com/vsimko/corrplot-ng/pull/124) corrected NA-versus-NaN parity and reran the Rust source-package checks on the current scene-v1 prototype.

Latest source-package workflow: [run #41151817266](https://github.com/vsimko/corrplot-ng/actions/runs/41151817266). All Ubuntu, macOS, and Windows jobs passed. Each job:

1. Generates the temporary rextendr R package from the tracked prototype.
2. Checks the included extendr MIT license notice.
3. Resolves and audits the locked Cargo dependency graph and vendors the crate sources.
4. Installs the generated R package without network access.
5. Exercises the R-to-Rust function.
6. Builds and inspects the actual R source tarball, including the vendored inputs.
7. Runs `R CMD check --as-cran` on that tarball.

| Platform | Compressed source tarball | `R CMD check --as-cran` |
| --- | ---: | --- |
| Ubuntu | 1,270,392 bytes | 2 NOTEs |
| macOS | 1,264,958 bytes | 2 NOTEs |
| Windows | 1,260,318 bytes | 2 NOTEs |

The Linux installed package size was 5.9 MB. The macOS and Windows checks reported the installed-size check as OK.

The separate [Windows binary install run #41151817223](https://github.com/vsimko/corrplot-ng/actions/runs/41151817223) passed. It built a Windows binary package, transferred it to a fresh runner, removed Rust from the restricted `PATH`, installed the binary, and ran the R smoke test successfully.

## Remaining CRAN conditions

All three `R CMD check --as-cran` jobs report the same two NOTEs:

1. **Hidden files in the source package:** Cargo vendor metadata files `.cargo-checksum.json` and `.cargo_vcs_info.json` are present in the vendored source archive. They are part of the current offline Cargo packaging strategy and need a documented CRAN disposition rather than being removed blindly.
2. **Compiled-code NOTE:** the generated extendr integration references the non-API R entry point `R_NamespaceRegistry`. This needs review against the pinned extendr version and CRAN policy.

The CI workflow disables CRAN incoming and remote incoming checks. Therefore these results demonstrate reproducible source builds and package checks on the hosted platform matrix; they do not prove CRAN acceptance. Before a real submission, maintainers must resolve or explicitly justify both NOTEs, verify the minimum Cargo/rustc versions against the oldest supported CRAN build environments, and reassess package-size guidance with current CRAN/rextendr documentation.

## CRAN requirements relevant to production use

- Ship Rust source and all required third-party crate sources in the source package; do not fetch crates during installation.
- Use a committed lockfile and Cargo offline mode with a reproducible vendoring step.
- Declare Cargo/rustc system requirements and report toolchain versions.
- Keep Cargo parallelism resource-bounded.
- Include third-party copyright and license notices.
- Verify source-package checks on supported platforms and review all remaining warnings/NOTEs.
- Keep the production R package independent of Rust until a separate architecture decision authorizes adoption.

Canonical references:

- [CRAN Repository Policy](https://stat.ethz.ch/CRAN/web/packages/policies.html)
- [Using Rust in CRAN packages](https://cran.r-project.org/web/packages/using_rust.html)
- [Writing R Extensions](https://stat.ethz.ch/CRAN/doc/manuals/r-devel/R-exts.html)
- [rextendr CRAN compliance](https://rdrr.io/cran/rextendr/man/cran.html)

## Reproduction

From the repository root, with R, stable Rust/Cargo, and the required R packages installed:

```sh
Rscript prototypes/rust-rextendr/prepare-r-package.R
```

The complete vendoring, offline install, source-tarball inspection, and `R CMD check --as-cran` procedure is automated in `.github/workflows/rust-rextendr-offline-package.yml`. Use that workflow as the reproducible reference rather than hand-editing the generated `.ci-r-package` tree.

## Scope boundary

This is validation of an isolated prototype only. It does not authorize adding rextendr or Rust dependencies to `implementations/r`, does not make a CRAN submission, and does not select Rust as the production shared-core language. See [ADR 0001](adr/0001-shared-core-architecture.md) for the current language decision.
