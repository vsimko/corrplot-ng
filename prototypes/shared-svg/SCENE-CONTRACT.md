# Renderer-neutral scene contract, version 1

**Status:** Experimental prototype contract, not a stable public API.

The JSON Schema in [scene-v1.schema.json](scene-v1.schema.json) describes the prepared scene produced by the shared-SVG prototype. Language adapters may construct this data and independent renderers may consume it without sharing implementation code.

The two JSON examples have distinct roles:

- [example.json](example.json) is the input configuration for the Python scene builder.
- [scene-v1.example.json](scene-v1.example.json) is the canonical expected scene output. It is also the language-neutral payload consumed by the R smoke test.

The Python test suite compares the generated scene with this canonical output. This makes changes to geometry, cell order, labels, missing values, or style explicit and reviewable across language consumers.

## Semantics

- `schema_version` is `1`; incompatible structural changes require a new version.
- `kind` is `correlation_plot` or `general_matrix_plot`. Correlation scenes retain the [-1, 1] cell-value constraint; general-matrix scenes preserve arbitrary finite numeric values.
- `width`, `height`, positions, and sizes use logical canvas units. They are not pixels, points, or CSS units.
- `labels` contains one item per matrix variable, in prepared matrix order. Optional `variable_order` records the ordering strategy, the ordered-to-original zero-based index permutation, and original labels. `text` is always the original plain string, preserving variable identity. Optional `row_visible` and `column_visible` independently control the two label sides; `row_position` and `column_position` identify left/top/diagonal placement. `column_rotation` is the rotation in degrees, and `offset` is the legacy label offset multiplier. Optional `plotmath` preserves a prefixed R expression as a safe `prefix` plus unevaluated `source` string. It is metadata, not HTML or SVG markup.
- `cells` is a flat array in row-major order. `row` and `column` are zero-based indices into the prepared matrix/labels order. When `variable_order` is present, map a prepared index back to the source matrix with `variable_order.indices`. Triangle filtering never removes or renumbers prepared cells.
- When display options are supplied, each cell carries `visible`, `region` (`upper`, `lower`, or `diagonal`), and its resolved `method`. `display_type` selects `full`, `upper`, `lower`, or `diagonal`; `show_diagonal` independently controls diagonal visibility. Upper means `row < column`, lower means `row > column`, and the diagonal is selected separately. Upper/lower methods default to the global `method`.
- Renderers must skip cells with `visible: false` while retaining their identity and geometry in the scene. This keeps cell metadata aligned for other consumers and avoids treating a display mask as data removal.
- `value` is the original numeric matrix value, or JSON `null` for a missing value. Correlation scenes constrain observed values to [-1, 1]; general-matrix scenes do not. Missing must not be silently converted to zero. Optional `normalized_value` is a visual-only transformed magnitude derived from the resolved matrix normalization; renderers use it for glyph sizing without replacing the original value.
- Optional cell `color` is the mapped six-digit sRGB color for that value. Optional `p_value` carries the significance value permuted to the same row/column identity; null means unavailable, not nonsignificant. Optional `significance` carries the prepared status, annotation, and blanking decision. Optional `confidence_interval` carries original lower/upper bounds and prepared whisker geometry.
- Optional cell `encoding` is prepared by the scene layer and contains the selected method, normalized magnitude, sign, semantic color, and method-specific geometry. The SVG backend serializes this geometry without re-deriving value-to-shape mappings.
- Optional `legend` is an ordered array of renderer-neutral `kind`, `label`, and `color` entries. It describes the exact mapping used by cells and may include a distinct missing-value entry.
- `x`, `y`, and `size` describe the cell's top-left position and square extent. `center_x` and `center_y` are its center.
- `annotation` is plain text. This version supports one annotation string, not arbitrary positioned/layered elements.
- Optional `glyph_method` identifies the default cell encoding: `circle`, `square`, `ellipse`, `number`, `shade`, `color`, or `pie`. A per-cell `method` overrides it. Circle radius and square side scale linearly with absolute magnitude; ellipse eccentricity and diagonal orientation encode magnitude and sign; number prints the formatted value; shade uses magnitude-dependent tint plus a sign-oriented diagonal; color fills the cell with a neutral-to-signed color interpolation; pie uses a signed sector whose sweep angle is proportional to absolute magnitude. A mapped cell `color` takes precedence over fallback colors.
- Optional `text_annotations` is a row-major list of positioned cell labels. Each entry carries zero-based `row` and `column` identity, a `role` (`coefficient` or `value`), preformatted text, color, font size, weight, and style. Renderers must place each label at the matching cell center; labels are not re-associated by array position. The row and column indices refer to the prepared scene after any variable permutation; `variable_order` retains the corresponding original indices and labels. Mixed upper/lower/diagonal methods determine the annotation role independently for each visible cell.
- Coefficient labels are emitted only when a coefficient color is supplied. Missing cells are never formatted as zero. Visibility is applied before labels are created; a per-cell `number` method takes precedence over coefficient labels.
- The number method takes precedence over the coefficient overlay, matching corrplot 0.95: it emits value labels using the number typography and does not emit a second coefficient label.
- `number_digits` defaults to 2 for decimal output and 0 for percentage output. Percent formatting multiplies the original coefficient by 100 before rounding. `number_cex` scales the default logical text size; `number_font` maps R font values 1–4 to plain, bold, italic, and bold-italic.
- Optional `matrix_semantics` records the explicit semantic mode, resolved two-value color domain, sequential/diverging mapping family, selected palette name, and positive zoom/intercept normalization. General numeric matrices use the legacy `YlOrBr` sequential palette for one-sided ranges and `RdBu` diverging palette for mixed-sign ranges. `general_matrix_plot` requires `mode: general_numeric`. Normalization affects visual mapping only; it never replaces the original cell values. Optional cell `normalized_value` carries the transformed magnitude used for glyph sizing, while `value` remains the original input.
- `style` contains six-digit sRGB fallback colors. It defines semantic colors only; it does not prescribe a particular shape or renderer.

The SVG prototype does not evaluate or typeset R plotmath expressions. It raises an explicit error when a scene contains `plotmath` metadata rather than silently rendering the source as ordinary text. R adapters can recover a native expression with `corrplot_ng_parse_label()`; other renderers must implement an explicit plotmath capability or reject the feature.

## Deliberate limits

Version 1 remains an experimental subset. Ordering metadata, significance modes/markers, and correlation confidence intervals are represented. Arbitrary positioned annotations, full label typography, accessibility metadata, and native R graphics-device behavior remain outside this contract. The seven cell encodings are supported by the scene and SVG prototype; renderers must reject unknown methods rather than silently substituting another glyph. Coefficient/value text typography is limited to the explicit fields above. The optional legend is a semantic scene capability; individual renderers remain responsible for layout and clipping.

## Current consumers and future bindings

The current prototype demonstrates that scene-v1 is a data boundary rather
than an R object model:

| Consumer | Role | Evidence |
| --- | --- | --- |
| Python shared-SVG prototype | Produces canonical scene-v1 JSON and validates it against the JSON Schema | `prototypes/shared-svg/scene.py`, `test_scene_schema.py` |
| Rust native CLI | Non-R consumer; imports the canonical fixture's matrix identity and values, then validates and re-emits them in its own output format | `tools/corrplot-ng/tests/cli.rs`, `tools/corrplot-ng/src/publication.rs` |
| R scene smoke test | Reads the canonical JSON fixture through the R JSON consumer path | `prototypes/shared-svg/test_r_scene_consumer.R` |

The Rust CLI importer is deliberately a matrix consumer, not a second scene
renderer: it reads `kind`, `labels`, and `cells`, preserves missing values,
and ignores canvas geometry and presentation metadata. Its tests exercise the
same checked-in `scene-v1.example.json` fixture used by the Python schema
tests. This proves that a non-R consumer can parse and validate the shared
contract without depending on R bindings or implementation-specific objects.

### Version and compatibility rules

- A consumer must require `schema_version` and reject versions it does not
  support. The current Rust importer accepts exactly version 1 and tests that
  version 2 is rejected.
- Consumers must reject unknown `kind` values. Correlation and general-matrix
  semantics are distinct; a consumer must not infer one from the value range.
- Cell identity is zero-based and row-major. Consumers must use each cell's
  explicit `row` and `column` indices rather than relying on incidental JSON
  key order or re-associating cells by position.
- JSON `null` means missing. It must remain missing through decoding,
  validation, and re-encoding; it must never become zero or a non-finite number.
- The v1 schema is closed to undeclared properties. Additions must be made to
  the schema and shared fixture together. Structural or semantic changes that
  cannot be consumed compatibly require a new `schema_version`.
- Optional fields may be ignored only when the consuming operation does not
  require their semantics. A renderer that does not support a requested glyph
  method or plotmath capability must return an explicit unsupported-capability
  diagnostic rather than silently substituting a different representation.
- `plotmath` is plain metadata (`prefix` plus unevaluated `source` string),
  not an R expression object, HTML, or executable markup. Non-R consumers must
  not evaluate it as code.

### Requirements for future Python and TypeScript consumers

No packaged Python or TypeScript consumer is defined by this prototype yet.
Future bindings should follow the schema and fixture rather than create a
language-specific scene model:

- Python should decode JSON `null` as `None`, retain the explicit row/column
  indices, validate against the published schema, and expose unsupported
  optional capabilities as explicit errors. The current Python module is a
  reference producer, not a stable installed API.
- TypeScript should model cell values as `number | null`, constrain `kind` to
  the two declared variants, use zero-based numeric indices, and validate
  incoming documents against the same JSON Schema. DOM, SVG, React, or R-specific
  types belong in adapters, not in the shared contract.
- Both should consume the shared canonical fixture in tests and include
  compatibility tests for unsupported schema versions, unknown kinds,
  missing cells, and optional capabilities they do not implement.
- Neither binding should duplicate matrix preparation, ordering, or value
  normalization rules. Those semantics belong to the producer of the prepared
  scene; consumers interpret the resulting versioned data.

## Compatibility and evolution

Keep the legacy R API as the behavioral reference. Before using this contract beyond the prototype, validate it against representative legacy cases and decide how version negotiation and optional capabilities should work. Do not add renderer-specific fields to this scene merely to simplify one backend; propose a versioned semantic field instead.
