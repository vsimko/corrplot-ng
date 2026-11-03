# Dependency update policy

## Automated update coverage

Dependabot checks these declared dependency sources weekly on Monday
(Europe/Berlin time):

| Ecosystem | Source | Policy |
| --- | --- | --- |
| GitHub Actions | `.github/workflows/` | Minor/patch updates grouped; major updates separate |
| Cargo | Seven ordinary Cargo project directories | Minor/patch updates grouped by dependency name across projects; major updates separate |
| npm | `bindings/node/package.json` | External dependencies are maintained; the four internal platform-package references are ignored until published |
| pip | `requirements-dev.txt` | Python development/test dependencies; minor/patch updates grouped |

The Cargo projects covered by routine version updates are
`bindings/node`, `bindings/wasm`, `crates/corrplot-core`,
`experiments/wasm-r/native-r`, `experiments/wasm-r/wasm-abi`,
`experiments/wasm-r/wasm-host`, and `tools/corrplot-ng`.

## Deliberate exclusions and manual coordination

- `prototypes/rust-rextendr` is configured for security updates only. Its
  `extendr-api` dependency is pinned to 0.9.0 in the Cargo manifest and in
  the generated-package preparation and license-provenance checks. A routine
  version change must update those pieces together and pass the three-platform
  offline vendor/package and CRAN checks. The zero open-PR limit disables
  routine version-update PRs for this directory; it does not disable security
  update PRs.
- The four `@corrplot-ng/native-node-*` packages are internal platform
  packages, not external npm dependencies. Their versions must move together
  with the base package and prebuilt artifacts, so Dependabot ignores those
  references rather than proposing unresolvable registry updates.
- The `dtolnay/rust-toolchain` GitHub Action is excluded from Dependabot
  version updates. Its tags represent Rust toolchain releases rather than
  ordinary semver action releases, which caused Dependabot to propose
  nonexistent versions. Update the stable and pinned MSRV toolchains manually
  and validate the affected Rust workflows.
- Python packages are now declared in `requirements-dev.txt`. There is no
  separate `pyproject.toml` or Python packaging manifest.
- R package dependencies are declared in the relevant `DESCRIPTION` files.
  Dependabot does not provide an R package ecosystem; update these dependencies
  deliberately and validate them with the R package and cross-platform
  integration workflows.
- There are no Docker manifests or independently maintained Python lockfiles
  in the repository, so no Docker or additional pip update jobs are configured.

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
- npm changes under `bindings/node` run the native Node test matrix.
- Python requirement changes run the foundation test suite.
- GitHub Actions updates are reviewed as workflow changes and must leave the
  relevant workflow checks green.

Do not commit generated vendor directories or compiled package artifacts as
independent dependency updates. Regenerate and validate them through their
source manifests and the existing packaging workflows.
