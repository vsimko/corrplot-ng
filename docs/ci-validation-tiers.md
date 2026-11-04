# CI validation tiers

The CI is split into a **fast, path-filtered pull-request tier** and a
**comprehensive tier**. Pull requests run the smallest relevant integration
check for each changed component. Full operating-system and runtime matrices,
release packaging, and experimental measurements remain available on
`workflow_dispatch`, scheduled runs, or pushes to `main`.

## Fast pull-request checks

| Changed paths | Pull-request validation |
| --- | --- |
| Test fixtures, schemas, normalized-matrix, shared SVG, publication config/SVG, renderer capability declaration | `test-foundation.yml`: fixture integrity, schema, Python contract tests, and generated SVG checks |
| `implementations/r/**` and R adapter tests | `test-r.yml` plus the foundation legacy-semantic check when its fixture paths change |
| `crates/corrplot-core/**`, `tools/corrplot-ng/**`, `prototypes/rust-rextendr/**` | `test-shared-rust-core.yml`: stable shared-core consumer, WASM portability, and declared MSRV checks |
| `tools/corrplot-ng/**` or shared core / publication CLI inputs | `test-rust-cli.yml`: Ubuntu release build, CLI smoke, and installed-outside-checkout test |
| `bindings/node/**` | `test-node-native.yml`: Linux x64 GNU on Node 24, including native addon tests and platform-package assembly |
| `bindings/wasm/**`, `packages/typescript/**`, `package.json`, `crates/corrplot-core/**` | `test-wasm-binding.yml`: Node 24 WASM tests, browser ESM build, and TypeScript facade browser smoke |
| `experiments/wasm-r/**` | `test-wasm-r-experiment.yml`: Ubuntu WASI and embedded-ABI builds, R/Node parity, and measurements |
| `prototypes/rust-rextendr/**` | Ubuntu offline package/vendor build, shared-core consumer, Windows binary/install smoke, and the fast CRAN license-policy check |
| `prototypes/cpp-rcpp/**` | `cpp-rcpp-cran-package.yml`: source-package validation |
| Rust rextendr Cargo/lock/deny files | `rust-cran-dependency-audit.yml`: fast license-policy check |
| `.github/workflows/**` | `test-workflows.yml`: actionlint validates every workflow, including manually dispatched workflows |
| `requirements-dev.txt` or `.github/dependabot.yml` | Foundation quick checks verify the declared Python test dependency and local command entry point |

Docs-only changes do not start component test jobs. The lightweight
path-detection jobs still run, and the documentation review remains part of
the pull request.

## Comprehensive validation

The following expensive or broad checks are deliberately not duplicated on
every pull request:

| Validation | Comprehensive trigger |
| --- | --- |
| Native Rust CLI on Linux, macOS, and Windows | `test-rust-cli.yml`: manual dispatch and push to `main` |
| Native Node binding on four OS/architectures and Node 22/24/26 | `test-node-native.yml`: manual dispatch and push to `main` |
| Browser WASM on Node 24 and 26, including browser smoke and baseline artifact | `test-wasm-binding.yml`: manual dispatch, weekly schedule, and push to `main` |
| WASM-in-R experiment on Ubuntu, Intel macOS, and Windows with pinned Wasmtime C API | `test-wasm-r-experiment.yml`: manual dispatch |
| Generated rextendr offline package on Ubuntu, Intel macOS, and Windows | `rust-rextendr-offline-package.yml`: manual dispatch or pushes to the two experiment branches; the pull request still runs Ubuntu and the separate Windows binary smoke |
| Prebuilt Node package artifact matrix | `build-node-prebuilt-packages.yml`: manual dispatch |
| Native CLI release packaging and archive audit | `release-rust-cli.yml`: manual dispatch or `cli-v*` tag |
| Full Cargo vendor/offline audit | `rust-cran-dependency-audit.yml`: weekly schedule or manual dispatch |

Security/dependency policy checks remain enabled for relevant pull requests.
The weekly CRAN dependency audit is not removed or replaced by the fast tier.

## Shared-core fan-out

A change under `crates/corrplot-core/**` intentionally fans out to all
supported consumers: shared-core stable/MSRV/WASM checks, native CLI,
Node-API, browser WASM, WASM/R, rextendr offline packaging, and the Windows
R binary smoke. Each PR workflow now runs its fast representative job; the
full matrices remain available through the comprehensive triggers above.

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
