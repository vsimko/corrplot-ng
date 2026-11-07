# ADR 0003: Repository layout as a multi-ecosystem monorepo

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decision owners:** corrplot-ng maintainers
- **Related:** [ADR 0001](0001-shared-core-architecture.md), [ADR 0002](0002-cross-runtime-typescript-architecture.md), [current architecture](../architecture.md)

## Context

corrplot-ng is one product family with several independently buildable and distributable components: a Rust shared core and CLI, native Node and browser-WASM bindings, multiple npm packages, an R package, and a Python reference implementation or future binding. These components share contracts, fixtures, release coordination, and CI, but they have different toolchains and dependency managers.

The current repository has grown incrementally. Runtime-specific components are not yet consistently grouped by ecosystem, and the repository root risks becoming a place where unrelated package managers install dependencies or maintain lockfiles. This makes ownership, setup, CI path filtering, and release boundaries harder to understand.

The goal is a monorepo: one Git repository and one coordinated history, containing multiple clearly bounded projects. Each project should be independently understandable and buildable while shared implementation and contract dependencies remain explicit.

## Decision

Adopt an ecosystem-oriented monorepo layout. **Do not use Git submodules** for the components described here. A directory in this repository is a normal tracked project, not a nested independent Git repository. The monorepo keeps one source of truth, atomic cross-component changes, shared CI, and repository-wide issue/PR coordination.

The root directory is reserved for repository-wide governance, documentation entry points, and tooling configuration. It must not become the default installation location for npm, Cargo, Python, or R dependencies.

### Target layout

The following is the target structure. It is a destination and ownership model, not a claim that every directory already exists.

```text
corrplot-ng/
├── contracts/
│   ├── normalized-matrix-v1/
│   ├── scene-v1/
│   ├── diagnostics-v1/
│   ├── renderer-capability-v1/
│   └── publication-config-v1/
│
├── rust/
│   ├── Cargo.toml                 # Rust workspace root
│   ├── Cargo.lock                 # shared Rust dependency resolution
│   ├── crates/
│   │   └── corrplot-core/
│   ├── tools/
│   │   └── corrplot-cli/
│   └── bindings/
│       ├── node/
│       ├── wasm/
│       └── r/
│
├── npm/
│   ├── package.json               # private npm workspace root
│   ├── package-lock.json          # one lockfile for this npm workspace
│   └── packages/
│       ├── native-node/
│       ├── wasm/
│       ├── typescript/
│       ├── react/
│       └── vue/
│
├── r/
│   └── corrplotng/                 # standalone R package root
│       ├── DESCRIPTION
│       ├── NAMESPACE
│       ├── R/
│       ├── src/
│       ├── tests/
│       └── ...
│
├── python/
│   ├── reference/                  # current Python reference/prototype
│   └── corrplot-ng/                # future distributable Python package, if approved
│
├── packaging/
│   ├── common/                     # shared release/package metadata and helpers
│   ├── debian/                     # Debian source/binary package rules
│   ├── rpm/                        # RPM spec and supporting files
│   └── arch/                       # Arch PKGBUILD and supporting files
│
├── tools/
│   └── repository/                 # repo-wide scripts only
│
├── experiments/                  # isolated non-production research and cross-runtime proofs
│   └── wasm-r/                    # independent experiment; excluded from production Rust workspace
│
├── fixtures/
│   ├── normalized-matrix-v1/
│   ├── scene-v1/
│   └── matrix-input-v1/
│
├── docs/
│   ├── adr/
│   └── ...
│
├── .github/
│   └── workflows/
│
├── .editorconfig
├── .gitignore
├── CONTRIBUTING.md
└── README.md
```

The exact placement of a component may be adjusted during the inventory phase if current build tooling imposes a concrete constraint. Such an adjustment must preserve the ownership and dependency rules in this ADR and be recorded in the migration issue/PR. Avoid moving files merely to make directory names symmetrical.

## Ownership and project boundaries

| Area | Owns | Must not own |
|---|---|---|
| `contracts/` | Versioned schemas, compatibility rules, contract documentation | Runtime code, package-manager dependencies |
| `rust/` | Shared Rust semantics, Rust bindings, Rust CLI, Cargo workspace | npm/R/Python dependency installation |
| `npm/` | npm package manifests, workspace orchestration, JS/TS package builds and tests | Rust source of truth or R/Python environments |
| `r/corrplotng/` | R public API, R compatibility behavior, R tests and package metadata | Duplicated shared semantics where Rust core is available and approved |
| `python/reference/` | Reference/prototype behavior and conformance support | An implied production Python API or independent authoritative core |
| `packaging/` | OS distribution recipes, package metadata, file lists, signing/release helpers specific to distributable artifacts | Application source, ecosystem dependency manifests, CI workflow definitions |
| `python/corrplot-ng/` | Only a future explicitly approved Python distribution | Unapproved duplicate production semantics |
| `fixtures/` | Cross-language golden inputs and expected outputs | Runtime-specific test harnesses |
| `experiments/` | Isolated, non-production research and cross-runtime experiments such as `experiments/wasm-r/`; each keeps an independent build/test boundary and remains outside production workspaces | Production APIs, authoritative semantics, release artifacts without separate approval |
| `.github/` | CI orchestration, automation, release governance | Application/package implementation |

A distributable component must have a clear owner directory, manifest, tests, build instructions, and declared dependency boundary. A component can be independently released without becoming a separate Git repository.

## Dependency and workspace policy

### npm

- `npm/` is the npm workspace root. It has a private root `package.json` and a single committed `package-lock.json`.
- Every installable package under `npm/packages/*/` has its own `package.json`, package name, versioning/release metadata as appropriate, exports, dependencies, and package-level tests.
- Run `npm ci` from `npm/`. It creates `npm/node_modules/`; no npm install is run from repository root.
- Do not commit package-local lockfiles for members of this single npm workspace. If a package later needs truly independent dependency resolution, first split it from the workspace deliberately and document the trade-off.
- Keep npm scripts and workspace configuration scoped to `npm/`. Repository-wide scripts may delegate to it.
- Public npm publication remains separately gated by #293. The layout change does not authorize public publication or credentials.

### Rust

- `rust/Cargo.toml` defines the Cargo workspace and `rust/Cargo.lock` is the workspace lockfile for applications/binaries and reproducible workspace builds.
- Each crate/binding has its own `Cargo.toml` and explicit path/version dependencies.
- Run Cargo commands with `rust/` as the working directory or an explicit manifest path. Do not create a root Cargo workspace merely for convenience.
- Preserve the selected Rust shared-core architecture in ADR 0001. Bindings remain adapters and must not fork shared semantics.

### R

- `r/corrplotng/` is the R package root and owns `DESCRIPTION`, `NAMESPACE`, `R/`, `src/`, `tests/`, and package-specific configuration.
- R package installation/check commands run against this package directory.
- Use an R-specific dependency lock/environment mechanism such as `renv` only if adopted for a concrete reproducibility need; do not create a fake root-level lockfile.
- The production R package rollout gates in #173 and the CRAN NOTE disposition in #268 remain unchanged by this ADR.

### Packaging and operating-system distributions

- `packaging/` owns distribution-specific packaging configuration and supporting files, separated by target format (`debian/`, `rpm/`, `arch/`) plus genuinely shared metadata/helpers under `common/`.
- Debian control/rules/changelog/source-package files belong under `packaging/debian/`; RPM spec files belong under `packaging/rpm/`; Arch `PKGBUILD` files belong under `packaging/arch/`.
- Packaging recipes consume versioned build outputs produced by the owning ecosystem (initially the Rust CLI under `rust/`). They must not become a second build-system or duplicate application source.
- Build configuration and dependency manifests remain with their ecosystem; package recipes and distribution metadata live under `packaging/`; CI orchestration remains under `.github/workflows/`.
- Shared release metadata may live under `packaging/common/` only when it is consumed by more than one packaging target. Avoid premature generic packaging frameworks.
- Dockerfiles and deployment descriptors belong with the deployable component or deployment target they describe; reusable release/package manifests and distribution scripts belong under `packaging/` or `tools/repository/` according to ownership. The inventory issue must decide each concrete path rather than moving files by filename alone.
- Package signing keys and credentials must never be committed. Signing configuration may be versioned, while secrets remain in the approved CI secret store.
- Linux distribution packages remain deferred until the CLI is stable, as tracked by #99. This ADR and the layout migration do not authorize publishing packages.

### Python

- The current Python implementation remains a reference/prototype under `python/reference/` unless inventory proves a different path is required.
- A distributable Python package must be a separate explicit project with `pyproject.toml`, source package, tests, and documented build/install workflow.
- Use one lockfile per independently managed Python environment/project when the selected package manager requires it (for example, `uv.lock`). Do not imply that a production Python package exists merely by creating its directory.
- Do not introduce a second authoritative implementation of shared semantics; follow ADR 0001 and use Rust bindings or the versioned contracts as appropriate.

## Cross-project dependency rules

1. Contracts are language-neutral and do not import implementation code.
2. Rust core owns shared correlation/visualization semantics as they are implemented.
3. Native Node, browser WASM, CLI, and approved R/Python bindings depend on the Rust core through explicit adapter boundaries.
4. npm packages depend on one another through declared workspace/package dependencies, never relative imports that escape package ownership.
5. React and Vue remain thin browser adapters over the supported browser/WASM API.
6. The Python reference remains explicitly non-authoritative.
7. Shared fixtures are consumed by ecosystem-specific test harnesses; fixtures do not become runtime dependencies.
8. No package may rely on another ecosystem's generated dependency directory (for example, `node_modules`, `.venv`, Cargo target output, or R library cache).
9. No dependency may be installed implicitly at repository root.
10. Any intentional cross-boundary dependency must be declared, documented, and exercised by CI.

## Root-level policy

Allowed at repository root:
- `experiments/` only for isolated, non-production research or cross-runtime proofs with their own declared manifests and test boundary; it is not part of the production Rust workspace.
- Git and GitHub configuration, CODEOWNERS, contribution/security/governance documents.
- Repository-wide documentation entry points and ADR index.
- Shared editor settings and ignore rules.
- Thin orchestration scripts that delegate to an ecosystem workspace.
- CI workflow definitions.

Not allowed at repository root:
- npm `package.json`/lockfile used to install product dependencies.
- Cargo workspace/lockfile used as an accidental catch-all.
- Python virtual environment, Python package environment, or unrelated Python lockfile.
- R package metadata or installed R libraries.
- Distribution packaging recipes or OS package metadata (these belong under `packaging/`).
- Generated `node_modules/`, `target/`, `.venv/`, build outputs, or package caches.

A root-level task runner may be introduced only as orchestration; it must not absorb package manifests or dependency graphs.

## CI and developer experience

- CI workflows must be scoped by changed paths where practical, while retaining a repository-wide integration workflow for cross-project changes.
- Each ecosystem must have a documented clean setup, build, test, lint/check, and artifact/package validation command.
- Root-level CI may call ecosystem workflows, but the actual commands should run in the owning project directory.
- Packaging jobs consume validated artifacts from the owning build workflow and invoke recipes from `packaging/`; they must not silently rebuild a different source revision.
- Add a layout/ownership check that detects forbidden root manifests and generated dependency directories.
- Cross-language contract changes must run all relevant producer/consumer conformance tests.
- A clean checkout must be sufficient to reproduce documented setup; generated dependencies and build products are not committed.
- Avoid requiring every contributor to install every toolchain for a change isolated to one ecosystem.

## Migration principles

The migration is structural, not a rewrite. Preserve behavior, package names, public APIs, contract versions, and release gates unless a separate issue explicitly changes them.

1. Inventory current files, manifests, lockfiles, workflows, path assumptions, package names, release scripts/manifests, Dockerfiles, deployment descriptors, signing configuration, distribution recipes, and generated artifacts.
2. Create the destination directories and move one ecosystem at a time using history-preserving Git moves where possible.
3. Update workspace manifests, relative paths, build scripts, test fixtures, documentation, and CI in the same bounded migration step.
4. Verify clean-checkout installation/build/test/package artifacts for each moved ecosystem.
5. Verify cross-platform workflows and release workflows before removing compatibility paths.
6. Remove obsolete root-level manifests and scripts only after all consumers have migrated.
7. Keep each migration PR reviewable and reversible. Do not combine layout migration with semantic refactoring or public release changes.
8. Update architecture documentation and this ADR if evidence requires a deliberate deviation.

## Non-goals

- Splitting the repository into multiple Git repositories or introducing Git submodules.
- Rewriting the Rust core, R package, TypeScript API, or Python prototype.
- Declaring all current integrations production-ready or claiming full semantic parity.
- Authorizing production R rollout, CRAN submission, or public npm publication.
- Creating a production Python package without a separate product decision.
- Changing package names, versions, or public APIs as a side effect of moving files.

## Consequences

- Each ecosystem has a visible, bounded project root and its own dependency lifecycle.
- The repository root stays clean and does not accumulate unrelated package-manager state.
- Cross-component changes remain atomic and can be reviewed in one PR.
- Workspace-level lockfiles keep dependency resolution reproducible without duplicating lockfiles for every member package.
- CI and documentation need coordinated updates during migration; temporary path changes are expected.
- The migration will require several small, ordered issues/PRs rather than one large directory move.

## Acceptance criteria for this ADR

- The target layout and ownership boundaries are documented and agreed.
- Existing files and workflows are mapped to target locations before moves begin.
- Each ecosystem's workspace and lockfile strategy is explicit.
- No generated dependency directory is committed.
- Every migration step has clean-checkout validation and a rollback path.
- Existing release gates and architectural decisions remain intact.
