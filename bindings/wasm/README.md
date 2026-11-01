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
wasm-pack build --target web --release --out-dir pkg
~~~

The browser build is ESM and initializes asynchronously. Serve this directory
over HTTP (not `file://`) and open
[`examples/browser/index.html`](examples/browser/index.html) to run the
browser smoke example. The page checks the SVG result envelope, verifies
representative RGBA pixels and the Canvas copy, and confirms that a later WASM
call does not mutate the previously returned JavaScript-owned pixel buffer.

After building, run `node benchmarks/measure.mjs` from this directory to
report the raw/gzip/Brotli WASM module size, cold initialization time, available
WASM linear-memory size, process RSS delta, and median/p95 timings for matrix
validation, SVG rendering, and RGBA rendering. The script uses fixed fixtures
and iteration counts; results are informational baselines, not noisy
per-pull-request performance gates. CI records them on Node.js 24 and 26.

## Browser threading and Canvas status

The current build is single-threaded WebAssembly. It does not enable shared
WebAssembly memory, atomics, or a Rust thread pool, and the binding does not
create or manage Web Workers. A host application may load its own module
instance inside a worker, but worker startup, message protocols, transferable
input/output buffers, cancellation, and worker-pool policy are not part of this
package and have not been validated here. Shared-memory threading is therefore
intentionally deferred; it would require an explicit browser compatibility
policy (including cross-origin isolation) and dedicated CI coverage.

The browser smoke page copies the returned RGBA pixels into a regular Canvas
2D context. This is not a direct Canvas renderer: drawing remains host-owned,
and the Rust backend currently returns SVG or RGBA data. OffscreenCanvas and
worker-owned Canvas rendering are not implemented or claimed as supported.
Applications can transfer the owned RGBA result to their own Canvas adapter,
but device-pixel-ratio policy, resizing, and OffscreenCanvas lifecycle remain
the responsibility of that adapter.

The only automated real-browser smoke environment currently configured in CI
is the preinstalled Google Chrome on Ubuntu 24.04. Node 24 and 26 both run the
WASM tests and baseline measurements; the browser smoke currently runs on
Node 26. This is a tested CI configuration, not a claim of a comprehensive
browser compatibility matrix.

## Scope boundary

The current Rust core provides matrix validation and typed-scene SVG/RGBA
rendering, but not a complete scene-v1 parser or matrix-to-scene builder.
This first slice exposes existing operations without inventing missing
semantics. Full scene-v1 construction, automated browser conformance/visual
tests, performance budgets, package publishing, and the unified TypeScript
facade remain follow-up work in #142 and #145.
