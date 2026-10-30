# Renderer-neutral scene contract, version 1

**Status:** Experimental prototype contract, not a stable public API.

The JSON Schema in [scene-v1.schema.json](scene-v1.schema.json) describes the prepared scene produced by the shared-SVG prototype. Language adapters may construct this data and independent renderers may consume it without sharing implementation code.

The two JSON examples have distinct roles:

- [example.json](example.json) is the input configuration for the Python scene builder.
- [scene-v1.example.json](scene-v1.example.json) is the canonical expected scene output. It is also the language-neutral payload consumed by the R smoke test.

The Python test suite compares the generated scene with this canonical output. This makes changes to geometry, cell order, labels, missing values, or style explicit and reviewable across language consumers.

## Semantics

- `schema_version` is `1`; incompatible structural changes require a new version.
- `kind` is `correlation_plot`.
- `width`, `height`, positions, and sizes use logical canvas units. They are not pixels, points, or CSS units.
- `labels` contains one item per matrix variable, in matrix order. `text` is always a string. Row and column label positions are independent.
- `cells` is a flat array in row-major order. `row` and `column` are zero-based indices into the original matrix/labels order.
- `value` is a correlation coefficient in [-1, 1], or JSON `null` for a missing value. Missing must not be silently converted to zero.
- Optional cell `color` is the mapped six-digit sRGB color for that value. It is absent in legacy scenes that use semantic style colors.
- Optional `legend` is an ordered array of renderer-neutral `kind`, `label`, and `color` entries. It describes the exact mapping used by cells and may include a distinct missing-value entry.
- `x`, `y`, and `size` describe the cell's top-left position and square extent. `center_x` and `center_y` are its center.
- `annotation` is plain text. This version supports one annotation string, not arbitrary positioned/layered elements.
- `style` contains six-digit sRGB fallback colors. It defines semantic colors only; it does not prescribe a particular shape or renderer.

## Deliberate limits

Version 1 is only enough for the current synthetic example. It does not yet define ordering algorithms, significance markers, confidence intervals, arbitrary annotations, typography, accessibility metadata, or native R graphics-device behavior. The optional legend is a semantic scene capability; individual renderers remain responsible for layout and clipping.

## Compatibility and evolution

Keep the legacy R API as the behavioral reference. Before using this contract beyond the prototype, validate it against representative legacy cases and decide how version negotiation and optional capabilities should work. Do not add renderer-specific fields to this scene merely to simplify one backend; propose a versioned semantic field instead.
