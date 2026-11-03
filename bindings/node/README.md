# Native Node.js binding

This package is a thin Node-API adapter over the repository's shared
`crates/corrplot-core` Rust library. Matrix validation, scene construction,
diagnostics, and rendering semantics remain in Rust.

## Implemented operations

- `validateMatrix(input)` calls the shared Rust matrix validator.
- `renderSceneSvg(scene, options)` invokes the shared Rust SVG renderer.
- `renderSceneRgba(scene, options)` invokes the shared Rust RGBA renderer and
  returns an owned Node Buffer with dimensions, stride, pixel format, alpha,
  color-space, and optional DPI metadata.
- `renderMatrixSvg(sceneV1Json, method)` parses scene-v1 JSON, constructs the
  normalized matrix scene, and renders SVG through the shared Rust core.
- `renderMatrixRgba(sceneV1Json, method, options)` uses the same shared
  matrix-to-scene builder and raster renderer, returning an owned Node Buffer
  with the standard RGBA metadata.
- `getRendererCapabilities()` returns the core's capability-v1 declaration.

Errors are exposed as `CorrplotError` instances with a diagnostic-v1 record.
The RGBA pixel vector is moved into the returned Node Buffer, avoiding a
second pixel-buffer copy at the Rust/Node ownership boundary.

## Installation and supported binaries

The package selects a platform-specific optional dependency at runtime.
Prebuilt packages use the stable Node-API ABI and support:

| Platform package | OS / architecture | libc / ABI |
| --- | --- | --- |
| `@corrplot-ng/native-node-linux-x64-gnu` | Linux x64 | glibc |
| `@corrplot-ng/native-node-darwin-x64` | macOS x64 | Darwin |
| `@corrplot-ng/native-node-darwin-arm64` | macOS arm64 | Darwin |
| `@corrplot-ng/native-node-win32-x64-msvc` | Windows x64 | MSVC |

Linux musl, 32-bit systems, and other CPU architectures are not claimed.
A source checkout can build the addon locally with Rust 1.78+, Node.js
22/24/26, and npm using `npm run build`; consumers of a published binary
package do not need Cargo or a Rust toolchain.

The native binding CI builds the platform tarball on each matching runner,
installs that tarball together with the base package into a clean temporary
consumer project with lifecycle scripts disabled, and verifies validation,
renderer capabilities, and SVG rendering. This checks the actual prebuilt
package layout rather than only the source-built addon.

## Building release packages

The `.github/workflows/build-node-prebuilt-packages.yml` workflow builds
platform-specific npm tarballs on native GitHub-hosted runners and uploads
them as workflow artifacts. This is a build/package workflow only: it does
not publish to npm. Public publication is intentionally deferred to a later
maintainer-approved release (#293). When that work is prioritized, publish
the platform packages before `@corrplot-ng/native-node` so its optional
dependencies can resolve. The broader non-publishing package/release
preparation is tracked by #290 and documented in
[`docs/npm-package-distribution.md`](../../docs/npm-package-distribution.md).

## Scope boundary

This is the native Node backend, not the unified cross-runtime TypeScript
facade. Backend selection and shared browser/native API design remain in
issue #145. The WASM and direct Canvas work remain in issues #142 and #144.
