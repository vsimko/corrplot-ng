# npm package distribution and release preparation

**Status:** Workspace and non-publishing staging infrastructure are being implemented under #290.  
**Publication status:** Explicitly deferred. No package in this plan is authorized for public npm publication yet.

## Goals

Prepare a coherent TypeScript ecosystem for Node.js and browser consumers while keeping runtime dependencies isolated. Package builds, tests, versioning, and tarball staging can be implemented now; registry publication is a later maintainer-approved activity.

The shared API contract and semantics should be consistent across runtimes. That does not require one physical npm package or one dependency graph containing every backend.

## Workspace

The repository root is a private npm workspace, not a publishable package. Its workspace globs include the existing Node binding, all four platform-specific native packages, browser WASM binding, and initial TypeScript facade. Working bindings remain in their current directories; no package relocation is required.

Install and validate locally with:

```sh
npm install --ignore-scripts
npm ls --workspaces --depth=0
npm test
npm run pack:dry-run
```

The workspace uses Changesets for independent package versions and coordinated changelogs. Add a changeset for consumer-visible package changes with `npm run changeset`; do not add changesets for repository-only maintenance that does not alter published package contents.

The planned `@corrplot-ng/core`, React, and Vue packages are not yet present. They join through the workspace globs when their implementation issues (#145, #291, and #292) establish their manifests and public contracts. The current facade remains named `@corrplot-ng/typescript`; renaming it is a separately reviewed migration, not part of workspace setup.

## Target package topology

| Package | Responsibility | Runtime dependencies |
| --- | --- | --- |
| `@corrplot-ng/core` | Public TypeScript types, diagnostics, and runtime-neutral contracts | None; planned |
| `@corrplot-ng/wasm` | Browser execution backend and WASM assets | WASM/browser only |
| `@corrplot-ng/native-node` | Node.js execution backend | Node-API addon and matching platform package |
| `@corrplot-ng/react` | Optional React components/hooks | React peer dependency and browser API; planned |
| `@corrplot-ng/vue` | Optional Vue components/composables | Vue peer dependency and browser API; planned |
| `@corrplot-ng/typescript` | Initial runtime-selecting facade | Optional peer dependencies for the Node and WASM backends |

The Node package already has an implementation under `bindings/node/` and platform-specific package names. Preserve these names unless a separate migration is reviewed. Do not rename or recreate working native package artifacts as part of package-workspace preparation.

The browser package must not depend on, optionally install, import, or bundle the Node native package. React and Vue integrations must depend only on the browser-facing API. Frameworks remain optional peer dependencies and must not become dependencies of core or WASM.

## API consistency versus runtime isolation

The Node and browser packages should expose the same backend-neutral operations and public types where their renderer capabilities overlap. They may expose runtime-specific initialization and output capabilities where necessary.

- Browser users install the WASM package, not the Node package.
- Node users install the native Node package, not the browser WASM package.
- React/Vue users install the corresponding framework adapter and its browser-facing dependencies.
- A runtime-neutral core package contains types/contracts only; it must not pull in either execution backend.
- Do not create a universal package whose dependency graph installs both backends.
- Conditional exports may improve entry-point selection, but they do not replace dependency-graph isolation: npm still installs declared dependencies.

The current architecture and API contract remain governed by [ADR 0002](adr/0002-cross-runtime-typescript-architecture.md). Package topology here refines distribution boundaries so the browser dependency graph cannot be polluted by Node binaries.

## Coordinated versioning and build pipeline

Use Changesets for independent package versions, dependency-aware version bumps, and generated changelogs. Packages may share a release train without being forced to use identical version numbers.

The pull-request workspace workflow validates manifests, workspace links, Node binding tests, TypeScript facade tests, WASM tests/build, and package contents. The manual **Stage npm package tarballs (no publishing)** workflow:

1. Builds each native platform package on its matching GitHub-hosted runner.
2. Builds the browser WASM package.
3. Stages the WASM and Node base package tarballs after the platform binaries.
4. Stages the TypeScript facade after its runtime backends.
5. Assembles all seven tarballs into one CI artifact, records SHA-256 checksums, and documents the dependency order.

The staging workflow must remain non-publishing: it must not run `npm publish`, use npm credentials, request OIDC publication permissions, or publish automatically from tags. The current stage is build/test/package only.

## Future publication order

When a public release is explicitly approved, the intended order is:

1. Runtime-neutral core/types package.
2. Browser WASM package and Node platform-specific native packages.
3. Main Node package after its platform packages are available.
4. React and Vue integrations after their browser-facing dependencies.
5. The existing TypeScript facade, if retained, after its optional runtime backends.

The exact order must be derived from the actual package dependency graph at release time, not hard-coded if the topology changes.

npm publication is not transactional. The future workflow must detect already-published versions, stop safely on partial failure, and resume without attempting to overwrite immutable versions. A clean consumer install must verify each package after publication.

## Security and maintainer controls

Do not configure live npm credentials or Trusted Publishing during the current preparation work. When publication is prioritized, prefer GitHub Actions Trusted Publishing/OIDC with protected environments and explicit maintainer approval over long-lived npm tokens, where supported by npm's current requirements.

The future publication issue is #293. It remains deferred until maintainers explicitly schedule the release.

## React and Vue integration requirements

The framework packages are part of the intended ecosystem, but they are not prerequisites for completing the core/browser API:

- React integration: #291.
- Vue integration: #292.

Both must be thin host adapters. They must not implement correlation or rendering semantics, and their production bundles must be tested to ensure no Node native artifact or Node-only import is present.

## Work breakdown

- #145 — shared TypeScript API contract and runtime package isolation.
- #290 — workspace, versioning, CI, and non-publishing tarball/release preparation.
- #291 — React integration.
- #292 — Vue integration.
- #293 — later npm publication; explicitly deferred.

Existing backend implementation work remains in #141 (Node) and #142 (WASM). Extend the existing prototypes rather than restarting them.
