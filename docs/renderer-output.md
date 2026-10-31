# Renderer output contract v1

The renderer API is separate from publication configuration and from the
scene-v1 interchange format. A publication config describes the requested
figure semantics. A scene-v1 document describes renderer-neutral content.
A render request selects the artifact format and its output characteristics.

The machine-readable request schema is
[renderer-output-v1.schema.json](../prototypes/renderer-output-v1.schema.json).
The corresponding dependency-free Rust types and trait are in
[crates/corrplot-core/src/render.rs](../crates/corrplot-core/src/render.rs).

## Interface

The Rust contract consists of:

- `SceneDocument`: an opaque, schema-validated scene-v1 JSON payload plus its
  schema version, matrix kind, logical dimensions, and cell methods. The
  renderer API does not require a JSON library.
- `RenderRequest`: output format, dimensions, background, pixel format,
  alpha mode, negotiation mode, and required/optional scene features.
- `Renderer<S>`: a generic renderer interface with a backend-specific
  associated output type. SVG can return a string, PNG encoded bytes, raw
  RGBA a pixel buffer, and a browser adapter may use its own target type.
- `RenderedArtifact<T>`: output format, requested dimensions, renderer
  identity/version, any explicitly omitted optional features, and payload.
- `RendererCapabilities`: the typed counterpart of the
  [capability declaration schema](renderer-capabilities.md).

A renderer must call `validate_render_request` before writing any output.
Strict negotiation rejects unsupported required or optional features.
Permissive negotiation may omit only features explicitly listed as optional;
each omission must be returned as a warning with diagnostic code
`renderer.optional_feature_omitted`. Unknown scene versions, unsupported
matrix kinds, output formats, and cell methods always fail.

## Artifact formats

| Format | Artifact | Intended use |
| --- | --- | --- |
| `svg` | UTF-8 SVG document | Vector output for publication, web, and LaTeX |
| `png` | Encoded PNG bytes | Portable raster artifact |
| `rgba` | Raw RGBA8 pixel buffer | In-memory raster consumers and image pipelines |

`scene-json` and `normalized-json` are interchange/serialization outputs,
not renderer formats. They are intentionally absent from renderer capability
declarations. A direct browser Canvas target is also not a file format: a
future Canvas adapter consumes the same scene/request semantics and draws to
a caller-supplied Canvas context without pretending that the context is a
serialized artifact.

## Dimensions and scaling

- `scene_units` uses the logical width and height carried by scene-v1. It is
  suitable for SVG; raster output must not guess a pixel scale.
- `physical` uses positive width and height with an explicit `mm` or `in`
  unit. Raster output additionally requires a positive DPI.
- `pixels` specifies exact positive integer output dimensions.
- For physical raster output, pixel dimensions are
  `round(inches × DPI)`; millimetres are converted to inches by dividing by
  25.4 before applying DPI. Implementations must document their rounding
  rule and reject overflow rather than wrapping dimensions.

The scene's logical coordinate system is not changed when output dimensions
or DPI change. Renderers scale the scene into the requested output viewport.

## Background, pixel format, and alpha

Colors are 8-bit sRGB RGBA. The background is either explicitly transparent
or an explicit RGBA color; a renderer must not silently choose a different
background. The initial raw raster pixel format is `rgba8`, with rows stored
top-to-bottom and pixels left-to-right.

- `opaque` guarantees every output pixel has alpha 255.
- `straight` stores unassociated RGB channels with alpha.
- `premultiplied` stores RGB channels multiplied by alpha and is valid only
  for raw RGBA buffers. PNG encoding uses straight alpha as required by PNG.
- A transparent background cannot be combined with opaque alpha.

SVG uses its native vector transparency model; pixel format and raster alpha
storage modes do not alter its geometry.

## Determinism and compatibility

For the same scene, request, renderer version, and supported runtime, an
implementation must produce deterministic output. SVG serialization must
have stable element/attribute ordering and numeric formatting. PNG encoding
and RGBA pixels must be deterministic for a given renderer version and
rasterization environment. This does not promise pixel identity between
different renderers, fonts, or operating systems.

Renderer identity/version and selected format/dimensions belong in
reproducibility metadata. Output-format selection must not be embedded in
publication-config v1, so the same configuration can be rendered by SVG,
PNG, RGBA, or a future Canvas target without changing figure semantics.

## Current implementation

The native publication-config SVG path is the first implementation of the
typed renderer interface. It preserves the existing physical dimensions and
white background, uses strict capability validation, and returns SVG text.
The current CLI capability declaration advertises only `svg`; scene-json and
normalized-json remain separate serialization commands.

PNG/RGBA renderers and the browser Canvas adapter are not implemented by this
contract PR. They can implement the same trait and output request without
changing publication configuration or scene-v1. The raster work remains
tracked separately in [#154](https://github.com/vsimko/corrplot-ng/issues/154).
