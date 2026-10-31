# CRAN policy and release implications: Rust/rextendr vs C++/Rcpp

**Reviewed:** 2026-10-31  
**Scope:** CRAN source-package submission, source installation, generated/build artifacts, platform requirements, and binary installation. This is an evidence review, not a production-language decision.

## Executive summary

Both approaches are viable in principle for a CRAN package. Neither is a zero-toolchain source-install option: installing the package from source compiles native code. CRAN-hosted Windows/macOS binaries are built by CRAN's binary infrastructure and install without the end user's compiler toolchain.

The material difference is the additional Rust toolchain and Cargo supply-chain/build process. C++ through Rcpp follows R's established compiled-package path and has the lower CRAN operational risk. Rust through rextendr is feasible, but the package maintainer must deliberately satisfy the Rust-specific policy: vendor all crate sources, compile offline, declare Cargo/rustc, detect and report toolchain versions, constrain Cargo parallelism, and keep the minimum Rust/Cargo versions compatible with older CRAN build environments.

**Preliminary assessment:** C++/Rcpp — PASS, lower release risk. Rust/rextendr — PASS WITH CONDITIONS, higher release and maintenance risk. These are assessments of policy fit, not a recommendation of the project's production core language.

## 1. CRAN rules that apply to both options

CRAN submissions are source tarballs built with `R CMD build`; the maintainer must run `R CMD check --as-cran` on the actual tarball. Source packages must contain source for their components and must make copyright, authorship and licensing clear. Source packages may not contain binary executable code.

The R extension mechanism compiles native source during a source installation. The resulting installed package contains a platform-specific shared library/DLL. CRAN's Windows and macOS binaries are prepared by CRAN's binary maintainers, not uploaded by package authors. Users installing those binaries do not compile the package locally; users installing the source tarball need the applicable native build toolchain.

For external libraries, CRAN policy requires static libraries on Windows and macOS. The package must also obey the platform-specific R toolchain conventions and pass checks on the supported CRAN platforms.

References:
- CRAN Repository Policy: https://stat.ethz.ch/CRAN/web/packages/policies.html
- Writing R Extensions (R 4.7.0 development manual, reviewed 2026-10-31): https://stat.ethz.ch/CRAN/doc/manuals/r-devel/R-exts.html
- R Installation and Administration: https://stat.ethz.ch/CRAN/doc/manuals/R-admin.html

## 2. C++ through Rcpp

### Source-package build requirements

- R's normal compiled-package toolchain: a C/C++ compiler, linker, make, R headers, and the platform's supported R build environment.
- On Windows this means the matching Rtools toolchain; on macOS the supported Apple/Xcode command-line toolchain; on Linux the distribution's compiler and development tools.
- Declare Rcpp in both `Imports` and `LinkingTo`: `Imports` makes the runtime R package available, while `LinkingTo` supplies headers for compilation.
- Put C++ sources and headers in `src/`. Use R's configured compiler variables and explicitly select a portable C++ standard through the package build configuration where needed; do not rely on compiler defaults.
- If third-party native libraries are added, account for their source, licensing, portability, and CRAN's static-library requirement on Windows/macOS.

### Generated and maintained artifacts

- `Rcpp::compileAttributes()` generates `src/RcppExports.cpp` and `R/RcppExports.R` from annotated C++ exports. These are source files, not compiled binaries, and should be regenerated and kept in sync when exported signatures change.
- `DESCRIPTION` records `Imports: Rcpp` and `LinkingTo: Rcpp`; `NAMESPACE` registers the native library and imports the Rcpp registration helper as appropriate.
- `src/Makevars` (and only when genuinely required, platform-specific variants such as `Makevars.win` or `Makevars.ucrt`) carries portable build settings. Avoid custom makefiles and hard-coded machine paths.
- Build outputs such as object files and shared libraries are generated during installation and must not be committed as package source.

### Release implications

Rcpp is a mature, widely used CRAN package with an established client-package workflow and broad downstream use. It is itself compiled, so source installation of Rcpp and of a package using it still depends on the normal R native toolchain. A package using Rcpp does not need a separate Rust-like toolchain or dependency-vendoring step.

**Assessment: PASS — lower incremental CRAN risk.** Main risks are ordinary C++ portability, compiler-standard compatibility, generated-wrapper drift, and any non-CRAN native library dependencies.

References:
- Rcpp CRAN package page (version 1.1.2, published 2026-07-05): https://CRAN.R-project.org/package=Rcpp
- Rcpp package vignette / reference manual: https://cran.r-project.org/web/packages/Rcpp/vignettes/Rcpp-package.pdf
- Writing R Extensions, C/C++ compilation and `LinkingTo`: https://stat.ethz.ch/CRAN/doc/manuals/r-devel/R-exts.html
- Rcpp package skeleton and generated exports: https://cran.nics.utk.edu/cran/web/packages/Rcpp/refman/Rcpp.html

## 3. Rust through extendr/rextendr

### Rust-specific CRAN requirements

CRAN's Rust guidance and the current rextendr CRAN-compliance documentation require the package to:

1. Include the Rust source code and all dependency sources in the source package, normally using Cargo vendoring. Do not rely on fetching crates from the network during package installation.
2. Compile offline from the submitted source package.
3. Declare the toolchain in `DESCRIPTION`, including `SystemRequirements: Cargo (Rust's package manager), rustc`.
4. Check for `cargo` and `rustc` in both the normal `PATH` and common user toolchain locations such as `~/.cargo/bin`; check required versions conservatively.
5. Tell users to install the toolchain when it is missing. The package must not install Rust or bootstrap rustup itself.
6. Limit Cargo build parallelism to 1 or 2 jobs; Cargo's default based on logical CPU count can exceed CRAN's allowed resource use.
7. Report the Rust compiler version in installation output, especially on failures.
8. Include accurate authorship and copyright information for bundled crates and their source licenses.

The CRAN Rust guidance recommends testing with older Cargo versions because CRAN's Linux check machines may use distribution-provided toolchains that lag upstream. It also warns that a vendored source archive can materially increase package size. The general CRAN policy says source tarballs should, if possible, not exceed 10 MB; it explicitly prefers bundling third-party source over downloading it at installation and allows a modestly increased limit to be requested at submission. Separately, rextendr 0.5.0 warns that a built package larger than 5 MB may be rejected and recommends explaining the vendored archive in `cran-comments.md`. These are distinct thresholds/guidance: 10 MB is the general source-tarball guideline, while 5 MB is the rextendr-specific warning about the built package. Measure both the source tarball and built package for the actual prototype and seek CRAN guidance if either threshold is exceeded.

### Build tools and artifacts

- In addition to R's native build environment, source installation requires compatible `cargo` and `rustc`.
- The rextendr scaffold supplies the package-side integration/build machinery, but its CRAN documentation explicitly leaves vendoring to the package author via `vendor_crates()`.
- Expect a Cargo manifest (`Cargo.toml`) and lockfile (`Cargo.lock`), Rust sources under the package's Rust source directory, the vendored crate sources (commonly archived as an xz-compressed vendor bundle), and Cargo configuration that redirects builds to the vendored sources in offline mode.
- The package integration also needs the rextendr-generated R/native registration wrappers and build/configuration scripts. Keep generated wrapper files synchronized with the Rust module and keep all generated files textual source; compiled objects and shared libraries are build outputs, not source-package contents.
- The exact scaffold filenames and layout are version-dependent. Pin and inspect the selected rextendr release/template before adopting a file checklist; do not copy paths from an older template blindly.
- Test the oldest supported Cargo/rustc pair, Linux CRAN check-farm assumptions, Windows, and macOS. Validate static-link behavior for any external native libraries on Windows/macOS.

### Release implications

Rust adds a second compiler/package manager to the source-install contract. This can be acceptable, but it raises the probability of source-install failures on older distributions, creates an explicit MSRV/Cargo compatibility policy, adds vendored-source maintenance and licensing work, and increases the source tarball. Offline builds and reproducible lock/vendor state must be release gates, not best-effort developer conveniences.

Prebuilt CRAN binaries avoid asking end users to install Rust, because they install the already-built package binary. However, this does not remove the Rust toolchain requirement from CRAN's own source checks/builds, from users installing source packages, or from platforms for which a CRAN binary is unavailable.

**Assessment: PASS WITH CONDITIONS — technically feasible, higher release/maintenance risk than Rcpp.** No policy blocker was found, but feasibility depends on conservative toolchain requirements, complete vendoring, offline builds, platform checks, and a clear plan for the larger source tarball.

References:
- CRAN, Using Rust in CRAN packages (guidance page last simplified 2023-07-13): https://cran.r-project.org/web/packages/using_rust.html
- rextendr CRAN-compliance documentation (rextendr 0.5.0 documentation built 2026-04-18): https://rdrr.io/cran/rextendr/man/cran.html
- rextendr CRAN package page (0.5.0): https://CRAN.R-project.org/package=rextendr
- rextendr project and scaffold documentation: https://github.com/extendr/rextendr

## 4. Source installation versus CRAN binary installation

| Concern | Source tarball | CRAN Windows/macOS binary |
|---|---|---|
| What is distributed | Package source, including native source and required bundled source dependencies | Installed package contents, including the compiled shared library/DLL |
| Who compiles | The installer's machine | CRAN's platform binary build infrastructure |
| C++/Rcpp user requirement | R native build toolchain and C++ compiler | No local C++ compiler for the package install |
| Rust/rextendr user requirement | R native build toolchain plus compatible Cargo and rustc | No local Rust compiler for the package install |
| Network during package compilation | Should not be required; vendored sources are preferred | Not needed by the end user to install the already-built binary |
| Availability | General source distribution, including unsupported or new platforms | Only platforms/architectures for which CRAN publishes binaries |

CRAN does not accept package-maintainer uploads of Windows/macOS binaries. CRAN's binary maintainers build and publish them after source-package acceptance and successful platform checks. Binary availability can lag the source publication.

## 5. Side-by-side assessment

| Dimension | Rust/rextendr | C++/Rcpp |
|---|---|---|
| CRAN policy status | Pass with explicit Rust-specific conditions | Pass through established native R package path |
| Source build toolchain | R toolchain + Cargo + rustc | R toolchain + C++ compiler |
| Offline source build | Mandatory; vendor all crates | Normally uses package sources and declared dependencies; avoid network fetches |
| Generated files | rextendr wrappers/configuration plus Cargo metadata and vendor sources; exact layout template-version dependent | `RcppExports.cpp`, `RcppExports.R`, NAMESPACE registration |
| Toolchain version risk | Rust/Cargo MSRV must fit older CRAN systems | C++ standard/compiler support must fit R's platform matrix |
| Windows/macOS | Must verify Rust target/linking and static external-library constraints | Standard Rtools / Apple toolchain; static external libraries required |
| Source tarball size | Potentially larger due to vendored crates | Usually smaller unless external source libraries are bundled |
| Maintainer workload | Higher: vendor refresh, Cargo compatibility, Rust logs, offline checks | Lower incremental burden for a conventional Rcpp package |
| Prebuilt binary user experience | No Rust installation needed for CRAN binary | No C++ installation needed for CRAN binary |

## 6. Project-specific implications and open questions

The current R package at `implementations/r` is still a provisional compatibility harness. Its `DESCRIPTION` currently has no `Rcpp` or `rextendr` dependency and no `SystemRequirements` declaration. The repository's Rust CLI decision in issue #98 is explicitly separate from the R integration decision; it must not be treated as evidence that the R package has selected Rust.

Before the parent issue #12 makes a production decision, the maintainers should confirm:

- Whether the intended shared core is compiled into the R package at all, or whether the R package will call a separately installed CLI/process. The latter has different CRAN and portability implications and is not evaluated as an equivalent in-process Rcpp/rextendr binding here.
- The exact rextendr version/template and generated file layout to pin if Rust remains under consideration.
- The minimum supported R, Cargo, and rustc versions and how those intersect with CRAN's Linux check machines and Windows/macOS toolchains.
- The measured source-tarball size and built-package size for the actual vendored Rust prototype; explain the vendor archive in `cran-comments.md` and request CRAN guidance if the relevant size guidance is exceeded.
- Whether all Rust crates and C++ dependencies have licenses compatible with CRAN distribution and the project's MIT licensing.
- Whether any native dependencies can be built from bundled source and meet the Windows/macOS static-library requirement.
- Whether the project can maintain the Rust toolchain and vendoring process over the package's long support lifetime.

## 7. Recommended validation before a final language decision

For each prototype, build a real source tarball and test the same minimal R-facing function through the actual CRAN installation path.

1. Run `R CMD build` and `R CMD check --as-cran` on the resulting tarball.
2. Install from source in clean Linux, Windows, and macOS environments, recording every compiler/toolchain version.
3. For Rust, test with vendored dependencies and network disabled; verify Cargo uses at most two jobs and does not silently update dependencies.
4. For C++, test the declared C++ standard against the oldest supported compiler on the CRAN platform matrix.
5. Install CRAN-like prebuilt binaries in clean user environments without the native toolchain to confirm the expected end-user experience.
6. Record source tarball size, build time, failure modes, and maintenance steps. Keep this comparison separate from runtime performance and scene-builder parity, which are covered by the other child issues.

## 8. Conclusion for issue #111

Both options can satisfy CRAN requirements. Rcpp is the simpler and lower-risk CRAN route for an in-process R native extension. Rust/rextendr remains a viable option, but only with a consciously maintained Rust/Cargo compatibility range, fully vendored and offline builds, explicit toolchain diagnostics, platform validation, and a source-tarball-size plan.

This issue does not select the production implementation language. The comparison should be combined with the maintainer/CI evidence, scene-builder parity/performance, and R-independence evidence from the sibling issues before ADR 0001 is finalized.

---

*Review date: 2026-10-31. CRAN policy and toolchain requirements can change; re-check the linked canonical guidance immediately before the first submission. The current rextendr 0.5.0 scaffold details above are based on its published package-development article and may change in later releases.*


## 9. Prototype validation evidence (2026-10-31)

The repository already contains useful implementation evidence that supplements the policy review:

### Rust/rextendr source-package check

The merged PR [#55](https://github.com/vsimko/corrplot-ng/pull/55) expanded the disposable rextendr package validation to Ubuntu, macOS, and Windows. Workflow run [#49](https://github.com/vsimko/corrplot-ng/actions/runs/41118035720) completed all three jobs successfully. Each job generated the rextendr package, audited and vendored its dependencies, installed it with network access disabled, exercised the R-to-Rust call, built and inspected the source tarball, and ran `R CMD check --as-cran`.

Measured source tarball sizes were approximately 1.26 MB compressed:
- Ubuntu: 1,270,268 bytes
- macOS: 1,264,644 bytes
- Windows: 1,259,971 bytes

The Ubuntu check reported an installed package size of 5.9 MB. All three platform checks completed with **2 NOTEs**, not a clean NOTE-free result:
- hidden Cargo vendor metadata files (`.cargo-checksum.json` and `.cargo_vcs_info.json`) in the source package;
- a compiled-code note for the non-API R entry point `R_NamespaceRegistry`.

The workflow disables CRAN incoming and remote incoming checks for deterministic CI, so this is not a substitute for a real CRAN submission or remote incoming checks. The 5.9 MB installed-size measurement should be assessed against rextendr's package-size warning; it is not the compressed source-tarball size. The two NOTEs need explicit disposition before a release candidate is considered ready.

### C++/Rcpp source-package check

The merged PR [#38](https://github.com/vsimko/corrplot-ng/pull/38) added a three-platform smoke matrix. Workflow run [#16](https://github.com/vsimko/corrplot-ng/actions/runs/41061324489) successfully generated Rcpp wrappers, installed the C++/Rcpp prototype from source, installed the Rust/rextendr prototype, and verified semantic parity against all five shared scene-v1 fixtures on Ubuntu, macOS, and Windows.

The dedicated source-package workflow added in this PR, [run #2](https://github.com/vsimko/corrplot-ng/actions/runs/41147928422), completed successfully on Ubuntu, macOS, and Windows. On each platform it generated the Rcpp wrappers, built the source tarball, and completed `R CMD check --as-cran`. CRAN incoming and remote incoming checks are disabled in this deterministic CI workflow; this is not a substitute for actual CRAN submission or remote incoming checks.

### Updated interpretation

- Rust/rextendr: source-tarball build, offline install, R integration, and `R CMD check --as-cran` have run on all three platforms. Status is **PASS WITH NOTES / CONDITIONS**, not a clean pass. Resolve or justify both notes and confirm the installed-size policy interpretation.
- C++/Rcpp: source installation, semantic parity, source-tarball build, and `R CMD check --as-cran` passed on all three hosted platforms. This closes the prototype's basic CRAN source-package validation gap, but does not establish CRAN acceptance or oldest-toolchain compatibility.
- Neither prototype is the production R package. These results do not establish CRAN acceptance, binary publication, oldest-toolchain compatibility, or final language choice.


### Latest confirmation on PR #124 (2026-10-31)

The merged [PR #124](https://github.com/vsimko/corrplot-ng/pull/124) reran the generated Rust source-package path on all three hosted platforms. [Workflow run #41151817266](https://github.com/vsimko/corrplot-ng/actions/runs/41151817266) passed all three jobs, including dependency vendoring/audit, offline installation, R integration, source-tarball creation and inspection, and `R CMD check --as-cran`.

| Platform | Compressed R source tarball | `R CMD check --as-cran` |
| --- | ---: | --- |
| Ubuntu | 1,270,392 bytes | 2 NOTEs |
| macOS | 1,264,958 bytes | 2 NOTEs |
| Windows | 1,260,318 bytes | 2 NOTEs |

The Linux installed package size was 5.9 MB; the macOS and Windows checks reported the installed-size check as OK. The two NOTEs on each platform are the same known items documented above: hidden Cargo vendor metadata files and use of the non-API R entry point `R_NamespaceRegistry`. They are not silently treated as a clean CRAN check; they still require maintainer/CRAN disposition before submission.

The separate [Rust Windows binary install run #41151817223](https://github.com/vsimko/corrplot-ng/actions/runs/41151817223) also passed: it built the binary package and installed/smoke-tested it on a fresh Windows runner with Rust removed from the restricted `PATH`.

These deterministic CI checks disable CRAN incoming and remote incoming checks. They demonstrate a reproducible prototype source-package path, not CRAN acceptance. The production R package remains independent of Rust.

### Updated Rust/rextendr evidence after extendr 0.9.0 (2026-10-31)

The current generated package pins `extendr-api = "=0.9.0"` and declares Rust/Cargo 1.78. The dedicated CI workflow verifies offline installation and `R CMD check --as-cran` on Ubuntu, macOS, and Windows, as well as Windows binary installation without Rust on the user's PATH.

The Ubuntu 24.04 / R 4.6.2 / GCC 13.3.0 job measured a compressed source tarball of **1,314,220 bytes** and an installed package size of **6.0 MB** (5.9 MB in `libs`). The source archive is below CRAN's general 10 MB guideline; the installed size exceeds rextendr's 5 MB caution and needs explicit CRAN guidance before any production R migration.

The latest Ubuntu `R CMD check --as-cran` result is **1 NOTE**. Extendr 0.9.0 removed the `R_NamespaceRegistry` non-API NOTE, and pruning unused vendored tests/examples/benchmarks removed the non-portable-path NOTE. The remaining NOTE lists Cargo's hidden `.cargo-checksum.json` and `.cargo_vcs_info.json` files. They are retained because the checksum manifest is required for offline Cargo integrity and the VCS metadata preserves source provenance; explain them in any CRAN submission comments. This is not evidence of CRAN acceptance.

Rust/Cargo 1.78 is tested for the core, CLI, and rextendr consumer. Controlled human maintainer time remains unmeasured; existing effort estimates are planning estimates only.
