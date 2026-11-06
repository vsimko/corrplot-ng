# Dependency update policy

## Automated update coverage

Dependabot checks these declared dependency sources weekly on Monday
(Europe/Berlin time):

| Ecosystem | Source | Policy |
| --- | --- | --- |
| GitHub Actions | `.github/workflows/` | Minor/patch updates grouped; major updates separate |
| Cargo | Seven ordinary Cargo project directories | Minor/patch updates grouped by dependency name across projects; major updates separate |
| npm | Root npm workspace (`package.json`, `bindings/node`, `bindings/wasm`, and `packages/*`) | External dependencies across workspace manifests are maintained; four unpublished native platform-package references are ignored |
| pip | `requirements-dev.txt` | Python development/test dependencies; minor/patch updates grouped |

The Cargo projects covered by routine version updates are
`rust/bindings/node`, `rust/bindings/wasm`, `rust/crates/corrplot-core`,
`experiments/wasm-r/native-r`, `experiments/wasm-r/wasm-abi`,
`experiments/wasm-r/wasm-host`, `rust/tools/corrplot-cli`, and the standalone `rust/bindings/r/rextendr-prototype` lockfile.

The npm workspace is rooted at `/` and declares `bindings/node`,
`bindings/wasm`, and `packages/*` in the root `package.json`. Dependabot
must be configured at that root rather than with overlapping per-workspace
entries, so that it can coordinate changes to the workspace package manifests.
The external npm dependencies currently declared by the root and workspaces
include `@changesets/cli`, TypeScript, esbuild, React/React DOM and types,
Vue and its Vite plugin, Vite, and the Node binding's optional platform
package references. The `bindings/wasm` package has no external npm
dependencies of its own.

## Deliberate exclusions and manual coordination

- `rust/bindings/r/rextendr-prototype` is configured for security updates only. Its
  `extendr-api` dependency is pinned to 0.9.0 in the Cargo manifest and in
  the generated-package preparation and license-provenance checks. A routine
  version change must update those pieces together and pass the three-platform
  offline vendor/package and CRAN checks. The zero open-PR limit disables
  routine version-update PRs for this directory; it does not disable security
  update PRs.
- The four `@corrplot-ng/native-node-*` packages are unpublished internal
  platform packages, not external npm dependencies. Their versions must move
  together with the base package and prebuilt artifacts, so Dependabot ignores
  those references rather than proposing unresolvable registry updates.
- Other `@corrplot-ng/*` package dependencies are workspace packages declared
  in the root workspace. They are resolved from the local workspace and are
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
  integration workflows.
- There are no Docker manifests or independently maintained Python lockfiles
  in the repository, so no Docker or additional pip update jobs are configured.

## npm reproducibility decision

The npm workspace currently has no committed root `package-lock.json`.
The workspace test, browser-WASM, and package-staging workflows explicitly
install with `--package-lock=false`; they therefore resolve semver ranges at
run time rather than reproducing an identical transitive dependency tree.
The current policy is to keep this pre-release workspace lockfile-free while
the package set and internal package relationships are changing, and to be
explicit that these installs are not bit-for-bit reproducible.

A committed root npm lockfile is the preferred approach before release
candidate or publication work. At that point, generate it with the repository's
supported npm version, switch CI installs to `npm ci` (retaining the required
lifecycle-script and optional-dependency controls), and validate the Node,
WASM, TypeScript, React, and Vue workflows. Do not add a hand-authored or
partially generated lockfile. The npm Dependabot entry uses
`versioning-strategy: increase` so that it can keep a future root lockfile
coherent with workspace manifest updates.

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

- Cargo changes in the shared core, CLI, Node, WASM, and WASM/R projects run
  their corresponding locked build/test workflows.
- rextendr security updates run the offline package/vendor and R integration
  workflow, including all three supported operating systems.
- npm workspace changes run `test-npm-workspace.yml`, which validates the
  workspace graph and exercises Node, TypeScript, React, Vue, and browser-WASM
  package checks.
- npm changes under `bindings/node` also run the native Node test matrix;
  changes to the browser binding and framework packages run
  `test-wasm-binding.yml`.
- Python requirement changes run the foundation test suite.
- GitHub Actions updates are reviewed as workflow changes and must leave the
  relevant workflow checks green.

Do not commit generated vendor directories or compiled package artifacts as
independent dependency updates. Regenerate and validate them through their
source manifests and the existing packaging workflows.
