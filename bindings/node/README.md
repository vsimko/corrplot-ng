# Native Node.js binding (first slice)

This package is a thin Node-API adapter over the repository's shared
`crates/corrplot-core` Rust library. It does not implement matrix validation
or rendering semantics in JavaScript.

## Implemented operations

- `validateMatrix(input)` calls the shared Rust matrix validator.
- `renderSceneSvg(scene, options)` converts a typed scene object and invokes
  the shared Rust SVG renderer.
- `getRendererCapabilities()` returns the core's capability-v1 declaration.

Errors are exposed as `CorrplotError` instances with a diagnostic-v1 record.
Matrix values and scene fields are copied into Rust-owned values. This first
slice does not promise zero-copy buffers.

## Build and test

Requirements: Node.js 24 or 26, npm, and the Rust 1.78+ toolchain.

From this directory:

~~~sh
npm test
~~~

The build produces `index.node` for the current OS and architecture. The
resulting addon can be loaded by Node.js without a Rust toolchain at runtime.
When packaging a release, `prepack` builds the host-native addon and includes
it in the package archive. A source checkout therefore needs Rust to build;
a packaged release needs only a compatible Node.js runtime.

The initial CI platform matrix is Linux x64, macOS x64/arm64 runners, and
Windows x64, with Node.js 24 and 26. Other architectures are not yet claimed.

## Scope boundary

The Rust core currently provides matrix validation and typed-scene SVG/RGBA
rendering, but not a complete scene-v1 parser or matrix-to-scene builder.
This binding exposes the existing operations without inventing those missing
semantics. Scene creation and the unified asynchronous TypeScript facade
remain follow-up work in #141 and #145.
