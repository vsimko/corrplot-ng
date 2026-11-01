# Browser WebAssembly binding (first slice)

This package compiles the same `crates/corrplot-core` Rust library used by the
native CLI and Node-API adapter. The binding contains only JavaScript value
conversion; matrix validation and SVG rendering remain in Rust.

## Implemented operations

- `validateMatrix(input)` calls the shared Rust matrix validator.
- `renderSceneSvg(scene, options)` converts a typed scene and calls the shared
  Rust SVG renderer.
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
browser smoke example.

## Scope boundary

The current Rust core provides matrix validation and typed-scene SVG/RGBA
rendering, but not a complete scene-v1 parser or matrix-to-scene builder.
This first slice exposes existing operations without inventing missing
semantics. Scene construction, browser-side conformance/visual tests, performance budgets,
package publishing, and the unified TypeScript facade remain follow-up work in
#142 and #145.
