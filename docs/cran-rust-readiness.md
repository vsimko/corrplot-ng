# CRAN readiness: Rust integration

**Status (2026-10-31):** the isolated `prototypes/rust-rextendr` scene-v1 spike has a reproducible, cross-platform source-package validation path. It is **not** the production R package and has **not** been accepted by CRAN. The latest recorded result is **PASS WITH ONE NOTE / EXTERNAL GUIDANCE REQUIRED**, not a clean CRAN check.

## Current validation evidence

The completed work for [issue #45](https://github.com/vsimko/corrplot-ng/issues/45) is recorded in the comparative review at [`cran-rust-cpp-comparison.md`](cran-rust-cpp-comparison.md) and the merged implementation PRs:

- [PR #55](https://github.com/vsimko/corrplot-ng/pull/55) established the three-platform vendored/offline source-package workflow.
- [PR #118](https://github.com/vsimko/corrplot-ng/pull/118) added the side-by-side C++ CRAN source-package checks and consolidated the current CRAN policy comparison.
- [PR #124](https://github.com/vsimko/corrplot-ng/pull/124) corrected NA-versus-NaN parity and reran the Rust source-package checks on the current scene-v1 prototype.
- [PR #184](https://github.com/vsimko/corrplot-ng/pull/184) records the extendr 0.9.0 / Rust 1.78 release-readiness evidence and the six successful associated CI workflows.

The latest recorded release-readiness evidence is the extendr 0.9.0 / Rust 1.78 validation summarized in [PR #184](https://github.com/vsimko/corrplot-ng/pull/184). All six associated CI workflows passed, including the cross-platform offline source-package checks and the Windows binary-install check. The workflows cover:

1. Generating the temporary rextendr R package from the tracked prototype.
2. Checking the included extendr MIT license notice.
3. Resolving and auditing the locked Cargo dependency graph and vendoring the crate sources.
4. Installing the generated R package without network access.
5. Exercising the R-to-Rust function.
6. Building and inspecting the actual R source tarball, including vendored inputs.
7. Running `R CMD check --as-cran` on that tarball.
8. Building and installing a Windows binary package on a clean runner without Rust on `PATH`.

| Measurement | Latest recorded result |
| --- | ---: |
| Compressed source tarball | 1,314,220 bytes |
| Installed package | 6.0 MB |
| Installed `libs` directory | 5.9 MB |
| Supported CI platforms | Ubuntu, macOS, Windows |
| `R CMD check --as-cran` | 1 NOTE: Cargo vendor metadata |

The platform matrix establishes reproducible build and installation behavior on the hosted runners. The recorded package-size values above are the measured package snapshot; they should not be interpreted as separately measured identical sizes on every platform.

## Remaining CRAN condition

The current remaining NOTE concerns hidden Cargo vendor metadata in the source package: `.cargo-checksum.json` and `.cargo_vcs_info.json`. The checksum manifests are required for Cargo to verify vendored sources during offline builds; the VCS metadata preserves source provenance. They are intentionally retained and must be explained in any CRAN submission comments. Do not remove them merely to silence the NOTE, because doing so would undermine the offline integrity/provenance workflow.

The earlier `R_NamespaceRegistry` compiled-code NOTE was removed by the extendr 0.9.0 update; it is not a current open condition. The previous two-NOTE platform table and the older 0.8.2 package-size measurements are superseded by the PR #184 evidence above.

The CI workflow disables CRAN incoming and remote incoming checks. Therefore these results demonstrate reproducible source builds and package checks on the hosted platform matrix; they do not prove CRAN acceptance. The installed package is 6.0 MB (5.9 MB in `libs`), above rextendr's 5 MB caution, so maintainers must obtain CRAN guidance before production R use.

## External questions requiring a maintainer response

No CRAN response or controlled human-effort measurement is recorded yet. The precise outstanding questions are:

1. **Vendor metadata NOTE:** Is retaining `.cargo-checksum.json` for offline Cargo integrity and `.cargo_vcs_info.json` for source provenance acceptable when both are explained in submission comments? If not, what packaging alternative does CRAN recommend that preserves verifiable offline builds?
2. **Installed package size:** Is a 6.0 MB installed package (5.9 MB in `libs`) acceptable for this use case despite rextendr's 5 MB caution? What evidence or mitigation would CRAN expect?
3. **Build toolchain:** Is Rust/Cargo 1.78 an acceptable minimum for the intended CRAN build environments, or should the project target a newer/older explicitly supported baseline?
4. **Submission checks:** Before submission, should the project enable or run any additional incoming/remote checks beyond the current hosted `R CMD check --as-cran` matrix?

Record the date, contact or guidance source, exact package/toolchain snapshot, question, response, and resulting project decision. Until then, the NOTE and package-size disposition remain open; do not claim CRAN approval.

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
