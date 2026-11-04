# Browser WebAssembly binding (first slice)

This package compiles the same `crates/corrplot-core` Rust library used by the
native CLI and Node-API adapter. The binding contains only JavaScript value
conversion; matrix validation and SVG/RGBA rendering remain in Rust.

## Implemented operations

- `validateMatrix(input)` calls the shared Rust matrix validator.
- `renderSceneSvg(scene, options)` converts a typed scene and calls the shared
  Rust SVG renderer. It returns an object containing `svg` or a diagnostic.
- `renderSceneRgba(scene, options)` calls the shared Rust raster renderer and
  returns an owned `Uint8Array` plus dimensions, stride, `rgba8` pixel format,
  straight alpha, sRGB color space, and optional DPI metadata.
- `renderMatrixSvg(sceneV1Json, method)` parses the matrix-bearing portion of
  scene-v1 JSON, constructs the normalized matrix scene, and renders SVG
  entirely through the shared Rust core. Supported methods are circle,
  square, ellipse, number, shade, color, and pie.
- `renderMatrixRgba(sceneV1Json, method, options)` uses the same Rust scene
  construction path and raster renderer, returning JavaScript-owned RGBA pixels
  with the same metadata and lifetime guarantees as `renderSceneRgba`.
- `getRendererCapabilities()` returns the core capability-v1 JSON string.

Results use diagnostic-v1 records. Input objects are deserialized into
Rust-owned values. RGBA output is copied once into JavaScript-owned typed-array
memory: a view into WebAssembly linear memory would not remain safe after the
Rust allocation is released or memory is grown. The returned `Uint8Array` can
therefore be retained independently of subsequent WASM calls.

## Build and test

Requirements: Rust 1.78+, the `wasm32-unknown-unknown` target, Node.js 24 or 26,
and wasm-pack.

~~~sh
wasm-pack test --node
npm run build
npm run test:pack
~~~

The npm package `@corrplot-ng/wasm` exports the generated browser ESM entry point from `pkg/`. The build removes wasm-pack's generated nested ignore file, and `npm run test:pack` verifies that both the ESM entry point and WASM binary are present in the npm tarball. The browser build is ESM and initializes asynchronously. Serve this directory
over HTTP (not `file://`) and open
[`examples/browser/index.html`](examples/browser/index.html) to run the
browser smoke example. The page checks matrix-to-SVG and matrix-to-RGBA output,
exercises all seven supported cell methods against the canonical missing-value
fixture and a construction fixture with per-cell overrides and figure text,
verifies positive, negative, and missing-cell colors in both the returned pixel buffer and a real browser
Canvas, checks malformed scene-v1 diagnostics, and confirms that a later WASM
call does not mutate the previously returned JavaScript-owned pixel buffer.

After building, run `node benchmarks/measure.mjs` from this directory to
report the raw/gzip/Brotli WASM module size, cold initialization time, available
WASM linear-memory size, process RSS delta, and median/p95 timings for matrix
validation, SVG rendering, and RGBA rendering. The script uses fixed fixtures
and iteration counts; results are informational baselines, not noisy
per-pull-request performance gates. A weekly scheduled run and manual
`workflow_dispatch` run collect the report on Node.js 26 and upload a JSON
artifact retained for 90 days. Node.js 24/26 correctness tests still run on
pull requests.

The separate Node/native-WASM comparison is run from the repository root after
building both backends:

~~~sh
(cd bindings/node && npm run build)
(cd bindings/wasm && wasm-pack build --target nodejs --release --out-dir pkg-node)
node bindings/cross-backend/benchmark-node-wasm.mjs
~~~

This benchmark first requires exact Node/WASM scene, SVG, RGBA metadata, and
pixel parity. It then records cold module-load samples, point-in-time RSS/heap/
external-memory deltas, validation and scene-construction timings for 2×2 and
64×64 matrices, SVG/RGBA timings, and native-addon/WASM artifact sizes. The
weekly/manual parity workflow uploads raw samples as a 90-day artifact; PRs
only smoke-test the harness and never enforce timing thresholds. Memory
deltas are snapshots, not peak-memory measurements.

## Browser threading and Canvas status

The current build is single-threaded WebAssembly. It does not enable shared
WebAssembly memory, atomics, or a Rust thread pool, and the binding does not
create or manage Web Workers. A host application may load its own module
instance inside a worker, but worker startup, message protocols, transferable
input/output buffers, cancellation, and worker-pool policy are not part of this
package and have not been validated here. Shared-memory threading is therefore
intentionally deferred; it would require an explicit browser compatibility
policy (including cross-origin isolation) and dedicated CI coverage.

The low-level browser example in `bindings/wasm/examples/browser` copies
returned RGBA pixels into a regular Canvas 2D context. That example demonstrates
host-owned presentation of the WASM output; it is not the direct Canvas API.
The browser TypeScript facade in `packages/typescript/` adds a separate direct
Canvas 2D adapter that draws the same renderer-neutral scene without an SVG or
RGBA intermediate. It supports caller-owned HTMLCanvasElement, OffscreenCanvas,
and Canvas 2D contexts, including device-pixel-ratio scaling and letterboxing;
see the [TypeScript facade README](../../packages/typescript/README.md).
OffscreenCanvas rendering is implemented by that facade where the runtime
provides a 2D context. Worker startup, message protocols, transferable matrix
buffers, cancellation, and worker-pool policy remain host responsibilities
and are not implemented or validated by this low-level WASM package.

The browser smoke suite runs against Chromium, Firefox, and WebKit on
Ubuntu 24.04 using Playwright. Node 24 and 26 both run the WASM tests and
package checks; the browser matrix runs on Node 24. This is a focused engine
compatibility check, not a claim of coverage across every browser version,
operating system, or mobile device.

## Scope boundary

The Rust core provides an initial scene-v1 matrix parser and deterministic
matrix-to-scene builder. The Node and WASM bindings expose the same normalized-
matrix scene construction and typed-scene SVG/RGBA operations, alongside the
matrix-bearing scene-v1 compatibility renderers. The runtime-isolated TypeScript
facade in `packages/typescript/` exposes this workflow through separate Node and
browser entry points, including typed-array normalization and explicit missing-
cell masks. Its browser entry point also provides the direct Canvas 2D adapter
implemented under #144; that adapter is a separate TypeScript layer rather than
part of the low-level WASM package API. The parser and builder intentionally
cover only the matrix-bearing subset; full scene-v1 presentation semantics and
broader cross-backend/browser conformance remain separate work in #146.
Package publication is explicitly deferred under #293.
