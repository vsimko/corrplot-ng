# ADR 0002: Cross-runtime Rust and TypeScript architecture

- **Status:** Accepted
- **Date:** 2026-10-31
- **Decision owners:** corrplot-ng maintainers
- **Related:** [ADR 0001](0001-shared-core-architecture.md), [#139](https://github.com/vsimko/corrplot-ng/issues/139), [#51](https://github.com/vsimko/corrplot-ng/issues/51), [#71](https://github.com/vsimko/corrplot-ng/issues/71), [#115](https://github.com/vsimko/corrplot-ng/issues/115), [#140](https://github.com/vsimko/corrplot-ng/issues/140), [#141](https://github.com/vsimko/corrplot-ng/issues/141), [#142](https://github.com/vsimko/corrplot-ng/issues/142), [#143](https://github.com/vsimko/corrplot-ng/issues/143), [#144](https://github.com/vsimko/corrplot-ng/issues/144), [#145](https://github.com/vsimko/corrplot-ng/issues/145)

## Context

ADR 0001 selects Rust as the authoritative implementation of shared correlation and visualization semantics. Merged PR #180 proves that the native CLI and the disposable R/rextendr adapter call the same `crates/corrplot-core` matrix-validation operation. The crate is currently small and does not yet implement the full scene-construction or rendering pipeline.

The project intends to expose the same shared implementation to Node.js and browser applications. Those environments have different loading, memory, packaging, and host-integration requirements. They must not acquire separate implementations of matrix validation, ordering, scene construction, or rendering semantics.

The repository already has:

- scene-v1 and normalized-matrix v1 as language-neutral contracts;
- renderer capability negotiation in #71, with a machine-readable schema and backend-specific declarations;
- diagnostic-v1 in #72 for stable errors and warnings;
- scene-v1 conformance evidence in #115;
- a Rust CLI and a reusable Rust crate.

This decision defines the cross-runtime boundary. It does not implement Node bindings, WebAssembly, Canvas, raster output, or the interactive experience in #51.

## Decision

**Keep all shared correlation and visualization semantics in Rust. Expose the same Rust core to Node.js through Node-API and to browsers through WebAssembly. Publish one TypeScript-facing API that selects the appropriate execution backend without changing renderer semantics.**

The implementation has four layers:

1. **Rust core — `crates/corrplot-core`:** owns common validation, normalization, ordering, scene construction, layout, and shared rendering algorithms as they are implemented. It must remain independent of Node.js, browser DOM APIs, filesystem access, CLI process handling, and graphics-device state.
2. **Native Node binding — proposed `bindings/node/`:** a thin Node-API (N-API) adapter. Prefer a maintained Rust N-API binding framework. It converts JavaScript inputs to Rust-owned values and maps Rust results and diagnostics to JavaScript.
3. **Browser WASM binding — proposed `bindings/wasm/`:** a thin `wasm32-unknown-unknown` adapter using `wasm-bindgen` or an equivalent maintained tool. It exposes the same core operations and diagnostic meanings as the native binding.
4. **TypeScript facade — proposed `packages/typescript/`:** owns the public TypeScript types, runtime entry points, backend selection, and host-specific convenience APIs. It must not reimplement correlation, normalization, ordering, scene construction, or rendering algorithms.

These paths describe the target package boundary. They are not a request to create empty packages or move existing files in this design issue.

## Runtime and backend selection

The public factory is asynchronous in both environments so callers use the same call pattern and WebAssembly initialization is explicit:

```ts
const engine = await createCorrplotEngine({ backend: "auto" });
const result = await engine.render(matrix, { format: "svg" });
```

The supported backend options are:

- `auto`: select the native Node binding when running in Node.js; select WebAssembly in a supported browser.
- `native`: require the Node-API binding. It is unavailable in browsers.
- `wasm`: require the browser WebAssembly binding. It is not part of the initial Node.js support contract.

Backend selection is an execution decision, not an output-format decision. The facade must use package conditional exports or separate environment entry points so browser bundlers do not load native binaries and Node-only deployments do not fetch browser WASM assets.

Selection must be deterministic. Do not silently fall back from a requested backend or switch implementations after a load failure. Return an actionable diagnostic that identifies the requested backend, runtime, and available alternatives. In `auto` mode, failure to load the preferred backend is an error; it must not silently change performance or feature characteristics.

The facade may accept an injected backend in tests and advanced integrations. Runtime detection must not depend on user-agent strings. Do not assume that a browser bundler provides Node globals such as `process`.

## Public TypeScript API boundary

The initial facade should expose a small, backend-neutral surface:

- `createCorrplotEngine({ backend })`: asynchronously initialize the selected execution backend.
- `engine.validate(input)`: validate normalized matrix input; invalid input raises `CorrplotError` with a diagnostic-v1 record.
- `engine.createScene(input, options)`: construct the renderer-neutral scene using Rust semantics.
- `engine.getRendererCapabilities(rendererId)`: return that renderer's declaration, validated against the existing capability-v1 contract.
- `engine.render(input, options)`: render a supported output format using the renderer contract defined in #153.
- `engine.dispose()`: release backend-owned resources where the runtime requires it; it is safe to call more than once.

The public TypeScript types must not expose internal Rust structs, pointers, N-API handles, or WebAssembly memory offsets. Use stable input/output types and diagnostic-v1 records. The facade may provide ergonomic JavaScript arrays and objects, but the shared core remains the only source of semantic decisions.

The API factory is asynchronous in both environments. This gives one predictable call pattern and makes WASM initialization explicit. Operations that are pure TypeScript type conversion need not become asynchronous on their own.

### Matrix input and memory ownership

- Accept a documented row-major matrix representation with explicit dimensions and labels.
- Support typed-array input for large numeric matrices. Missing cells must use an explicit mask or an equivalent lossless representation; do not overload NaN as missing because NaN and infinities are invalid observed values in the shared core.
- Validate dimension multiplication and allocation sizes before allocating buffers.
- In the first version, bindings may copy inputs into Rust-owned memory and return JavaScript-owned output buffers. Do not promise zero-copy behavior.
- Do not return long-lived views into WebAssembly linear memory. WebAssembly memory can grow and invalidate assumptions about backing buffers.
- Define ownership and lifetime for every returned byte or pixel buffer. Callers own returned JavaScript buffers; Rust must not retain borrowed JavaScript memory after the call.
- Optimize copies only after representative measurements, and only with an explicit ownership contract.

## Execution backend and renderer separation

Execution backend and output renderer are independent axes:

| Concern | Examples | Owner |
| --- | --- | --- |
| Execution backend | Node-API native, browser WebAssembly | TypeScript facade and binding adapters |
| Output renderer | SVG, PNG/RGBA, browser Canvas 2D | Renderer implementations and host adapters |
| Renderer capability | Supported scene versions, cell methods, annotations, metadata | Existing renderer capability-v1 contract (#71) |
| Interchange/conformance | normalized-matrix v1, scene-v1 | Versioned language-neutral contracts |

The native and WASM backends must call the same Rust semantic implementation. They must not maintain separate renderer capability schemas. The existing capability-v1 declaration remains authoritative; each renderer reports only the features it actually preserves. Capability negotiation remains strict by default, and unsupported required features fail before output is written.

SVG and PNG/RGBA output belong behind the renderer interface in #153. Direct Canvas rendering is a browser adapter tracked by #144. Canvas and DOM objects remain in TypeScript; Rust must not depend on browser object types. Canvas must consume the shared scene or a documented renderer command representation directly, not serialize to SVG and parse it again.

This ADR does not settle pixel-buffer layout, DPI rules, alpha/background semantics, or the complete renderer API. Those belong to #153 and its implementation work.

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

- Rust errors map to the same stable `code`, `severity`, `stage`, `message`, and optional location/details fields in both bindings.
- Node calls expose a typed `CorrplotError` whose `diagnostic` property contains the diagnostic-v1 record.
- Browser calls reject with the same typed error shape. The facade must not require callers to parse message text.
- Warnings remain structured diagnostics in successful results; they are not converted to errors.
- A backend initialization failure must be represented in the shared diagnostic-v1 schema, with runtime/backend context in `details`. Add a shared catalogue code if the existing catalogue has no suitable entry before implementing the bindings.
- Unexpected internal failures use a distinct shared diagnostic code; do not expose Rust panic payloads as a public API.

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

The browser policy deliberately targets evergreen browsers and WebAssembly MVP. Exact browser major versions are selected from the current stable and previous stable channels in CI and recorded in release evidence. Safari must receive a real browser smoke test before a release; WebKit automation alone is not proof of Safari packaging behavior. Legacy browsers and embedded webviews are outside the initial support contract.

## Testing and release gates

- Run the same semantic fixtures against the Rust core, native Node binding, WASM binding, and TypeScript facade.
- Verify equivalent validation codes, coordinates, scene data, and renderer capability decisions across native and WASM targets.
- Test the same public TypeScript examples in Node.js and a browser. Browser tests must load the actual packaged WASM asset, not a mocked module.
- Test the Node binding on every supported Node LTS line and each published operating-system/architecture binary.
- Test backend selection explicitly, including unavailable backends, missing assets, initialization failures, and dependency-injection paths.
- Test buffer dimensions, missing masks, memory limits, output ownership, and disposal.
- Test worker execution separately from the single-threaded baseline. Threaded WASM remains deferred.
- Keep performance and bundle-size measurements separate from routine correctness CI until stable baselines and budgets have been agreed.
- Preserve scene-v1 and diagnostic-v1 compatibility. A runtime binding must not silently change either contract.

## Relationship to existing issues

- **#51 — interactive exploration:** remains a separate P4 design issue. This ADR defines the rendering/core boundary, not selection, filtering, navigation, scatterplot links, or interactive state. Those features may build on the TypeScript facade after #51 defines their scope.
- **#71 — renderer capabilities:** complete. Reuse its schema and negotiation rules; do not create a second capability model for native versus WASM.
- **#115 — scene-v1 independence:** complete. Keep scene-v1 as the interchange and conformance format while consumers call the shared Rust implementation for common semantics.
- **#140 — portability:** implement the portable Rust core and native/WASM build proof next.
- **#153 — renderer output contract:** define output dimensions, scaling, pixel layout, and format semantics independently of runtime selection.
- **#141 and #142 — bindings:** implement thin Node-API and WASM adapters only after the portability boundary is verified; coordinate exposed rendering operations with #153.
- **#144 — Canvas:** keep Canvas/DOM integration in the browser adapter.
- **#145 — unified API:** implement the public package after the backend and renderer contracts are stable.

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
- The current core must grow beyond matrix validation before it can satisfy the complete rendering API. This ADR does not claim that those future operations already exist.
- Production R adoption remains separately gated by the CRAN and maintainer-workflow conditions in ADR 0001.
