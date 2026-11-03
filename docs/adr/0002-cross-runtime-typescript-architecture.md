# ADR 0002: Cross-runtime Rust and TypeScript architecture

- **Status:** Accepted
- **Date:** 2026-10-31
- **Decision owners:** corrplot-ng maintainers
- **Related:** [ADR 0001](0001-shared-core-architecture.md), [#139](https://github.com/vsimko/corrplot-ng/issues/139), [#51](https://github.com/vsimko/corrplot-ng/issues/51), [#71](https://github.com/vsimko/corrplot-ng/issues/71), [#115](https://github.com/vsimko/corrplot-ng/issues/115), [#140](https://github.com/vsimko/corrplot-ng/issues/140), [#141](https://github.com/vsimko/corrplot-ng/issues/141), [#142](https://github.com/vsimko/corrplot-ng/issues/142), [#143](https://github.com/vsimko/corrplot-ng/issues/143), [#144](https://github.com/vsimko/corrplot-ng/issues/144), [#145](https://github.com/vsimko/corrplot-ng/issues/145)

## Context

ADR 0001 selects Rust as the authoritative implementation of shared correlation and visualization semantics. Merged PR #180 proves that the native CLI and the disposable R/rextendr adapter call the same `crates/corrplot-core` matrix-validation operation. The core also has typed-scene SVG and direct RGBA renderers. PR #237 adds the dependency-free scene-v1 parser and the initial normalized-matrix/scene-v1 construction path. The shared core applies deterministic word wrapping for `label_collision: wrap` and rejects plotmath label expressions until a renderer-neutral representation is implemented; the document description is retained and emitted as SVG `<desc>`, and figure-text/cell-annotation roles are preserved as SVG semantic labels.

The project intends to expose the same shared implementation to Node.js and browser applications. Those environments have different loading, memory, packaging, and host-integration requirements. They must not acquire separate implementations of matrix validation, ordering, scene construction, or rendering semantics.

The repository already has:

- scene-v1 and normalized-matrix v1 as language-neutral contracts;
- renderer capability negotiation in #71, with a machine-readable schema and backend-specific declarations;
- diagnostic-v1 in #72 for stable errors and warnings;
- scene-v1 conformance evidence in #115;
- a Rust CLI and a reusable Rust crate.

At the time this ADR was accepted, the Node, WebAssembly, and TypeScript layers described below were design targets. Their current implementation status is recorded separately so that the original architectural decision remains intact without presenting completed work as planned.

## Decision

**Keep all shared correlation and visualization semantics in Rust. Expose the same Rust core to Node.js through Node-API and to browsers through WebAssembly. Publish one TypeScript-facing API that selects the appropriate execution backend without changing renderer semantics.**

### Implementation status — 2026-11-04

The current implementation consists of four layers:

1. **Rust core — `crates/corrplot-core`:** implements shared matrix validation, the matrix-bearing scene-v1 parser and normalized-matrix-to-scene construction path, typed-scene SVG rendering, and RGBA raster rendering. The scene-v1 parser currently supports the matrix-bearing subset used by the bindings; it is not a general implementation of every scene-v1 presentation field.
2. **Native Node binding — `bindings/node/`:** a Node-API adapter exposing matrix validation, typed-scene SVG/RGBA rendering, matrix-bearing scene-v1 SVG/RGBA rendering, renderer capabilities, and diagnostic-v1 mapping. Its CI covers Node.js 22, 24, and 26 on Linux, macOS, and Windows.
3. **Browser WASM binding — `bindings/wasm/`:** a `wasm32-unknown-unknown` adapter exposing the same validation, typed-scene and matrix-bearing SVG/RGBA operations, capability data, and structured diagnostics. RGBA bytes are copied into JavaScript-owned `Uint8Array` memory. Browser smoke coverage runs on Chromium, Firefox, and WebKit; the WASM Node tests run on Node.js 24 and 26.
4. **TypeScript facade — `packages/typescript/`:** an implemented but deliberately initial facade with separate Node and browser entry points. It asynchronously loads the matching native or WASM backend and exposes validation, matrix-bearing scene-v1 rendering to SVG/RGBA, renderer capabilities, diagnostic errors, adapter injection, and disposal. Its public API is still narrower than the target contract in this ADR; see the current API section below and issue #145.

The npm workspace currently contains `@corrplot-ng/native-node`, `@corrplot-ng/wasm`, and `@corrplot-ng/typescript`. The runtime-neutral `@corrplot-ng/core` package and the React/Vue adapters remain planned. These implementation notes do not imply a production npm release, full scene-v1 presentation support, or direct Canvas rendering.

## Runtime and backend selection

The public factory is asynchronous in both environments so callers use the same call pattern and WebAssembly initialization is explicit:

```ts
const engine = await createCorrplotEngine({ backend: "auto" });
const sceneV1Json = /* matrix-bearing scene-v1 JSON string */;
const svg = engine.render(sceneV1Json, { format: "svg", method: "circle" });
```

The supported backend options are:

- `auto`: select the native Node binding when running in Node.js; select WebAssembly in a supported browser.
- `native`: require the Node-API binding. It is unavailable in browsers.
- `wasm`: require the browser WebAssembly binding. It is not part of the initial Node.js support contract.

Backend selection is an execution decision, not an output-format decision. The facade must use package conditional exports or separate environment entry points so browser bundlers do not load native binaries and Node-only deployments do not fetch browser WASM assets.

Selection must be deterministic. Do not silently fall back from a requested backend or switch implementations after a load failure. Return an actionable diagnostic that identifies the requested backend, runtime, and available alternatives. In `auto` mode, failure to load the preferred backend is an error; it must not silently change performance or feature characteristics.

The facade may accept an injected backend in tests and advanced integrations. Runtime detection must not depend on user-agent strings. Do not assume that a browser bundler provides Node globals such as `process`.

## Public TypeScript API boundary

The API described in the original decision is the target contract. The facade currently implements only an initial subset (verified against `packages/typescript/src/index.d.ts`, `factory.js`, `node.js`, and `browser.js` on 2026-11-04).

### Implemented facade

- `createCorrplotEngine({ backend, adapter })`: asynchronously creates an engine. The Node entry point accepts `auto` or `native`; the browser entry point accepts `auto` or `wasm`. An injected `adapter` is available for tests and advanced integrations.
- `engine.validate(input)`: accepts a `MatrixInput` with `labels`, `rows`, `columns`, `values: Array<number | null>`, and `kind`. It returns `true` or throws `CorrplotError` containing a diagnostic-v1 record. A `null` value represents a missing cell.
- `engine.render(sceneV1Json, options)`: accepts a JSON string containing the matrix-bearing scene-v1 subset; it does not accept a `MatrixInput` or a typed scene object. `format: "svg"` returns an SVG string. `format: "rgba"` returns an object containing an owned `Uint8Array` and width, height, stride, pixel-format, alpha-mode, color-space, and optional DPI metadata. The supported cell methods are `circle`, `square`, `ellipse`, `number`, `shade`, `color`, and `pie`. RGBA options require positive integer `widthPx` and `heightPx` plus a background color; SVG options currently expose the optional cell method.
- `engine.getRendererCapabilities()`: returns the capability-v1 declaration. The current method takes no renderer ID.
- `engine.dispose()`: releases the adapter when supported, is idempotent, and prevents subsequent engine operations.
- `CorrplotError`: exposes the backend diagnostic through its `diagnostic` property.

The lower-level Node and WASM bindings also expose typed-scene `renderSceneSvg()` and `renderSceneRgba()` operations. Those operations are not yet surfaced by the TypeScript facade.

### Remaining API work

Issue #145 remains open. In particular, the facade does not yet expose `engine.createScene()`, a renderer-ID argument to `getRendererCapabilities()`, typed-array matrix input, or a direct normalized-matrix-to-render operation. The intended input, scene-construction, memory, and output contracts below remain design requirements rather than claims about the current facade.

The public TypeScript types must not expose internal Rust structs, pointers, N-API handles, or WebAssembly memory offsets. The shared Rust core remains the only source of correlation, normalization, ordering, and scene-construction semantics.

### Matrix input and memory ownership

- The current facade's `MatrixInput.values` is a JavaScript array of numbers or explicit `null` missing values; typed-array matrix input is not implemented in this facade yet.
- NaN and infinities are not substitutes for missing values; they are invalid observed values in the shared core.
- The bindings convert inputs into Rust-owned values. The WASM RGBA path copies output into JavaScript-owned typed-array memory; callers may retain the returned pixels independently of later WASM calls. The Node binding returns a Node Buffer, which is a `Uint8Array` subclass.
- Do not expose long-lived views into WebAssembly linear memory or promise zero-copy behavior without an explicit ownership contract.
- Keep checked dimension and allocation handling in the shared core and binding boundary.

## Execution backend and renderer separation

Execution backend and output renderer are independent axes:

| Concern | Examples | Owner |
| --- | --- | --- |
| Execution backend | Node-API native, browser WebAssembly | TypeScript facade and binding adapters |
| Output renderer | SVG and RGBA raster (current facade formats); direct browser Canvas 2D (not implemented) | Renderer implementations and host adapters |
| Renderer capability | Supported scene versions, cell methods, annotations, metadata | Existing renderer capability-v1 contract (#71) |
| Interchange/conformance | normalized-matrix v1, scene-v1 | Versioned language-neutral contracts |

The native and WASM backends must call the same Rust semantic implementation. They must not maintain separate renderer capability schemas. The existing capability-v1 declaration remains authoritative; each renderer reports only the features it actually preserves. Capability negotiation remains strict by default, and unsupported required features fail before output is written.

The renderer-neutral contract in #153 and the RGBA raster implementation in #154 are complete. The current TypeScript facade advertises only `svg` and `rgba` formats; it does not advertise a PNG format. Direct scene-to-Canvas rendering remains a separate browser adapter in #144 and is not implemented. Canvas and DOM objects remain in TypeScript; Rust must not depend on browser object types. Canvas must consume the shared scene or a documented renderer command representation directly, not serialize to SVG and parse it again.

The output contract defines pixel-buffer layout, dimensions, DPI, alpha, and background semantics. The remaining renderer work is integration and conformance, not initial contract definition.

## Portability, initialization, and concurrency

- The Rust core must compile for the supported native targets and `wasm32-unknown-unknown` from the same source. Target-specific code may adapt I/O or host calls, but must not fork algorithms.
- Keep filesystem, environment-variable, process, dynamic-library loading, and CLI concerns outside the portable core.
- WebAssembly module loading is asynchronous. The browser package must expose bundler-friendly ESM exports and an explicit initialization error when the module or its assets cannot be loaded.
- The initial WASM backend is single-threaded. Do not require `SharedArrayBuffer`, cross-origin isolation headers, WebAssembly threads, SIMD, WebGPU, or OffscreenCanvas for the baseline.
- Applications may run the engine inside a Web Worker. Worker creation and message transport are host integration choices, not implicit behavior of the core API.
- OffscreenCanvas support belongs to #144 and must be feature-detected. The initial WASM engine must also work without a worker or OffscreenCanvas.
- Use checked sizes and explicit allocation limits for untrusted or very large matrices. Return a diagnostic rather than relying on allocation failure or integer overflow.
- No Rust panic may unwind across N-API or WebAssembly boundaries. Convert expected failures to diagnostic-v1; contain unexpected panics at the binding boundary where the target's panic strategy supports unwinding. A panic must never become an undocumented alternate result.

## Errors and diagnostics

Use the existing diagnostic-v1 schema from #72 as the cross-runtime error contract.

- The Node and WASM bindings map Rust failures to diagnostic-v1 records. The TypeScript facade wraps backend diagnostics in `CorrplotError`, whose `diagnostic` property contains the record.
- The current facade treats a returned `diagnostic` as an error. It does not yet expose a success-result envelope for structured warnings.
- The current facade's own unavailable-backend, initialization-failure, and post-disposal errors use `configuration.invalid_field` at the `configuration` stage. Initialization context is included in the message and `backend` path; a dedicated backend-initialization diagnostic with structured runtime/backend details remains part of the #145 contract work.
- The intended cross-runtime contract remains stable codes and structured diagnostics; do not make callers parse backend-specific message text.
- Unexpected internal failures must not expose Rust panic payloads as a public API.

The diagnostic schema and catalogue remain authoritative. Do not add a second TypeScript-only or WASM-only error model.

## Compatibility matrix

The supported runtime matrix is a moving support policy. CI must test the versions listed here and the release documentation must record the exact versions used for each release.

| Environment | Initial support policy | Backend | Required baseline |
| --- | --- | --- | --- |
| Node.js 26.x | Active LTS line as of 2026-10-31 | Native Node-API | N-API ABI; supported package binary or documented source-build path |
| Node.js 24.x | Active LTS line as of 2026-10-31 | Native Node-API | Same public API and semantic fixtures as Node 26 |
| Node.js 22.x | Maintenance LTS line as of 2026-10-31 | Native Node-API | Same public API and semantic fixtures as Node 26 |
| Chrome and Edge | Current and previous stable major releases | Browser WASM | WebAssembly MVP, ES modules, typed arrays, Promises |
| Firefox | Current and previous stable major releases | Browser WASM | WebAssembly MVP, ES modules, typed arrays, Promises |
| Safari on macOS and iOS | Current and previous stable major releases | Browser WASM | WebAssembly MVP, ES modules, typed arrays, Promises |

Do not support Node.js 27 Alpha or end-of-life Node lines for production releases. Re-evaluate this matrix when Node release lines change; the official [Node.js release schedule](https://nodejs.org/en/about/previous-releases) is authoritative. The Node.js release table on 2026-10-31 lists 26.x and 24.x as Active LTS and 22.x as Maintenance LTS. The package must not pin itself to one patch release.

The browser policy deliberately targets evergreen browsers and WebAssembly MVP. Current browser smoke automation runs Chromium, Firefox, and WebKit on Ubuntu 24.04; WebKit automation is not proof of Safari packaging behavior, and Edge/mobile packaging is not established by that matrix. Exact browser versions are selected from the CI images and should be recorded in release evidence. A real Safari smoke test is still required before claiming Safari release support. Legacy browsers and embedded webviews are outside the initial support contract.

## Testing and release gates

The following evidence is already present:

- Native Node binding tests run on Node.js 22, 24, and 26 across Linux, macOS, and Windows.
- WASM package tests run on Node.js 24 and 26. Browser smoke tests exercise Chromium, Firefox, and WebKit on Ubuntu 24.04 using the actual generated WASM package.
- The Node/WASM parity suite compares matrix validation, typed-scene SVG/RGBA, matrix-bearing SVG/RGBA across the shared fixtures and seven cell methods, renderer capabilities, and representative diagnostics.
- TypeScript facade tests cover injected-adapter validation, SVG/RGBA rendering, diagnostic conversion, RGBA dimension checks, capability access, and idempotent disposal.

Remaining release/conformance gates include:

- Complete the public TypeScript API and package-isolation contract in #145; the current facade is an initial subset, not the full target API.
- Implement direct scene-to-Canvas rendering in #144.
- Expand full-scene semantic and visual regression coverage across the backends in #146. The existing Node/WASM parity tests do not establish complete legacy corrplot or full presentation-scene parity.
- Keep performance thresholds and noisy measurements informational until the broader evidence and repeatability work in #146 supports stable budgets.
- Preserve scene-v1 and diagnostic-v1 compatibility. A runtime binding must not silently change either contract.

## Relationship to existing issues

- **#51 — interactive exploration:** remains separate. This ADR defines the rendering/core boundary, not selection, filtering, navigation, scatterplot links, or interactive state.
- **#71 and #72 — capabilities and diagnostics:** their contracts are implemented and reused by the bindings.
- **#115 — scene-v1 independence:** complete. Keep scene-v1 as the interchange and conformance format.
- **#140 — portability:** complete; the same Rust core is built for native and WASM targets.
- **#141 and #142 — Node/WASM bindings:** the initial binding slices are complete. This does not mean a production release or full API parity.
- **#143/#153 and #154 — renderer contract and RGBA raster:** complete. SVG and RGBA are available in the current binding APIs.
- **#236 — scene-v1 parsing and matrix-to-scene construction:** complete for the matrix-bearing subset used by the current bindings; full presentation-scene semantics remain outside that initial slice.
- **#144 — direct Canvas:** not implemented; remains a separate browser adapter.
- **#145 — unified TypeScript API:** active. Complete the facade beyond its current validation and matrix-bearing SVG/RGBA subset, and verify the intended package-isolation and diagnostics contract.
- **#146 — broad parity and visual regression:** active; it remains the broader conformance/performance gate.
- **#290 — npm workspace and non-publishing staging:** complete. Public publication remains separately deferred under #293.
- **#291 and #292 — React and Vue:** framework adapters remain follow-up work and depend on the browser-facing API contract in #145.
- **#173 — CRAN and human-maintainer evidence:** remains a separate R release-readiness gate and does not block Node/WASM/TypeScript engineering.

## Rejected alternatives

### Independent TypeScript implementation of correlation semantics

Rejected. It would create a second source of truth for validation, normalization, ordering, and scene construction and would allow Node/browser behavior to drift from the CLI and R consumers.

### Scene-v1 JSON as the only connection to Node and browser

Rejected as the production execution architecture. Scene-v1 remains valuable for interchange, reproducible fixtures, and conformance, but passing JSON between independently implemented algorithms would not satisfy the shared-library requirement in ADR 0001.

### Direct V8/Node internal ABI binding

Rejected in favor of Node-API. Node-API provides a stable ABI boundary across supported Node releases and avoids coupling the binding to V8 internals.

### Separate native and WASM algorithms

Rejected. Both targets must compile the same Rust semantic code. Only the binding, loading, and host-integration layers may differ.

### Browser threads and mandatory OffscreenCanvas in the first version

Deferred. They add cross-origin isolation, worker, browser-support, and deployment constraints. Start with a single-threaded WASM baseline and add these capabilities only when measured workloads justify them.

## Consequences

- Node.js and browser users receive one documented TypeScript API and the same semantic behavior.
- Native and browser deployments require separate binding artifacts but do not maintain separate algorithms.
- The TypeScript facade and binding packages add build and release work. Keep them thin and test their outputs against the shared Rust fixtures.
- The core now parses the matrix-bearing scene-v1 subset and constructs normalized-matrix scenes for the initial Node/WASM SVG and RGBA operations. The full presentation-scene contract and broader semantic parity remain incomplete; the TypeScript facade is also still a partial implementation of the target API.
- Production R adoption remains separately gated by the CRAN and maintainer-workflow conditions in ADR 0001.


## Package distribution and publication boundary

The npm dependency graphs are intentionally separate. A browser consumer must not install or bundle Node native binaries merely because the same semantic core is also available in Node.js.

The current workspace contains these installable packages:

- `@corrplot-ng/native-node`: Node-API backend and its platform-specific implementation packages.
- `@corrplot-ng/wasm`: browser/WASM backend; no Node native dependency or Node-only runtime imports.
- `@corrplot-ng/typescript`: the initial runtime-selecting facade with optional peer dependencies for the Node and WASM backends.

The following packages remain planned, not current workspace packages:

- `@corrplot-ng/core`: runtime-neutral public types, diagnostics, and API contracts only; no native or WASM backend dependency.
- `@corrplot-ng/react` and `@corrplot-ng/vue`: optional framework adapters that depend only on the browser-facing API; React/Vue are peer dependencies.

The existing Node package and platform package names under `bindings/node/` should be preserved unless a separately reviewed migration is justified. A universal package that installs both execution backends is rejected: conditional exports alone do not prevent npm from installing declared dependencies.

The current `@corrplot-ng/typescript` facade selects the runtime-specific backend through separate Node and browser entry points. Its current API is partial: it validates `MatrixInput`, renders matrix-bearing scene-v1 JSON to SVG or RGBA, returns renderer capabilities, wraps diagnostics, supports adapter injection, and disposes the adapter. It does not yet expose the complete `createScene`/normalized-matrix rendering contract described in this ADR; that remains #145. API consistency is the target, not a claim that all target operations are already exposed.

### Staged release policy

The workspace, Changesets versioning, pull-request validation, and manual non-publishing tarball staging have been implemented under #290. The current release stage remains **non-publishing**. The workflow must not call `npm publish`, use npm credentials, request OIDC publication permissions, or publish automatically from tags.

When publication is later prioritized, publish according to the actual dependency graph: runtime-neutral types/core first if that package is implemented, then WASM and Node platform packages, then the main Node package, then React/Vue integrations. npm publication is not transactional; the future workflow must detect already-published versions and safely handle partial failures.

Do not configure live npm Trusted Publishing or credentials until a separate maintainer-approved release issue is activated. The detailed plan is in [npm package distribution](../npm-package-distribution.md). #145 tracks the remaining facade/API contract, #291 and #292 track framework adapters, #290 is complete, and public publication remains deferred in #293.
