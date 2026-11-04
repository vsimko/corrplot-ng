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
4. **TypeScript facade — `packages/typescript/`:** a shared facade with separate Node and browser entry points. It asynchronously loads only the matching native or WASM backend; validates row-major matrix input (arrays or numeric typed arrays with an explicit missing mask); constructs a renderer-neutral scene in Rust; renders that scene to SVG/RGBA; exposes renderer capabilities and diagnostic errors; and supports adapter injection and disposal. It retains a compatibility path for matrix-bearing scene-v1 JSON. The scene builder currently covers the normalized-matrix subset described below, not the full scene-v1 presentation contract.

The npm workspace currently contains `@corrplot-ng/native-node`, `@corrplot-ng/wasm`, `@corrplot-ng/typescript`, and the browser-only `@corrplot-ng/react` and `@corrplot-ng/vue` adapters. The runtime-neutral `@corrplot-ng/core` remains planned. These implementation notes do not imply a production npm release, full scene-v1 presentation support, or direct Canvas rendering.

## Runtime and backend selection

The public factory is asynchronous in both environments so callers use the same call pattern and WebAssembly initialization is explicit:

```ts
const engine = await createCorrplotEngine({ backend: "auto" });
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.65, 0.65, 1],
  kind: "correlation",
};
const scene = engine.createScene(matrix, { cellMethod: "circle" });
const svg = engine.render(scene, { format: "svg" });
```

The supported backend options are:

- `auto`: select the native Node binding when running in Node.js; select WebAssembly in a supported browser.
- `native`: require the Node-API binding. It is unavailable in browsers.
- `wasm`: require the browser WebAssembly binding. It is not part of the initial Node.js support contract.

Backend selection is an execution decision, not an output-format decision. The facade must use package conditional exports or separate environment entry points so browser bundlers do not load native binaries and Node-only deployments do not fetch browser WASM assets.

Selection must be deterministic. Do not silently fall back from a requested backend or switch implementations after a load failure. Return an actionable diagnostic that identifies the requested backend, runtime, and available alternatives. In `auto` mode, failure to load the preferred backend is an error; it must not silently change performance or feature characteristics.

The facade may accept an injected backend in tests and advanced integrations. Runtime detection must not depend on user-agent strings. Do not assume that a browser bundler provides Node globals such as `process`.

## Public TypeScript API boundary

The facade exposes the shared normalized-matrix scene workflow in both runtime entry points. Its concrete surface is verified against `packages/typescript/src/index.d.ts`, `factory.js`, `node.js`, and `browser.js` on 2026-11-04.

### Implemented facade

- `createCorrplotEngine({ backend, adapter })`: asynchronously creates an engine. The Node entry point accepts `auto` or `native`; the browser entry point accepts `auto` or `wasm`. An injected `adapter` is available for tests and advanced integrations.
- `engine.validate(input)`: accepts a `MatrixInput` with `labels`, `rows`, `columns`, `values`, and `kind`. It returns `true` or throws `CorrplotError` containing a diagnostic-v1 record. Values may be a JavaScript array of numbers/`null` or a numeric typed array; a typed array uses the optional boolean `missingMask` (`true` means missing).
- `engine.createScene(input, options)`: passes normalized matrix input and presentation options to the shared Rust `matrix_to_scene_v1` builder. It returns a plain renderer-neutral scene containing dimensions, description, cells, glyphs, and text annotations.
- `engine.render(scene, options)`: renders that scene through the shared typed-scene renderer. `format: "svg"` returns an SVG string; `format: "rgba"` returns an object containing an owned `Uint8Array` and width, height, stride, pixel-format, alpha-mode, color-space, and optional DPI metadata. The supported cell methods are `circle`, `square`, `ellipse`, `number`, `shade`, `color`, and `pie`. RGBA options require positive integer `widthPx` and `heightPx` plus a background color.
- `engine.render(sceneV1Json, options)`: retains the compatibility path for a JSON string containing the matrix-bearing scene-v1 subset.
- `engine.getRendererCapabilities(rendererId?)`: returns the capability-v1 declaration and, when an ID is supplied, verifies it against the selected renderer's declared ID.
- `engine.dispose()`: releases the adapter when supported, is idempotent, and prevents subsequent engine operations.
- `CorrplotError`: exposes the backend diagnostic through its `diagnostic` property.

The Node and WASM bindings expose the same typed-scene and matrix-bearing scene-v1 operations used by the facade. The normalized-matrix builder currently produces the base scene subset; full presentation-scene semantics, significance/legend/cluster overlays from arbitrary input, and direct Canvas rendering remain separate work.

### Remaining API work

The initial cross-runtime facade, package isolation, typed-array normalization, renderer-ID checking, and runtime-specific initialization are implemented under #145. The full scene-v1 presentation contract is not implied by this initial normalized-matrix builder; broader semantics and visual conformance remain tracked by #146 and the direct Canvas adapter by #144.

The public TypeScript types must not expose internal Rust structs, pointers, N-API handles, or WebAssembly memory offsets. The shared Rust core remains the only source of correlation, normalization, ordering, and scene-construction semantics.

### Matrix input and memory ownership

- The facade accepts JavaScript arrays of numbers/`null` and numeric typed arrays. For typed arrays, use `missingMask` to represent missing cells explicitly; `true` marks a missing value. The facade normalizes this input to the binding's row-major `Array<number | null>` representation before crossing the native/WASM boundary.
- NaN and infinities are not substitutes for missing values; they remain invalid observed values in the shared core.
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
- The current facade's own unavailable-backend, initialization-failure, and post-disposal errors use `configuration.invalid_field` at the `configuration` stage. Initialization context is included in the message and `backend` path. A dedicated diagnostic code with structured runtime/backend fields is not currently exposed; the generic diagnostic remains deterministic and actionable for the implemented API slice.
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
- The Node/WASM parity suite compares matrix validation, normalized-matrix scene construction, typed-scene SVG/RGBA, matrix-bearing SVG/RGBA across the shared fixtures and seven cell methods, renderer capabilities, and representative diagnostics.
- TypeScript facade tests cover injected-adapter validation, scene creation, typed-array normalization, SVG/RGBA rendering, diagnostic conversion, renderer-ID checking, RGBA dimension checks, and idempotent disposal.
- TypeScript declaration fixtures compile Node ESM, Node CommonJS, and browser conditional-export usage examples. A bundled browser-facade smoke test runs the actual WASM backend in Chromium, Firefox, and WebKit; the Node facade integration test runs against the actual native addon.

Remaining release/conformance gates include:
- Direct scene-to-Canvas rendering is implemented in #144. The Canvas renderer supports the documented common feature subset and reports unsupported legends, cluster regions, and plotmath through its capability declaration and diagnostics.
- Continue expanding full-scene semantic and visual regression coverage across the backends in #146. The existing Node/WASM parity tests do not establish complete legacy corrplot or full presentation-scene parity.
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
- **#144 — direct Canvas:** implemented as a browser TypeScript adapter. It renders the shared typed scene directly and supports the declared Canvas feature subset, including supplied significance annotations and confidence intervals.
- **#145 — unified TypeScript API:** the initial shared facade, normalized-matrix scene builder, typed-scene rendering, typed-array normalization, renderer-ID checking, package isolation, and Node/browser runtime tests are implemented. Full scene-v1 presentation parity remains outside this issue's initial API slice.
- **#146 — broad parity and visual regression:** active; it remains the broader conformance/performance gate.
- **#290 — npm workspace and non-publishing staging:** complete. Public publication remains separately deferred under #293.
- **#291 — React:** implemented in `packages/react/` as a browser-only component and hook over `@corrplot-ng/wasm`; React is a peer dependency. Vite/esbuild bundles and browser lifecycle/error behavior are tested.
- **#292 — Vue:** implemented in `packages/vue/` as a browser-only component and composable over `@corrplot-ng/wasm`; Vue is a peer dependency.
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
- The core parses the matrix-bearing scene-v1 subset and constructs normalized-matrix scenes for Node/WASM SVG and RGBA operations. The TypeScript facade now exposes the initial normalized-matrix scene workflow and typed-scene rendering through runtime-isolated Node/browser entry points. Full scene-v1 presentation semantics and broader semantic/visual parity remain incomplete.
- Production R adoption remains separately gated by the CRAN and maintainer-workflow conditions in ADR 0001.


## Package distribution and publication boundary

The npm dependency graphs are intentionally separate. A browser consumer must not install or bundle Node native binaries merely because the same semantic core is also available in Node.js.

The current workspace contains these installable packages:

- `@corrplot-ng/native-node`: Node-API backend and its platform-specific implementation packages.
- `@corrplot-ng/wasm`: browser/WASM backend; no Node native dependency or Node-only runtime imports.
- `@corrplot-ng/typescript`: the initial runtime-selecting facade with optional peer dependencies for the Node and WASM backends.
- `@corrplot-ng/react`: browser-only React component and hook; it depends directly on `@corrplot-ng/wasm` and declares React as a peer dependency.

The following package remains planned, not a current workspace package:

- `@corrplot-ng/core`: runtime-neutral public types, diagnostics, and API contracts only; no native or WASM backend dependency.

The existing Node package and platform package names under `bindings/node/` should be preserved unless a separately reviewed migration is justified. A universal package that installs both execution backends is rejected: conditional exports alone do not prevent npm from installing declared dependencies.

The current `@corrplot-ng/typescript` facade selects the runtime-specific backend through separate Node and browser entry points. It validates array or typed-array `MatrixInput`, builds a renderer-neutral scene through the shared Rust core, renders that scene to SVG/RGBA, retains matrix-bearing scene-v1 JSON rendering for compatibility, returns renderer capabilities, wraps diagnostics, supports adapter injection, and disposes the adapter. Full scene-v1 presentation semantics and direct Canvas remain separate conformance/rendering work.

### Staged release policy

The workspace, Changesets versioning, pull-request validation, and manual non-publishing tarball staging have been implemented under #290. The current release stage remains **non-publishing**. The workflow must not call `npm publish`, use npm credentials, request OIDC publication permissions, or publish automatically from tags.

When publication is later prioritized, publish according to the actual dependency graph: runtime-neutral types/core first if that package is implemented, then WASM and Node platform packages, then the main Node package, then React/Vue integrations. npm publication is not transactional; the future workflow must detect already-published versions and safely handle partial failures.

Do not configure live npm Trusted Publishing or credentials until a separate maintainer-approved release issue is activated. The detailed plan is in [npm package distribution](../npm-package-distribution.md). The initial facade/API contract in #145 and browser-only React/Vue adapters #291/#292 are implemented; #290 is complete, and public publication remains deferred in #293.
