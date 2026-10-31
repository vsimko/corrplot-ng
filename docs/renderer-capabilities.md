# Renderer capability negotiation v1

Renderer capabilities are declared separately from publication configuration.
A figure describes requested semantics; a renderer declares what it can preserve.
A renderer must not silently discard required scene content.

## Capability declaration

The machine-readable format is defined by
\`prototypes/renderer-capabilities-v1.schema.json\`. Each declaration records:

- renderer identity and version;
- supported scene schema versions and output formats;
- supported matrix kinds and cell methods;
- support for labels, plotmath, missing cells, cell annotations,
  significance, confidence intervals, legends, figure-level text, and
  accessible metadata;
- supported negotiation modes.

Current declarations:

- [Python SVG prototype](../prototypes/shared-svg/renderer-capabilities.json)
- [Rust standalone CLI SVG output](../tools/corrplot-ng/renderer-capabilities.json)

The Rust CLI is a matrix-to-SVG application, not a general scene-v1 renderer.
Its scene-v1 input path extracts labels and cell values and then creates a
new output scene. Its declaration therefore lists no scene renderer version
and does not claim to preserve incoming scene layout or annotation metadata.
Run \`corrplot-ng capabilities\` to print its declaration.

## Negotiation modes

- **Strict:** reject any requested feature that the renderer does not
  support. The check occurs before SVG serialization, so unsupported
  semantics cannot produce a partial file.
- **Permissive:** still reject unsupported features by default. A caller may
  explicitly mark named features as optional. Each omitted feature produces
  a structured warning with code \`renderer.optional_feature_omitted\`,
  renderer identity, and feature name.

The Python reference API exposes
\`negotiate_scene_capabilities()\` and
\`render_scene_svg_with_diagnostics()\`. The existing
\`render_scene_svg()\` convenience API uses strict mode. A caller must opt in
to permissive behavior and list each feature that may be omitted. The
renderer returns one diagnostic per omission; it does not mutate the input
scene.

Unknown scene schema versions always fail. They cannot be treated as optional
because the renderer cannot safely interpret an unknown contract.

## Current capability limits

The Python SVG prototype supports all seven cell methods, missing cells,
cell text annotations, significance annotations, confidence intervals,
figure-level title/subtitle/caption/footnote text, and accessible title and
description metadata. It does not interpret R plotmath labels or draw a
legend. Those features fail in strict mode; permissive mode can omit them
only when the caller explicitly marks them optional.

The native Rust CLI supports circle and number cell methods for its matrix
input and publication-config workflow. Its declaration is intentionally
narrow. Do not infer feature parity between the Rust CLI and Python SVG
prototype from their shared use of the word SVG.

## Adding a renderer

1. Add a declaration that validates against the capability schema.
2. List only features the renderer actually preserves.
3. Add conformance fixtures for every declared feature and unsupported case.
4. Run strict negotiation before writing output.
5. If permissive mode is implemented, require an explicit optional-feature
   allow-list and return a structured warning for every omission.
6. Record the renderer identity and version in diagnostics and reproducible
   output metadata where available.
7. Keep renderer capabilities out of the public figure configuration and do
   not redefine correlation or scene semantics in a backend.
