# npm package distribution and release preparation

**Status:** Workspace, versioning, CI validation, and non-publishing staging infrastructure are complete under #290.  
**Publication status:** Explicitly deferred. No package in this plan is authorized for public npm publication yet.

## Goals

Prepare a coherent TypeScript ecosystem for Node.js and browser consumers while keeping runtime dependencies isolated. Package builds, tests, versioning, and tarball staging can be implemented now; registry publication is a later maintainer-approved activity.

The shared API contract and semantics should be consistent across runtimes. That does not require one physical npm package or one dependency graph containing every backend.

## Workspace

The private npm workspace is rooted at `npm/`, with its single `package-lock.json`. It owns `packages/native-node`, `packages/wasm`, `packages/typescript`, `packages/react`, and `packages/vue`; cross-backend tests live under `tests/cross-backend`. The four platform-specific native packages live beside `packages/native-node` as `packages/native-node-*` and remain outside the workspace because their `os`/`cpu` constraints must not make a cross-platform install fail. Their tarballs are built separately on matching runners. The native package's source workspace manifest keeps those unpublished optional packages out of the lockfile; `scripts/pack.mjs` injects the coordinated optional-dependency metadata from `platform-dependencies.json` into the distributable tarball.

Install and validate locally with:

```sh
cd npm
npm ci --ignore-scripts --omit=optional
npm ls --workspaces --depth=0
npm test
npm run pack:dry-run
```

The workspace uses Changesets for independent package versions and coordinated changelogs. Add a changeset for consumer-visible package changes with `npm run changeset`; do not add changesets for repository-only maintenance that does not alter published package contents.

The planned `@corrplot-ng/core` package is not yet present. The initial `@corrplot-ng/typescript` facade and its public normalized-matrix scene workflow are implemented under #145. The React and Vue adapters are implemented under #291 and #292 as separate browser-only workspace packages. The current facade remains named `@corrplot-ng/typescript`; renaming it is a separately reviewed migration, not part of workspace setup.

## Current implementation status — 2026-11-04

The workspace and release-preparation infrastructure tracked by #290 are implemented. The current npm workspace package set is `@corrplot-ng/native-node`, `@corrplot-ng/wasm`, `@corrplot-ng/typescript`, `@corrplot-ng/react`, and `@corrplot-ng/vue`. The four platform-specific native packages are built as separate tarballs rather than workspace entries. The root workspace remains private.

The native Node and browser WASM packages expose matrix validation, normalized-matrix scene construction, typed-scene SVG/RGBA operations, matrix-bearing scene-v1 SVG/RGBA operations, renderer capabilities, and diagnostic-v1 errors. The TypeScript facade validates array or typed-array `MatrixInput`, creates a renderer-neutral scene through the shared Rust builder, renders that scene to SVG/RGBA, retains matrix-bearing scene-v1 JSON rendering for compatibility, verifies an optional renderer ID, supports adapter injection, and disposes the backend. Node ESM/CommonJS and browser conditional-export usage are type-checked; runtime smoke tests exercise the native backend and a bundled browser/WASM facade.

The scene-v1 parser used by the bindings currently supports the matrix-bearing subset, not every presentation field in the full scene-v1 contract. The browser smoke example draws returned RGBA bytes into a host-owned Canvas; it is not the direct scene-to-Canvas renderer tracked by #144. The React adapter under #291 and Vue adapter under #292 are implemented as browser-only packages.

Public npm publication remains explicitly disabled and requires separate maintainer approval under #293.

## Target package topology

| Package | Responsibility | Runtime dependencies |
| --- | --- | --- |
| `@corrplot-ng/core` | Public TypeScript types, diagnostics, and runtime-neutral contracts | None; planned |
| `@corrplot-ng/wasm` | Browser execution backend and WASM assets | WASM/browser only |
| `@corrplot-ng/native-node` | Node.js execution backend | Node-API addon and matching platform package |
| `@corrplot-ng/react` | Optional React component and hook | `@corrplot-ng/wasm`; React peer dependency; implemented |
| `@corrplot-ng/vue` | Optional Vue component and composable | `@corrplot-ng/wasm`; Vue peer dependency; implemented |
| `@corrplot-ng/typescript` | Initial runtime-selecting facade | Optional peer dependencies for the Node and WASM backends |

The Node package already has an implementation under `npm/packages/native-node/` and platform-specific package names. Preserve these names unless a separate migration is reviewed. Do not rename or recreate working native package artifacts as part of package-workspace preparation.

The browser package must not depend on, optionally install, import, or bundle the Node native package. React and Vue integrations must depend only on the browser-facing API. Frameworks remain optional peer dependencies and must not become dependencies of core or WASM.

## API consistency versus runtime isolation

The Node and browser packages expose the same facade operations where their renderer capabilities overlap: validation, normalized-matrix scene construction, typed-scene SVG/RGBA rendering, matrix-bearing scene-v1 compatibility rendering, capability lookup, and disposal. The facade normalizes typed-array inputs to the same row-major array representation before calling either backend.

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
4. Stages the React and Vue adapters after the browser WASM package.
5. Stages the TypeScript facade after its runtime backends.
6. Assembles all nine tarballs into one CI artifact, records SHA-256 checksums, and documents the dependency order.

The staging workflow includes the React and Vue adapter tarballs after WASM and before the TypeScript facade. The current bundle contains nine tarballs: four native platform packages, WASM, the main Node package, React, Vue, and the TypeScript facade.

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

- React integration: #291 — implemented in `npm/packages/react/`. It exports `Corrplot` and `useCorrplot`, initializes the WASM backend after mount, ignores results from cleaned-up effects, and renders inline SVG. React is a peer dependency; the only runtime package dependency is `@corrplot-ng/wasm`. Vite and esbuild production bundles are checked for Node-only dependencies, and browser smoke tests cover Strict Mode, updates, unmount/remount, initialization failures, and rendering errors.
- Vue integration: #292 — implemented in `npm/packages/vue/` as a browser-only component and composable over `@corrplot-ng/wasm`; Vue is a peer dependency. It remains a thin browser host adapter and does not implement correlation or rendering semantics.

Framework packages must not pull native Node artifacts into browser dependency graphs.

## Work breakdown

- #145 — shared TypeScript API contract and runtime package isolation (**normalized-matrix scene workflow, typed-scene SVG/RGBA, typed-array normalization, runtime isolation, Node ESM/CommonJS and browser type checks, and native/browser runtime smoke coverage implemented**).
- #290 — workspace, versioning, CI, and non-publishing tarball/release preparation (**complete**).
- #291 — React integration (**component, hook, browser-only WASM dependency, Vite/esbuild bundling, and browser lifecycle/error coverage implemented**).
- #292 — Vue integration (**component, composable, reactive updates, lifecycle/error handling, browser-only bundling, and non-publishing package staging implemented**).
- #293 — later npm publication; explicitly deferred.

The initial Node and WASM backend implementation slices in #141 and #142 are complete. The runtime-isolated TypeScript facade and normalized-matrix scene workflow in #145 are implemented. Full scene-v1 presentation semantics and broader cross-backend conformance remain tracked separately by #146; direct Canvas remains #144. Extend the existing packages rather than restarting them.
