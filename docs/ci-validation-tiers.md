# CI validation tiers

The CI is split into a **fast, path-filtered pull-request tier** and a
**comprehensive tier**. Pull requests run the smallest relevant integration
check for each changed component. Full operating-system and runtime matrices,
release packaging, and experimental measurements remain available on
`workflow_dispatch`, scheduled runs, or pushes to `main`.

The workflow names and triggers below are reconciled with the current files in
`.github/workflows/`. There is no active C++/Rcpp package workflow; the
retired implementation and its comparison material are historical only.

## Fast pull-request checks

| Changed paths | Pull-request validation |
| --- | --- |
| Test fixtures, schemas, normalized-matrix, shared SVG, publication config/SVG, renderer capability declaration | `test-foundation.yml`: fixture integrity, schema, Python contract tests, generated SVG checks, and legacy semantic-oracle validation |
| `implementations/r/**` and R adapter tests | `test-r.yml` plus the foundation legacy-semantic check when its fixture paths change |
| `rust/crates/corrplot-core/**`, `rust/tools/corrplot-cli/**`, `rust/bindings/r/rextendr-prototype/**` | `test-shared-rust-core.yml`: stable shared-core consumer, WASM portability, and declared MSRV checks |
| `rust/tools/corrplot-cli/**`, shared core, or publication CLI inputs | `test-rust-cli.yml`: Ubuntu release build, CLI smoke, and installed-outside-checkout test |
| `npm/packages/native-node/**`, `rust/bindings/node/**` or `rust/crates/corrplot-core/**` | `test-node-native.yml`: Linux x64 GNU on Node 24, including native addon tests and platform-package assembly |
| `npm/packages/native-node/**`, `rust/bindings/node/**`, `npm/tests/cross-backend/**`, `npm/packages/wasm/**`, `rust/bindings/wasm/**`, `rust/crates/corrplot-core/**`, shared fixtures, or the two shared-SVG scene examples | `test-node-wasm-parity.yml`: Node 24 native/WASM semantic, SVG, and RGBA parity plus benchmark-harness smoke (no timing thresholds) |
| `npm/package.json`, `npm/packages/**`, `npm/packages/native-node/**`, `rust/bindings/node/**`, `npm/packages/wasm/**`, `rust/bindings/wasm/**`, or `npm/.changeset/**` | `test-npm-workspace.yml`: workspace graph, Node, TypeScript, React, Vue, browser-WASM package, and package-content checks |
| `npm/packages/wasm/**`, `rust/bindings/wasm/**`, `npm/packages/typescript/**`, `npm/packages/react/**`, `npm/packages/vue/**`, `npm/package.json`, or `rust/crates/corrplot-core/**` | `test-wasm-binding.yml`: Node 24 WASM tests, browser ESM build, TypeScript facade and direct Canvas browser conformance smoke, React Strict Mode lifecycle smoke, and Vue component/composable browser smoke |
| `experiments/wasm-r/**`, shared core, or the WASM/R experiment workflow | `test-wasm-r-experiment.yml`: Ubuntu WASI and embedded-ABI builds, R/Node parity, and measurements |
| `rust/bindings/r/rextendr-prototype/**`, shared core, or the rextendr offline-package workflow | `rust-rextendr-offline-package.yml`: Ubuntu offline package/vendor build, shared-core consumer, and fast CRAN package checks |
| `rust/bindings/r/rextendr-prototype/**`, shared core, `docs/cran-rust-dependency-audit-results.md`, or the Windows binary workflow | `test-rust-windows-binary.yml`: Windows binary package build and installation with a restricted PATH |
| rextendr Cargo manifest/lock/deny files, shared-core Cargo manifest, or the dependency-audit workflow | `rust-cran-dependency-audit.yml`: fast license-policy check |
| `CITATION.cff` or `validate-citation.yml` | `validate-citation.yml`: citation metadata validation |
| `.github/workflows/**` | `test-workflows.yml`: actionlint validates every workflow, including manually dispatched workflows |
| `requirements-dev.txt` or `.github/dependabot.yml` | Foundation quick checks verify the declared Python test dependency and local command entry point |
| `.github/workflows/cross-language-benchmarks.yml`, `benchmarks/**`, `rust/bindings/r/rextendr-prototype/**`, or `rust/crates/corrplot-core/**` | `cross-language-benchmarks.yml`: benchmark-harness and shared-fixture parity smoke; timing/stress runs are reserved for manual dispatch |

`test-foundation.yml` and `test-r.yml` use pull-request path detection inside
the workflow rather than event-level `paths` filters. Docs-only changes still
start their lightweight path-detection jobs, but do not start unrelated
component tests. The table above describes the component coverage, not a
claim that every workflow has an event-level path filter.

## Comprehensive validation

The following expensive or broad checks are deliberately not duplicated on
every pull request:

| Validation | Comprehensive trigger |
| --- | --- |
| Native Rust CLI on Linux, macOS, and Windows | `test-rust-cli.yml`: manual dispatch and push to `main` |
| Native Node binding on four OS/architectures and Node 22/24/26 | `test-node-native.yml`: manual dispatch and push to `main` |
| Browser WASM on Node 24 and 26, including browser smoke and baseline artifacts | `test-wasm-binding.yml`: manual dispatch, weekly schedule, and push to `main` |
| Native Node vs Node-hosted WASM performance baseline | `test-node-wasm-parity.yml`: manual dispatch, weekly schedule, and push to `main`; PRs smoke-test the harness without timing thresholds |
| WASM-in-R experiment on Ubuntu, Intel macOS, and Windows with pinned Wasmtime C API | `test-wasm-r-experiment.yml`: manual dispatch; PRs run the Ubuntu smoke |
| Generated rextendr offline package on Ubuntu, Intel macOS, and Windows | `rust-rextendr-offline-package.yml`: manual dispatch or pushes to the two experiment branches; PRs run the Ubuntu package check |
| Windows rextendr binary build and restricted-PATH installation | `test-rust-windows-binary.yml`: manual dispatch; PRs run the Windows binary smoke |
| Cross-language regular and B03/B04 stress benchmarks | `cross-language-benchmarks.yml`: manual dispatch; PRs run the path-filtered correctness/harness smoke |
| Prebuilt Node platform package artifact matrix | `build-node-prebuilt-packages.yml`: manual dispatch |
| Staged npm package set and checksums | `stage-npm-packages.yml`: manual dispatch |
| Native CLI release packaging and archive audit | `release-rust-cli.yml`: manual dispatch or `cli-v*` tag |
| Full Cargo vendor/offline audit | `rust-cran-dependency-audit.yml`: weekly schedule or manual dispatch |

Security/dependency policy checks remain enabled for relevant pull requests.
The weekly CRAN dependency audit is not removed or replaced by the fast tier.

## Shared-core fan-out

A change under `rust/crates/corrplot-core/**` intentionally fans out to all
supported consumers: shared-core stable/MSRV/WASM checks, native CLI,
Node-API, browser WASM, WASM/R, rextendr offline packaging, and the Windows
R binary smoke. Each PR workflow runs its fast representative job; the full
matrices remain available through the comprehensive triggers above.

Changes to a binding trigger that binding's fast integration test. Changes to
the renderer-neutral contracts and fixtures trigger the foundation checks.
Changes to workflow YAML trigger actionlint independently of the workflow's
own event configuration, so manual-only workflows are also linted.

## Design constraints

- Do not add timing thresholds to ordinary correctness tests.
- Keep the Rust MSRV, WASM portability, R integration, and dependency-license
  policy checks in the pull-request tier for shared-core and rextendr changes.
- Keep security updates independent from broad version-update groups.
- Keep stress benchmarks, cross-language performance comparisons, and the
  WASM/R experiment's full platform matrix opt-in.
- A skipped path-filtered job is intentional; the path table above is the
  source of truth for which checks a changed component requires.
