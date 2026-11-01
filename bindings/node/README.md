# Native Node.js binding (first slice)

This package is a thin Node-API adapter over the repository's shared
`crates/corrplot-core` Rust library. It does not implement matrix validation
or rendering semantics in JavaScript.

## Implemented operations

- `validateMatrix(input)` calls the shared Rust matrix validator.
- `renderSceneSvg(scene, options)` converts a typed scene object and invokes
  the shared Rust SVG renderer.
- `renderSceneRgba(scene, options)` invokes the shared Rust RGBA renderer and
  returns an owned Node Buffer with dimensions, stride, pixel format, alpha,
  color-space, and optional DPI metadata.
- `renderMatrixSvg(sceneV1Json, method)` parses the matrix-bearing portion of
  scene-v1 JSON, constructs the normalized matrix scene, and renders SVG
  entirely through the shared Rust core. Supported methods are circle,
  square, ellipse, number, shade, color, and pie.
- `getRendererCapabilities()` returns the core's capability-v1 declaration.

Errors are exposed as `CorrplotError` instances with a diagnostic-v1 record.
Matrix values and scene fields are converted into Rust-owned values. The RGBA
pixel vector is moved into the returned Node Buffer, avoiding a second pixel
buffer copy at the Rust/Node ownership boundary. The caller owns the returned
buffer and may retain or modify it independently of the Rust renderer.

## Build and test

Requirements: Node.js 22, 24, or 26, npm, and the Rust 1.78+ toolchain.

From this directory:

~~~
npm test
~~~

The test command builds the addon, runs the native tests, then creates an npm
tarball, installs it into a clean temporary project with lifecycle scripts
disabled, and imports it without invoking Cargo. This verifies that the
prepackaged native binary is present and loadable without a Rust toolchain at
runtime.

The build produces `index.node` for the current OS and architecture. The
resulting addon can be loaded by Node.js without a Rust toolchain at runtime.
When packaging a release, `prepack` builds the host-native addon and includes
it in the package archive. A source checkout therefore needs Rust to build;
a packaged release needs only a compatible Node.js runtime.

The initial CI platform matrix is Linux x64, macOS x64/arm64 runners, and
Windows x64, with Node.js 22, 24, and 26. Other architectures are not yet
claimed. Separate platform-specific prebuilt package publishing remains future
release work; this source-build package is not yet a published npm release.

## Scope boundary

The Rust core now also provides an initial scene-v1 matrix parser and deterministic matrix-to-scene builder. The Node and WASM bindings expose the same end-to-end matrix-to-SVG operation. The parser intentionally validates only the matrix-bearing subset; full scene-v1 presentation semantics are not yet implemented. Scene construction and the unified asynchronous TypeScript facade
remain follow-up work in #141 and #145.
