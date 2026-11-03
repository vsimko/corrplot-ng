# npm package distribution and release preparation

**Status:** Planned architecture; implementation tracked by #290–#293.  
**Publication status:** Explicitly deferred. No package in this plan is authorized for public npm publication yet.

## Goals

Prepare a coherent TypeScript ecosystem for Node.js and browser consumers while keeping runtime dependencies isolated. Package builds, tests, versioning, and tarball staging can be implemented now; registry publication is a later maintainer-approved activity.

The shared API contract and semantics should be consistent across runtimes. That does not require one physical npm package or one dependency graph containing every backend.

## Target package topology

| Package | Responsibility | Runtime dependencies |
| --- | --- | --- |
| `@corrplot-ng/core` | Public TypeScript types, diagnostics, and runtime-neutral contracts | None |
| `@corrplot-ng/wasm` | Browser execution backend and WASM assets | WASM/browser only |
| `@corrplot-ng/native-node` | Node.js execution backend | Node-API addon and matching platform package |
| `@corrplot-ng/react` | Optional React components/hooks | React peer dependency and browser API |
| `@corrplot-ng/vue` | Optional Vue components/composables | Vue peer dependency and browser API |

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

Use Changesets or a maintained equivalent to support independent package versions, dependency-aware version bumps, and generated changelogs. Packages may share a release train without being forced to use identical version numbers.

The non-publishing workflow should:

1. Validate workspace manifests, package names, dependency ranges, exports, and peer dependencies.
2. Build Rust Node/WASM bindings and TypeScript packages on their supported targets.
3. Run shared semantic, API, type, and runtime-isolation tests.
4. Run package-specific clean-install smoke tests.
5. Run `npm pack --dry-run` and stage actual tarballs as CI artifacts.
6. Record package versions, dependency graph, tarball checksums, and build evidence.
7. Preserve a deterministic dependency order so a future publication workflow can use the staged release plan.

The workflow must not run `npm publish`, must not use npm credentials, must not request OIDC publication permissions, and must not publish automatically from tags. The current stage is build/test/package only.

## Future publication order

When a public release is explicitly approved, the intended order is:

1. Runtime-neutral core/types package.
2. Browser WASM package and Node platform-specific native packages.
3. Main Node package after its platform packages are available.
4. React and Vue integrations after their browser-facing dependencies.

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
