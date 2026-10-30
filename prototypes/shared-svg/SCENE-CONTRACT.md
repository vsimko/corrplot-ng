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
- Optional cell `color` is the mapped six-digit sRGB color for that value. Optional `p_value` carries the significance value permuted to the same row/column identity. It is absent in legacy scenes that use semantic style colors.
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

Version 1 remains an experimental subset. Ordering metadata and p-values are represented, but this contract does not define significance markers, confidence intervals, arbitrary positioned annotations, full label typography, accessibility metadata, or native R graphics-device behavior. The seven cell encodings are supported by the scene and SVG prototype; renderers must reject unknown methods rather than silently substituting another glyph. Coefficient/value text typography is limited to the explicit fields above. The optional legend is a semantic scene capability; individual renderers remain responsible for layout and clipping.

## Compatibility and evolution

Keep the legacy R API as the behavioral reference. Before using this contract beyond the prototype, validate it against representative legacy cases and decide how version negotiation and optional capabilities should work. Do not add renderer-specific fields to this scene merely to simplify one backend; propose a versioned semantic field instead.
