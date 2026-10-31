# corrplot-core

This internal Rust library is the first shared compiled operation used by
both the native CLI and the rextendr R-binding spike.

## API boundary

\`validate_matrix(labels, rows, columns, values, kind)\` accepts row-major
\`Option<f64>\` values. \`None\` represents an explicit missing cell; NaN and
infinity remain observed values and are rejected. It validates non-empty
square dimensions, unique labels, finite observed values, and (for
correlation matrices) range, symmetry, and unit diagonal.

Errors expose stable diagnostic codes and optional zero-based row/column
coordinates. The crate does not depend on R, extendr, CLI parsing, JSON, or
any third-party Cargo package. Adapters own parsing and serialization and
translate R NA to None at the boundary.

This is an internal integration proof, not yet the complete production
visualization engine. Keep renderer and language-specific behavior outside
this crate.


## Portable targets

The crate is compiled from the same source for native targets and
`wasm32-unknown-unknown`. The portable core forbids unsafe Rust and does not
perform filesystem, process, environment, dynamic-library, browser-DOM, or
graphics-device operations. Runtime-specific I/O and bindings belong in
consumer adapters.

Check the WebAssembly target (including unit-test code) with the declared
minimum Rust version or stable toolchain:

```sh
rustup target add wasm32-unknown-unknown
cargo check --locked --offline --all-targets --target wasm32-unknown-unknown
cargo test --locked --offline
```

The WebAssembly build check proves target portability of this Rust library; it
does not create JavaScript exports or a browser package. The thin WASM binding
and its loading/memory boundary are tracked separately in issue #142. Native
unit tests execute the shared semantic test suite; the WebAssembly CI job also
compiles those same tests for the target, without claiming a browser runtime
test before the binding/runtime harness exists.


## Direct scene renderers

The typed `raster::Scene` is a renderer-neutral set of logical dimensions,
cell rectangles/glyphs, and positioned text. Both `raster::SceneSvgRenderer`
and `raster::RgbaRenderer` consume this same scene; the RGBA path draws pixels
directly and never serializes or reparses SVG.

The dependency-free RGBA renderer supports circle, square, ellipse, pie,
number, shade, and color cell primitives, visible/hidden cells, strokes,
viewport clipping, text annotations, straight alpha compositing, and
letterboxed scaling. It emits an owned top-left-origin sRGB RGBA8 buffer with
the stride and optional DPI metadata defined by renderer-output-v1.

The current bitmap text fallback is a built-in 5x7 ASCII font; unsupported
characters use a visible replacement glyph. It is intentionally not a
font-engine replacement and does not claim plotmath shaping or font-family
fidelity. Each render is limited to 128 MiB of pixel storage. PNG encoding is
not implemented yet; raw RGBA is the initial bitmap output. The same Rust
source compiles for native and `wasm32-unknown-unknown`; JavaScript exports
and browser loading remain in #142.
