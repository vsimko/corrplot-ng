# Monorepo migration inventory

**Status:** Inventory completed; no source files moved.  
**Parent decision:** [ADR 0003 — Repository layout as a multi-ecosystem monorepo](adr/0003-monorepo-layout.md).  
**Inventory snapshot:** `main` at `45d296b59bf9570149d2efdfc4b7cd082adf0d0a` (2026-11-05), immediately after ADR 0003 was accepted.  
**Method:** GitHub recursive Git tree plus inspection of package manifests, lockfiles, maintainer scripts, current architecture/layout documentation, and all 18 workflow definitions. The recursive tree returned 448 entries and was not truncated.

This document is the source-to-target map required before structural moves. It records existing ownership and path coupling; it is not authorization to change APIs, package names, release gates, or runtime behavior.

## 1. Repository-wide inventory

| Current path | Current contents / owner | Target path | Required follow-up |
|---|---|---|---|
| `README.md`, `docs/**`, `CITATION.cff` | Repository entry point, architecture/ADRs, project documentation and citation metadata | Remain at repository root / `docs/**` | Update links and command examples after each move. Keep `CITATION.cff` at root for GitHub citation discovery and keep its validation workflow. |
| `.github/**` | Dependabot and all CI/release orchestration | Remain at `.github/**` | Update path filters, working directories, cache keys, artifact paths and checkout-relative commands in #371. |
| `.gitignore`, `.gitattributes` | Repository-wide ignore and line-ending rules | Remain at root | Expand root ignore rules for `node_modules/`, Cargo `target/`, Python environments/caches, generated R libraries and local build outputs. Preserve the LF rule for `experiments/wasm-r/fixtures/*.json` or relocate it with the experiment. |
| `.changeset/**` | Changesets configuration and pending npm release notes | `npm/.changeset/**` | Update the Changesets config, scripts and CI references together with the npm workspace move. |
| Root `package.json` | Private npm workspace; workspaces are `bindings/node`, `bindings/wasm` and `packages/*` | `npm/package.json` | Move workspace root and scripts; keep the root private and non-publishable. Generate the real `npm/package-lock.json` using the supported npm version before switching installs to `npm ci` (#265). |
| Root `requirements-dev.txt` | Repository Python test/schema-validation dependency (`jsonschema`) | `tools/repository/requirements-dev.txt` | Update foundation CI and `dev-check.sh`; do not create a Python environment at repository root. |
| `crates/corrplot-core/**` | Authoritative shared Rust semantics and capability declaration | `rust/crates/corrplot-core/**` | Move as part of the Rust workspace migration; update all Cargo path dependencies and offline/MSRV checks. |
| `tools/corrplot-ng/**` | Standalone Rust CLI, tests, CLI README and capability declaration | `rust/tools/corrplot-cli/**` | Keep CLI orchestration thin over the shared core. Update release workflow, install/smoke tests and user documentation. |
| `bindings/node/**` | Mixed npm package and Rust Node-API crate | Split between `npm/packages/native-node/**` and `rust/bindings/node/**` | Move JS entry points, declarations, npm README, package tests and npm packaging scripts to the npm package. Move `Cargo.toml`, `build.rs` and `src/lib.rs` to the Rust binding. Rework build scripts to resolve the Rust manifest and generated addon across the new boundary. |
| `bindings/node/npm/{linux-x64-gnu,darwin-x64,darwin-arm64,win32-x64-msvc}/**` | Four platform-constrained optional npm packages; deliberately not workspace members | `npm/packages/native-node-<platform>/**` (or an equivalently explicit non-workspace package directory under `npm/`) | Keep each package's `os`/`cpu` constraints and independent native-runner tarball build. Do not add them as ordinary cross-platform workspace members. |
| `bindings/wasm/**` | Mixed npm package, Rust WASM crate, browser examples, package preparation and measurements | Split between `npm/packages/wasm/**` and `rust/bindings/wasm/**` | Move JS package metadata, package scripts, browser example and package tests to npm; move Cargo manifest and Rust source to Rust. Keep `pkg/` generated, not committed. |
| `bindings/cross-backend/**` | Node/WASM parity and comparison harness | `npm/tests/cross-backend/**` | Keep parity coverage adjacent to the two npm runtime packages while ensuring it builds both Rust-backed adapters. |
| `packages/typescript/**`, `packages/react/**`, `packages/vue/**` | TypeScript facade and browser-only framework adapters | `npm/packages/{typescript,react,vue}/**` | Preserve package names, conditional exports, peer-dependency isolation, browser-only boundaries and package-level tests. Update all relative example/test paths. |
| `implementations/r/**` | Current R package: DESCRIPTION, NAMESPACE, R API, manuals, installed extdata and testthat suite | `r/corrplotng/**` | Move as a structural-only change. Keep R package metadata, documentation, tests and package copies of fixtures together. Preserve the current upstream-corrplot delegation and experimental scene path. |
| `prototypes/rust-rextendr/**` | Disposable Rust/rextendr evaluation adapter, generated-package preparation and integration tests | `rust/bindings/r/rextendr-prototype/**` | Keep clearly separate from the production R package. Update its Cargo dependency, generated R package workflow and CRAN/offline audit references. |
| `experiments/wasm-r/**` | One cross-runtime R-host experiment: Rust WASI/ABI/native-R crates, Node and R runners, shared fixtures, benchmark results and report | **Retain as `experiments/wasm-r/**` as a deliberate cross-ecosystem exception** | ADR 0003's target tree/root allowlist should explicitly mention this experiment area. Do not split its runners, fixtures and result provenance into unrelated ecosystem roots. Update its Rust core path and workflow filters only when moved. |
| `prototypes/normalized-matrix-v1/**` | Normative normalized-matrix schema, compatibility text, Python validator/tests and valid/invalid fixtures | Schema/spec to `contracts/normalized-matrix-v1/**`; canonical fixtures to `fixtures/normalized-matrix-v1/**`; Python validator/tests to `python/reference/**` or repository validation tooling | Keep one canonical schema and fixture set. Preserve R package copies only where required for installed-package tests and keep drift checks. |
| `prototypes/shared-svg/**` | Scene-v1 schema/spec, Python scene/SVG reference, capability declaration, examples and tests, including R consumers/oracle bridge | Scene contract to `contracts/scene-v1/**`; canonical examples to `fixtures/scene-v1/**`; Python reference to `python/reference/scene-svg/**`; cross-runtime checks to owning harnesses / `tools/repository/**` | Do not move the normative schema with the non-authoritative renderer as one opaque unit. Keep the Python renderer explicitly a reference implementation. |
| `prototypes/diagnostic-v1.schema.json`, `prototypes/renderer-capabilities-v1.schema.json` | Normative diagnostic and renderer-capability schemas | `contracts/diagnostics-v1/**`, `contracts/renderer-capability-v1/**` | Update Rust, npm, R and Python schema consumers together. Backend capability declarations remain backend-owned data, not duplicate schemas. |
| `prototypes/publication-config-v1/**`, `prototypes/publication-svg-v1/**` | Versioned publication configuration/SVG contracts plus Python prototype implementation and tests | Schemas/specs to `contracts/publication-config-v1/**` and `contracts/publication-svg-v1/**`; Python implementation to `python/reference/**` | Separate normative contract assets from executable reference code. |
| `prototypes/**` other than the paths above | Mixed schemas, Python reference code, fixtures, examples and experiments | Split by ownership between `contracts/`, `fixtures/`, `python/reference/` and `rust/bindings/r/` | Do not bulk-move the whole directory. Inventory each remaining child and preserve the non-production status of prototypes. |
| `tests/fixtures/**` | Shared correlation/general-matrix fixtures and a publication example | `fixtures/**` (group by contract/version) | Preserve canonical bytes and update the fixture integrity test and every R/Python/CLI consumer. |
| `tests/legacy-visual/**` | Canonical legacy visual-case manifest and documentation | `fixtures/legacy-visual/**` | Keep the R package's installed/test copy under `r/corrplotng/tests/**`; retain exact-copy verification. |
| `tests/*.py`, `tests/README.md` | Repository-level Python fixture integrity, benchmark and SVG validation tests | `tools/repository/tests/**` (with benchmark tests under its benchmark subdirectory where appropriate) | Keep runtime-specific tests with their package. Update the Python test discovery commands and dependency installation. |
| `benchmarks/**` | Mixed Python, R and cross-language benchmark drivers, fixture exporters, result comparators and tests | Cross-runtime orchestration/comparison under `tools/repository/benchmarks/**`; Python-reference-specific drivers under `python/reference/benchmarks/**`; R-specific harnesses under the appropriate R/R-binding experiment area | Split by actual imports and responsibility, not filename extension alone. Preserve manual/stress opt-in behavior and artifact schemas. |
| `scripts/dev-check.sh` | Repository-wide local command entry point | `tools/repository/dev-check.sh` | Keep it as orchestration only; delegate to owning ecosystem roots and document all working directories. |
| `scripts/validate_svg.py` | Standard-library SVG artifact validator shared by CI and local commands | `tools/repository/validate_svg.py` | Update CLI/benchmark/foundation invocations. |
| `scripts/update-legacy-visual-snapshots.R` | R-only maintainer utility for regenerating legacy visual snapshots | `r/corrplotng/tools/update-legacy-visual-snapshots.R` | Keep the explicit environment-variable confirmation and pinned corrplot version check. |
| `docs/**` | Architecture, compatibility, benchmark, CRAN and npm distribution documentation | Remain under `docs/**` | Update links and command paths in the same PR as each owning ecosystem move. Reconcile the current layout page's reference to a root `.cargo/` directory: no tracked `.cargo/` path exists in this snapshot. |

## 2. Manifest and lockfile inventory

| Current file | Type / purpose | Target / decision |
|---|---|---|
| `package.json` | Private npm workspace root; workspaces `bindings/node`, `bindings/wasm`, `packages/*`; Node 22/24/26, npm >=10 | `npm/package.json`; one workspace lockfile is required by ADR 0003. |
| `bindings/node/package.json` | `@corrplot-ng/native-node`; optional dependencies on four platform packages | `npm/packages/native-node/package.json`. |
| `bindings/node/npm/*/package.json` (4) | Linux GNU x64, macOS x64/arm64, Windows x64 MSVC packages | Four separate non-workspace package manifests under `npm/`; retain platform metadata. |
| `bindings/wasm/package.json` | `@corrplot-ng/wasm`, generated ESM/WASM package | `npm/packages/wasm/package.json`. |
| `packages/{typescript,react,vue}/package.json` (3) | Facade and browser framework package manifests | `npm/packages/{typescript,react,vue}/package.json`. |
| `crates/corrplot-core/Cargo.toml` | Shared core crate; Rust 1.78 MSRV | `rust/crates/corrplot-core/Cargo.toml`. |
| `tools/corrplot-ng/Cargo.toml` | CLI crate; path dependency on shared core | `rust/tools/corrplot-cli/Cargo.toml`. |
| `bindings/node/Cargo.toml`, `bindings/wasm/Cargo.toml` | Rust binding crates; both use relative path dependency `../../crates/corrplot-core` | `rust/bindings/{node,wasm}/Cargo.toml`; rewrite to the new workspace-relative core path. |
| `prototypes/rust-rextendr/Cargo.toml` | Disposable rextendr crate; path dependency `../../crates/corrplot-core` | `rust/bindings/r/rextendr-prototype/Cargo.toml`; preserve the pinned extendr version. |
| `experiments/wasm-r/{native-r,wasm-abi,wasm-host}/Cargo.toml` (3) | Experiment crates; each path-depends on `../../../crates/corrplot-core` | Retain with the experiment or move as one experiment unit; rewrite all three paths if the experiment moves. |
| `crates/corrplot-core/Cargo.lock` | Committed lockfile for the current core crate (currently only the core package) | Replace with the single `rust/Cargo.lock` only when the Rust workspace is established and validated. |
| `tools/corrplot-ng/Cargo.lock` | Committed CLI lockfile | Consolidate into `rust/Cargo.lock`; preserve offline CLI builds. |
| `prototypes/rust-rextendr/Cargo.lock` | Committed prototype lockfile | Consolidate into the Rust workspace lockfile if the prototype becomes a workspace member; otherwise retain an explicit standalone lockfile. |
| Root `package-lock.json` | **Absent** | Open issue #265 requires a real npm-generated lockfile. Current npm CI uses `--package-lock=false`; do not hand-author a lockfile. |
| `pyproject.toml`, `uv.lock`, `poetry.lock`, `Pipfile` | **Absent** | Python currently has only root `requirements-dev.txt` declaring `jsonschema`. Decide a Python project/environment file when isolating the reference implementation; do not imply a distributable Python package. |
| `implementations/r/DESCRIPTION`, `NAMESPACE` | R package metadata | `r/corrplotng/DESCRIPTION`, `NAMESPACE`; no `renv.lock` exists. Do not add renv merely for symmetry. |
| Docker/Compose/Kubernetes manifests, Debian/RPM/Arch recipes, signing config | **None found in the tracked tree** | No packaging/deployment move is currently required. #374 should remain evidence-driven and must not create speculative empty packaging frameworks. |

There is no root `Cargo.toml` or root Cargo workspace in this snapshot. Rust is currently several independent crates with three committed Cargo lockfiles; Node/WASM bindings and the three WASM-R experiment crates have no committed lockfiles. The npm workspace is the only current root package-manager manifest, and its missing lockfile is tracked separately by #265.

## 3. Cross-runtime fixtures and generated artifacts

### Canonical fixture ownership

| Current canonical source | Current package/test copy | Target canonical location | Preservation rule |
|---|---|---|---|
| `prototypes/normalized-matrix-v1/fixtures/valid-cases.json` and `invalid-cases.json` | `implementations/r/inst/extdata/normalized-matrix-v1/**` | `fixtures/normalized-matrix-v1/**` | R copies remain package-local and byte-identical; keep `tests/test_fixture_integrity.py` coverage. |
| `tests/fixtures/general-matrix-cases.json` | `implementations/r/inst/extdata/general-matrix-cases.json` | `fixtures/normalized-matrix/**` | Preserve the documented R package projection that omits the canonical `all-missing` invalid render case. |
| `tests/fixtures/correlation-cases.json` | Consumers in foundation/legacy tests | `fixtures/correlation/**` | One canonical source; update consumers rather than creating divergent copies. |
| `tests/legacy-visual/cases.json` | `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json` plus PNG snapshots | `fixtures/legacy-visual/**` | Preserve exact manifest copy and snapshot provenance against corrplot 0.95. |
| `prototypes/shared-svg/*.example.json` | Python and R reference tests | `fixtures/scene-v1/**` | Keep versioned examples separate from the normative scene schema. |
| `experiments/wasm-r/fixtures/**` | Node and R experiment runners | Retain within `experiments/wasm-r/fixtures/**` | Experiment-specific fixtures remain tied to the benchmark harness and its artifact provenance. |
| `packages/typescript/test/fixtures/canvas-conformance.json` | TypeScript facade tests | `npm/packages/typescript/test/fixtures/**` | Package-specific conformance fixture remains with the TypeScript package. |

### Generated and transient outputs

The tracked tree contains no `node_modules/`, Cargo `target/`, `.venv/`, built Node addon, generated WASM `pkg/`, generated R library, vendor tree, or CI temporary build directory. These are generated by the owning workflows and must remain untracked. The current root `.gitignore` contains only `.idea/`; `bindings/wasm/.gitignore` and `tools/corrplot-ng/.gitignore` are local to those projects. Add explicit repository-wide generated-output ignores as part of #365, after confirming no intentional fixtures are hidden.

The release/build artifact hand-offs currently are:

- Node native prebuilt workflow: one platform tarball per matching runner, then artifact upload.
- npm staging workflow: four platform tarballs plus WASM, base Node, React, Vue and TypeScript tarballs (nine total), assembled with `SHA256SUMS` and `RELEASE-ORDER.txt`; it is explicitly non-publishing.
- Rust CLI release workflow: OS-specific binaries, package archives and SHA-256 checksums, followed by a versioned GitHub Release.
- WASM, cross-backend, R/rextendr and WASM-R benchmark workflows: run-scoped JSON/CSV/text artifacts, not source files.
- Rust CRAN audit workflow: vendored dependency archive, license inventory and metadata artifact; generated only in CI.

No Docker build context, deployment descriptor, SBOM/provenance file, package-signing configuration, signing key or secret is tracked at this snapshot. If such assets are introduced later, classify each by deployable component and owner before assigning it to `packaging/`; never commit signing keys or credentials.

## 4. Workflow and path-assumption inventory

All 18 current workflows are listed below. Workflows remain under `.github/workflows/`; this table identifies the path assumptions that #371 must update. The present workflows use a mixture of explicit path filters, internal changed-path detection, and unfiltered PR/push triggers.

| Workflow | Current source paths / behavior | Migration impact |
|---|---|---|
| `build-node-prebuilt-packages.yml` | Builds Rust Node addon from `bindings/node`; packages four native variants on matching OS runners | Split Rust crate and npm package roots; update Cargo working directory, package script, tarball staging and artifact paths. |
| `cross-language-benchmarks.yml` | Uses `benchmarks/**`, `prototypes/rust-rextendr/**`, R/Python scripts and fixtures; includes optional OS package smoke | Update each ecosystem command and fixture path; keep cross-language orchestration and manual/stress policy intact. |
| `release-rust-cli.yml` | Builds and publishes the CLI from `tools/corrplot-ng` on release/tag workflow | Move CLI manifest/source to `rust/tools/corrplot-cli`; update version check, build, smoke, archive and checksum steps. |
| `rust-cran-dependency-audit.yml` | Audits core plus `prototypes/rust-rextendr`; vendors Cargo dependencies and uploads license/provenance evidence | Move core/prototype manifests and path dependency; keep offline vendoring and CRAN license checks separate from npm. |
| `rust-rextendr-offline-package.yml` | Generates package via `prototypes/rust-rextendr/prepare-r-package.R`, installs offline, checks source tarball and uploads measurements | Move disposable adapter to `rust/bindings/r/rextendr-prototype`; update R package generator, source tarball and report paths. |
| `stage-npm-packages.yml` | Uses root npm workspace, `bindings/node`, `bindings/wasm`, `packages/react`, `packages/vue`, `packages/typescript`; assembles nine tarballs | Update workspace root, package working directories and native platform artifact hand-offs; preserve non-publishing permissions and checksum verification. |
| `test-foundation.yml` | Installs root `requirements-dev.txt`; runs `scripts/dev-check.sh`, root tests, `prototypes/**` tests and TypeScript facade tests; includes changed-path filtering | Move Python tooling/tests and facade; update path filter, requirements location and delegated command. |
| `test-node-native.yml` | Runs npm test/build/prebuilt package tests in `bindings/node` | Split Rust build root from npm package root; update cache and clean-consumer package paths. |
| `test-node-wasm-parity.yml` | Builds Node and Node-target WASM from `bindings/node` and `bindings/wasm`; runs `bindings/cross-backend/**` | Update both Rust manifests, npm package outputs and parity harness location. |
| `test-npm-workspace.yml` | Installs root workspace; path filter covers root `package.json`, `packages/**`, `bindings/node/**`, `bindings/wasm/**`, `.changeset/**` | Change to `npm/**` and include Rust binding paths where Rust source changes affect npm builds. Replace lockfile-free install only after #265 supplies the real lockfile. |
| `test-r.yml` | Checks `implementations/r`, legacy oracle and `prototypes/shared-svg` R consumer; internal changed-path filter | Move package root and R oracle; update check/build paths and filter. |
| `test-rust-cli.yml` | Tests `crates/corrplot-core` and `tools/corrplot-ng` with locked/offline Cargo commands; validates CLI fixtures | Update manifest paths, binary output location and shared-fixture paths. |
| `test-rust-windows-binary.yml` | Generates `prototypes/rust-rextendr` R package, builds Windows binary and installs with restricted PATH | Update adapter and generated R package paths; preserve restricted-PATH test. |
| `test-shared-rust-core.yml` | Compiles core, WASM target and `prototypes/rust-rextendr` consumer, including Rust 1.78 MSRV | Update workspace manifests and all path-dependency checks; keep MSRV coverage. |
| `test-wasm-binding.yml` | Builds `bindings/wasm`, npm workspace facade/React/Vue browser bundles, Playwright smoke and performance artifacts | Split npm and Rust paths; update package build and browser examples; retain Chromium/Firefox/WebKit coverage. |
| `test-wasm-r-experiment.yml` | Explicit path filter for `experiments/wasm-r/**`, `crates/corrplot-core/**` and itself; builds three experiment crates | Keep the experiment exception or update the filter and three Cargo path dependencies together. Preserve the nine-job manual matrix and artifact names. |
| `test-workflows.yml` | Runs actionlint over `.github/workflows/**` | Remains at root; must pass after every workflow path/filter edit. |
| `validate-citation.yml` | Validates root `CITATION.cff` | Remains root-scoped; no ecosystem move expected. |

Additional path-sensitive references are in `scripts/dev-check.sh`, package build/pack scripts, Cargo `path = "../../crates/corrplot-core"` dependencies, R package extdata paths, browser example Vite configs, package-isolation tests, and documentation links. These must be updated with the owning move, not in a later cleanup PR.

## 5. Distribution, deployment and security inventory

- **OS packaging:** no Debian `debian/` tree, RPM spec, Arch `PKGBUILD`, or distribution package recipe is currently tracked. Do not create empty `packaging/debian`, `packaging/rpm` or `packaging/arch` directories. #374 should move assets only if a later inventory finds real assets or a separately approved target.
- **Containers/deployment:** no Dockerfile, Compose file, Kubernetes manifest, Helm chart, deployment descriptor or container build context is present in the tracked tree.
- **Release metadata:** npm Changesets are under `.changeset/`; CLI release packaging/checksum logic is in `.github/workflows/release-rust-cli.yml`; npm tarball ordering/checksums are generated in `.github/workflows/stage-npm-packages.yml`. There is no separate root release manifest.
- **SBOM/provenance:** no committed SBOM, provenance attestation configuration, signing script or reusable release manifest was found. The CRAN audit generates dependency/license inventory in CI; npm staging generates a checksum list.
- **Signing/secrets:** no signing keys or package-signing configuration are tracked. CI secrets must remain in GitHub's secret store; none is part of this migration.
- **External automation:** Dependabot currently watches GitHub Actions at `/`, Cargo directories for each independent crate, npm at `/`, and pip at `/`. Update these entries after ecosystem moves. GitHub issue/PR links and scheduled automations may also contain old path names; search repository automation and workflow prompts before deleting compatibility paths.

## 6. Risks and required sequencing

1. **Accept ADR 0003 first.** Completed by merging PR #363. This inventory is based on that accepted target model.
2. **Resolve the cross-runtime experiment exception.** Add `experiments/` explicitly to the ADR root allowlist (or approve a different home) before moving `experiments/wasm-r`.
3. **Complete shared contracts/fixtures ownership (#370) and root policy (#365) early.** They affect all later ecosystem moves and can be implemented without moving production code.
4. **Generate the npm lockfile before moving the npm workspace (#265).** It must be produced by the supported npm toolchain in a network-enabled checkout, then validated with `npm ci` across Node, WASM, TypeScript, React, Vue and package dry-runs. Do not fabricate it through GitHub file APIs.
5. **Move Rust core and consumers as one coordinated sequence (#366).** Every current relative Cargo dependency points into `crates/corrplot-core`; offline/MSRV workflows and release paths are sensitive.
6. **Split Node/WASM dual ownership before or with the npm move (#367).** Their directories currently contain both Rust source and npm package content.
7. **Move the production R package independently (#368).** Keep it distinct from the disposable rextendr experiment and preserve CRAN gates #173/#268.
8. **Isolate Python reference code (#369).** Do not imply a distributable Python package or introduce a second authoritative implementation.
9. **Relocate packaging assets only when present (#374).** Coordinate with Rust CLI and CI work; Linux package publication remains deferred by #99.
10. **Update CI and developer documentation alongside each move (#371/#372).** Do not batch all path fixes into a final large PR.
11. **Run the final cross-project path and clean-checkout audit (#373).** Remove old root manifests only after every consumer and documented command has migrated.

## 7. Scope boundaries

This inventory does not authorize:
- production R migration or CRAN acceptance (#173/#268);
- public npm publication or publication credentials (#293);
- Linux distribution package publication (#99);
- a production Python package;
- semantic/API changes, package renames, or broad rewrites;
- the deferred manual cross-language performance dispatch in #339.

The private npm preview issue #361 remains blocked until maintainers identify and provision a genuinely private artifact destination. The repository is public, so its current GitHub Actions artifacts and GitHub Release assets cannot satisfy the private-preview access requirement.

## 8. Completion criteria

- [x] Current source roots, package manifests, lockfiles and ownership boundaries inventoried.
- [x] Current fixture sources and package copies mapped.
- [x] All 18 workflow path assumptions identified.
- [x] Container, deployment, OS packaging, release, SBOM/provenance and signing assets inventoried (including explicit absence where none exist).
- [x] Relative Cargo/npm/R/Python/documentation path risks recorded.
- [x] Source-to-target map and staged migration order documented.
- [x] No source files moved as part of this inventory.
