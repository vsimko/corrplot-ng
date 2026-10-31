# Renderer output contract, version 1

**Status:** Current contract; raster and browser Canvas implementations are tracked separately.

This document defines output selection independently of figure semantics, the Rust API types in crates/corrplot-core/src/output.rs, and the host boundary required by the Node and WebAssembly adapters.

## Separation of concerns

- **Scene:** describes plot meaning, geometry, annotations, and accessibility metadata. It does not select a file format.
- **Output options:** select a serialized output format and its dimensions, typography, background, and density metadata. In Rust these are OutputOptions.
- **Renderer:** consumes an immutable prepared scene and validated output options. The generic Renderer<Input> trait allows a typed shared Scene to be introduced without changing the output types.
- **Execution backend:** Node-API or WebAssembly. It does not change output semantics.
- **Host target:** browser Canvas 2D is owned by the TypeScript/browser adapter. It is not a serialized file or a raw pixel buffer.

The existing publication-config-v1 includes an SVG output path and a fixed SVG format for backward compatibility. Its existing field is an adapter convenience, not part of scene semantics. Do not add PNG/RGBA-specific fields to publication-config-v1. New consumers pass OutputOptions to the renderer call. The CLI's --format option remains separate from the publication figure configuration.

## Serialized output formats

The Rust OutputFormat enum currently defines:

| Format | Rust result | Contract |
| --- | --- | --- |
| svg | RenderOutput::Svg(String) | UTF-8 SVG document. |
| png | RenderOutput::Png(Vec<u8>) | Encoded PNG bytes with the PNG signature. |
| rgba | RenderOutput::Rgba(RgbaBuffer) | Owned raw RGBA8 pixel buffer. |

OutputOptions is a format-tagged enum. A renderer must validate the options before allocating output and return the matching RenderOutput variant. Call RenderOutput::validate_for(options) at the adapter boundary to check the format and output invariants.

## Dimensions and scaling

### SVG

- Width and height are positive finite numbers.
- Units are CSS pixels (px), millimetres (mm), or inches (in).
- The SVG viewBox remains in scene logical units; the root width and height define the requested viewport.
- Preserve the scene aspect ratio and center the scene in the viewport. Do not stretch or crop the plot.
- Typography is explicit through font family and point size. Font availability and glyph metrics remain viewer-dependent; renderers must not claim that fonts are embedded unless they are.

The current publication SVG path retains its existing physical mm/in sizing, viewBox, typography, and deterministic element ordering. It now adapts through the shared Renderer/OutputOptions types. The transitional adapter still accepts the publication input and builds its scene-v1 JSON internally because the Rust core does not yet expose a typed Scene. Existing output is regression-tested against the previous convenience method. The direct matrix CLI SVG path remains supported with its current dimensions and glyph behavior.

### PNG and raw RGBA

- Width and height are exact positive output pixel dimensions.
- dpi is optional density metadata. It does not silently change the requested pixel dimensions.
- Scale uniformly to fit the scene in the output viewport and center it. Do not crop or distort. Fill unused viewport area with the requested background.
- Raster dimensions and buffer-size multiplication are checked before allocation. The initial RGBA renderer enforces a 128 MiB per-render pixel-buffer limit.
- PNG output, when implemented, is an encoded PNG byte vector. When DPI is supplied, the PNG encoder should write equivalent physical-density metadata. The current renderer provides raw RGBA only.
- Raw RGBA output owns its byte vector and exposes the row stride. A consumer must not retain a borrowed pointer into Rust or WebAssembly memory.

**Raw RGBA rendering is implemented as the initial bitmap backend in #154.** The typed `corrplot_core::raster::Scene` is consumed by both `SceneSvgRenderer` and `RgbaRenderer`; the RGBA path rasterizes directly to pixels and does not serialize/reparse SVG. PNG byte encoding remains a follow-up. The current bitmap text path uses a built-in 5x7 ASCII font with a replacement glyph for unsupported characters; it does not claim full font-family or plotmath shaping.

## Pixel and alpha contract

Raw RGBA v1 uses:

- top-left origin;
- row-major rows;
- 8 bits per channel in red, green, blue, alpha order;
- sRGB color space;
- straight (unassociated) alpha;
- explicit stride_bytes, which must be at least width_px * 4;
- an owned byte vector whose length is exactly stride_bytes * height_px.

The PNG renderer must preserve the same pixel meaning and straight-alpha interpretation. The background is an explicit sRGB RGBA color. White is the default for compatibility; alpha zero requests a transparent background. A renderer must not silently premultiply alpha or change color space.

## Determinism

- For the same scene, options, and renderer version, SVG serialization is deterministic: stable element/attribute order and numeric formatting.
- Raw RGBA bytes are deterministic for the same scene, options, and renderer version.
- PNG output must be pixel-equivalent for the same inputs. Compressed byte equality is not guaranteed across encoder versions unless the encoder and compression settings are pinned.
- Renderer identity and version remain in the renderer-capabilities declaration and should be included in reproducibility metadata where available.
- Visual comparisons between SVG and raster output use documented tolerances; they do not require byte equality across different renderers.

## Direct browser Canvas target

Canvas 2D is a browser host target, not a serialized OutputFormat. The TypeScript/browser adapter owns the caller-provided HTMLCanvasElement, CanvasRenderingContext2D, or OffscreenCanvas. Rust must not receive or retain DOM objects.

The browser API is expected to expose a method equivalent to:

    await engine.renderToCanvas(scene, canvasOrContext, {
      width: 1200,
      height: 900,
      devicePixelRatio: window.devicePixelRatio,
      background: "#ffffff"
    });

The adapter sets the canvas backing-store dimensions, applies device-pixel-ratio scaling, clears/fills the background, and draws the shared scene or a documented renderer command representation. It must not serialize SVG and parse it to draw. The canvas context remains owned by the caller.

OffscreenCanvas is optional and feature-detected. The baseline Canvas path must work on the main thread without cross-origin isolation headers, SharedArrayBuffer, WebAssembly threads, WebGPU, or WebGL. Worker transport and OffscreenCanvas behavior are implemented in #144.

Renderer capability declarations use the existing capability-v1 schema. The optional host_targets property distinguishes browser-canvas-2d and browser-offscreen-canvas-2d from serialized output_formats. Canvas renderers still declare the same scene versions and feature flags and use the same strict/permissive negotiation and diagnostic rules. No second capability model is introduced.

## Diagnostics

The shared diagnostic-v1 catalogue includes:

| Code | Stage | Meaning |
| --- | --- | --- |
| renderer.output.invalid_options | renderer | Invalid output dimensions, units, or format-specific options. |
| renderer.output.invalid_buffer | export | Output bytes or pixel-buffer layout violate the output contract. |
| renderer.output.format_mismatch | export | The renderer returned a different format from the requested format. |

Runtime adapters map OutputValidationError to diagnostic-v1 records. Callers branch on the stable code and stage, not the message text. Capability negotiation continues to use the existing renderer.unsupported_feature, renderer.optional_feature_omitted, and renderer.schema_version_unsupported diagnostics.

## Implementation boundaries

- #153 defines this output contract and the Rust output types/trait.
- #154 implements direct raw RGBA rendering from the typed shared scene; PNG encoding remains future work.
- #144 implements direct browser Canvas rendering and the TypeScript host adapter.
- #140 ensures the shared core and output types compile for native and WASM targets.
- #141/#142 expose the same core through Node-API and WebAssembly.
- #145 publishes the unified TypeScript API.

Do not add renderer-specific output fields to scene-v1 or publication-config-v1. Do not put Canvas or DOM types in the Rust core.
