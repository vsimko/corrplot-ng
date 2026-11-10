# Dependency update policy

## Automated update coverage

Dependabot checks these declared dependency sources weekly on Monday
(Europe/Berlin time):

| Ecosystem | Source | Policy |
| --- | --- | --- |
| GitHub Actions | `.github/workflows/` | Minor/patch updates grouped; major updates separate |
| Cargo | `/rust` workspace (committed lockfile), three independent WASM-R experiment projects (manifest-only; no committed lockfiles), and excluded standalone R/rextendr package (separate lockfile) | Workspace and experiment minor/patch updates grouped; R/rextendr routine updates disabled, security updates retained |
| npm | npm workspace (`npm/package.json`, `npm/package-lock.json`, and `npm/packages/*`) | External dependencies across workspace manifests are maintained; four unpublished native platform-package references are ignored |
| pip | `requirements-dev.txt` | Python development/test dependencies; minor/patch updates grouped |

The production-facing Cargo workspace is rooted at `/rust` and contains the shared core, CLI, Node and WASM crates. The three `experiments/wasm-r` crates remain independent projects. The excluded `rust/bindings/r/rextendr-prototype` package retains its own lockfile and a separate security-only Dependabot entry.

Only `/rust` and `rust/bindings/r/rextendr-prototype` currently commit Cargo lockfiles. The three WASM/R experiment projects have no committed `Cargo.lock`; Dependabot updates their manifests, and their CI builds resolve compatible versions without `--locked`. This is an explicit limitation of the disposable experiments, not a reproducible locked-build guarantee. If any experiment becomes a maintained or release input, add its lockfile and validate switching its workflow to `--locked` as a separate change.

The npm workspace is rooted at `/npm` and declares the five installable packages in `npm/package.json`. Dependabot is configured at that root so it can coordinate manifest and lockfile updates.
The external npm dependencies currently declared by the root and workspaces
include `@changesets/cli`, TypeScript, esbuild, React/React DOM and types,
Vue and its Vite plugin, and Vite. The Node package's unpublished platform
package versions are coordinated in `npm/packages/native-node/platform-dependencies.json`.
The `npm/packages/wasm` package has no external npm dependencies of its own.

## Deliberate exclusions and manual coordination

- `rust/bindings/r/rextendr-prototype` is configured for security updates only. Its
  `extendr-api` dependency is pinned to 0.9.0 in the Cargo manifest and in
  the generated-package preparation and license-provenance checks. A routine
  version change must update those pieces together and pass the three-platform
  offline vendor/package and CRAN checks. The zero open-PR limit disables
  routine version-update PRs for this directory; it does not disable security
  update PRs.
- The four `@corrplot-ng/native-node-*` packages are unpublished internal
  platform packages, not external npm dependencies. Their versions must move together with the base package and prebuilt artifacts.
  Their coordinated metadata is kept outside the workspace manifest, so Dependabot
  does not propose unresolvable registry updates.
- Other `@corrplot-ng/*` package dependencies are workspace packages declared
  in the `npm/` workspace. They are resolved from the local workspace and are
  not intended to be updated from the public npm registry.
- The `dtolnay/rust-toolchain` GitHub Action is excluded from Dependabot
  version updates. Its tags represent Rust toolchain releases rather than
  ordinary semver action releases, which caused Dependabot to propose
  nonexistent versions. Update the stable and pinned MSRV toolchains manually
  and validate the affected Rust workflows.
- Python packages are declared in `requirements-dev.txt`. There is no
  separate `pyproject.toml` or Python packaging manifest.
- R package dependencies are declared in the relevant `DESCRIPTION` files.
  Dependabot does not provide an R package ecosystem; update these dependencies
  deliberately and validate them with the R package and cross-platform
  integration workflows. Some experiment/benchmark-only R packages are listed
  directly in `setup-r-dependencies` workflow steps rather than a package
  `DESCRIPTION`; maintain those lists manually as well.
- There are no Docker manifests or independently maintained Python lockfiles
  in the repository, so no Docker or additional pip update jobs are configured.

## Manually pinned CI tools and runtime dependencies

The following tools are installed or pinned directly in workflow files rather than declared in a dependency manifest, so Dependabot does not update their versions. Update all occurrences of a shared pin together and run the listed validation:

| Tool | Current pin | Source and manual validation |
| --- | --- | --- |
| Rust toolchain | `stable`; MSRV `1.78` | `dtolnay/rust-toolchain` is intentionally ignored by Dependabot because its tags are Rust releases, not semver action releases. Update stable/MSRV references manually and run affected Rust workflows. |
| `wasm-pack` | `0.13.1` | Pinned in `stage-npm-packages.yml`, `test-node-wasm-parity.yml`, `test-npm-workspace.yml`, and `test-wasm-binding.yml`, including cache keys. Update every occurrence together; validate npm workspace, Node/WASM parity, browser-WASM, and staging workflows. |
| Playwright | `1.56.1` | Installed transiently by `test-wasm-binding.yml`, outside the npm workspace manifest. Update the workflow pin and validate Chromium, Firefox, and WebKit smoke tests. |
| Wasmtime CLI and C API | `50.0.1` | Pinned in `test-wasm-r-experiment.yml`; C API archives also have per-platform SHA-256 pins. Update version, archive names, and hashes together, then run the manual experiment matrix. |
| actionlint | `1.7.12` | Pinned through `go run` in `test-workflows.yml`. Update manually and run the workflow-lint job. |
| `cffconvert` | `2.0.0` | Installed directly by `validate-citation.yml`, outside `requirements-dev.txt`. Update manually and run citation validation. |
| R experiment/benchmark packages | Workflow package lists | Packages such as `jsonlite`, `digest`, `V8`, `rextendr`, and `usethis` are installed by workflow `setup-r-dependencies` steps. Update those lists manually and run the owning R/experiment workflow. |

Runtime versions (Node, R, Python) and runner-provided OS packages are maintained in their owning workflow/support-policy definitions rather than by the package ecosystem update jobs.

## npm reproducibility decision

The npm workspace has a committed `npm/package-lock.json`, generated by npm 11 on Node 24 and validated with `npm ci`. Normal workspace validation uses `npm ci --ignore-scripts` with optional dependencies present. `--omit=optional` is reserved for the isolated consumer smoke test, which installs the separately staged platform tarballs directly. The four unpublished native platform packages are intentionally excluded from the workspace and lockfile; their metadata is added to the base Node package only when its distributable tarball is created. Dependabot updates the `/npm` workspace and lockfile together.

## Grouping and security updates

Only version updates are grouped. Minor/patch updates are grouped to reduce
noise; major updates remain separate for easier review. Security updates are
not placed in these version-update groups, so urgent vulnerability fixes are
not delayed waiting for a broader batch.

The repository's Dependabot security-update setting is managed in GitHub
repository settings, not by this YAML file. Keep Dependabot alerts and
security updates enabled under **Settings → Security & analysis**.

## Validation expectations

Dependabot pull requests must run the existing checks for the affected
manifest paths:

- Cargo changes in the `/rust` workspace run the locked build/test workflows. The independent WASM/R experiment projects run their manifest-based build workflow without committed lockfiles; pull requests use the fast Ubuntu job, while the full OS matrix is manually dispatched.
- rextendr security-update pull requests run the offline package/vendor and R integration workflow on Ubuntu. Its full Ubuntu/Intel macOS/Windows matrix is available through manual dispatch or the designated experiment-branch pushes, not on every pull request.
- npm workspace changes run `test-npm-workspace.yml`, which validates the
  workspace graph and exercises Node, TypeScript, React, Vue, and browser-WASM
  package checks. Its Node 24 native-toolchain matrix also installs the
  committed platform-specific optional dependencies and verifies the
  TypeScript 7 compiler and Vite 8/Rolldown bundle on Ubuntu x64, Intel macOS,
  Apple Silicon macOS, and Windows x64.
- npm changes under `npm/packages/native-node` also run the native Node test matrix;
  changes to the browser binding and framework packages run
  `test-wasm-binding.yml`.
- Python requirement changes run the foundation test suite.
- GitHub Actions updates are reviewed as workflow changes and must leave the
  relevant workflow checks green.

Do not commit generated vendor directories or compiled package artifacts as
independent dependency updates. Regenerate and validate them through their
source manifests and the existing packaging workflows.
