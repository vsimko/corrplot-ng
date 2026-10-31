# Legacy R adapter design

**Status:** Initial implementation slice; the adapter remains experimental.

This document defines how the legacy R API is introduced to the shared scene model without silently changing established corrplot behavior. The pinned upstream API inventory in [legacy-api-compatibility.md](legacy-api-compatibility.md) remains the source of truth for names, formals, defaults, and accepted values.

## Routing contract

corrplot_ng() exposes an explicit engine selector:

- engine = "legacy" is the default and delegates to corrplot::corrplot(). Existing calls therefore retain the established plotting behavior and graphics-device side effects.
- `corrplot_ng_mixed()` is the explicit legacy wrapper for `corrplot::corrplot.mixed()`; it preserves the upstream combined return object and remains legacy-only.
- engine = "scene" validates the currently supported subset and returns a corrplot_ng_scene object conforming to scene-v1. It prepares data; it does not draw a plot or select a glyph renderer.
- An explicitly requested scene operation never falls back to the legacy engine. Unsupported options produce an actionable error naming the unsupported arguments.

The scene selector is intentionally explicit. It avoids guessing whether a call can be translated and makes the returned value's type predictable.

## Legacy return values

The legacy adapter preserves upstream return objects and invisibility. See
[the R return-value contract](legacy-r-return-values.md) for the exact
`corr`, `corrPos`, and `arg` structures, coordinate mapping, upper/lower
filtering, and mixed diagonal quirks. The mixed wrapper is exported as
`corrplot_ng_mixed()`; it remains legacy-only until a mixed scene contract is
implemented.

## Graphics device ownership and lifecycle

The legacy route forwards plotting arguments, including `add`, directly to
`corrplot::corrplot()`. The active device and any existing plot window are
owned by the caller. `add = TRUE` composes into the current plot coordinates;
the adapter does not open, select, or close devices. `add = FALSE` starts a
new plot on the current device, not a new device.

The upstream corrplot implementation temporarily sets `par(mar, bg)` and
restores those values with `on.exit()`, including when drawing raises an
error. The adapter deliberately does not restore every `par()` value:
restoring `usr`/plot-window state would break the existing coordinates needed
for `add = TRUE`. Device closure remains the caller's responsibility.

The R package regression suite covers successful plotting, `add = TRUE`
composition, restoration after a rendering error, unchanged active-device
identity, and the scene engine's no-device-side-effect guarantee. See
`implementations/r/tests/testthat/test-graphics-device.R`.

## Cluster rectangles\n\nThe exported `corrRect()` adapter accepts one of three legacy forms: increasing one-based index boundaries, ordered variable-name boundaries, or `namesMat` cell-coordinate pairs resolved through `corrPos`. Index/name ranges are converted into the shared scene-v1 cluster-region request shape (`id`, `variables` or zero-based `indices`, `color`, `line_width`) before drawing. Diagonal square `namesMat` rectangles can be converted to the same request; arbitrary cell-coordinate rectangles remain R-only drawing annotations because the shared cluster-region contract describes diagonal variable groups.\n\n`corrRect.hclust()` clusters `1 - corr` with the selected legacy `hclust` method, cuts the tree, orders groups by dendrogram leaves, and creates one region request per contiguous group. The main `corrplot_ng()` pass-through also retains upstream `addrect`, `rect.col`, and `rect.lwd` arguments unchanged. Both helpers draw on the caller-owned active device and do not open or close it. Index/name inputs require identical row and column names; `namesMat` supports distinct row/column labels. See the [scene-v1 schema](../prototypes/shared-svg/scene-v1.schema.json) and the R help pages for [`corrRect()`](../implementations/r/man/corrRect.Rd) and [`corrRect.hclust()`](../implementations/r/man/corrRect.hclust.Rd).\n\n## Standalone color legend helper

The R package exports `colorlegend()` as an independently callable base-graphics helper, matching the formal arguments and defaults from corrplot 0.95. It draws on the active graphics device and supports vertical and horizontal bars, normalized tick positions, automatic or explicit tick spans, optional labels, and vertical label alignment. Additional graphical arguments are forwarded to `graphics::text()`.

This helper owns R device drawing only. It does not create or redefine the renderer-neutral scale and legend model specified by issue #64, and it does not add SVG serialization.

## Initial argument mapping

| Legacy concept / input | Scene-v1 mapping | Initial status |
|---|---|---|
| corr | Numeric square matrix; finite values; correlation mode additionally requires [-1, 1] | Supported |
| rownames(corr) / colnames(corr) | One ordered labels vector; both sets must match when both are present | Supported |
| Missing matrix values | Cell value: null; never coerced to zero; visible label/color map to a missing-role text annotation | Supported in scene mode for the default question mark and configurable label/color |
| annotation (corrplot-ng adapter option) | Scene-level plain-text annotation | Supported |
| method | No glyph/method field in scene-v1 | Rejected in scene mode |
| type, diag | Per-cell `visible` and `region` metadata; hidden cells retain identity and geometry | Supported |
| order, hclust.method | No permutation/order semantics in scene-v1 | Rejected in scene mode |
| is.corr | Explicit semantic mode; defaults to correlation and is never inferred from values | Supported |
| col.lim | Resolved color domain; correlation limits remain within [-1, 1], general-matrix limits must cover all observed values | Supported (per-cell colors and visual normalization are included in the scene) |
| col | Explicit palette assignment | Rejected in scene mode |
| Labels (tl.*) | Text styling, plotmath parsing, placement and rotation are not represented | Rejected in scene mode |
| Coefficients and number typography | `addCoef.col`, `addCoefasPercent`, `number.digits`, `number.cex`, and `number.font` map to renderer-neutral cell text annotations; `method = "number"` takes precedence over `addCoef.col` | Supported for circle coefficient overlays and number text; significance/CI remain rejected |
| add, mar, device state and helper drawing | R graphics-device behavior remains legacy-only | Legacy-only |

General-matrix mode is selected explicitly with `is.corr = FALSE`. It preserves original cell values and records `matrix_semantics` with the selected mode, resolved color limits, palette family/name, and normalization parameters. One-sided matrices use the legacy `YlOrBr` sequential palette; mixed-sign matrices use `RdBu`. Each observed cell carries its mapped color and visual-only normalized value, leaving `value` unchanged. Automatic general-matrix limits use observed values and exclude the diagonal when `diag = FALSE`; a constant-valued matrix receives finite padding of max(0.5, 5% of the absolute constant). An all-missing general matrix remains a valid missing-only scene. It carries has_observed_values = false and a neutral fallback color domain that must not be presented as observed numeric data. Explicit `col.lim` must be finite, strictly increasing, and cover every observed value. Correlation mode retains its [-1, 1] validation and defaults to [-1, 1].

The scene mode accepts the adapter-specific annotation option, coefficient/value text options, explicit matrix semantic mode and color limits, and the `method`, `type`, and `diag` subset described above. The adapter returns renderer-neutral mapped cell metadata but does not itself invoke an SVG or graphics-device renderer. Other arguments are rejected rather than ignored.

## Migration sequence

1. Establish explicit routing and preserve the default legacy path.
2. Implement and test a narrow scene-v1 input slice (matrix, labels, missing values, annotation).
3. Extend the versioned scene contract before routing additional legacy semantics. Coefficient/value text is the first annotation slice; ordering, the remaining glyph methods, significance, and confidence intervals remain separate work.
4. Add a renderer integration before describing scene mode as a plotting backend.
5. Expand compatibility fixtures and compare semantic results against the pinned upstream implementation for each newly routed capability.
6. Keep R graphics-device operations and unsupported options on the legacy path until an equivalent behavior is specified and tested.

## Compatibility guarantees and non-goals

- The default engine remains the legacy implementation.
- Scene mode fails closed when it receives unsupported arguments; it does not silently drop options or silently switch engines.
- This slice does not claim visual parity, full API compatibility, or a production renderer.
- No legacy function is removed or redefined, and this work does not decide the production implementation language.


## Missing-cell presentation

Scene mode supports the legacy-compatible `na.label` and `na.label.col`
options. The default label is `?` and the default color is `black`; setting
`na.label = ""` suppresses the text while retaining the missing-cell fill.
Missing text is represented by a renderer-neutral text annotation with role
`missing`, keyed to the same row/column coordinates as its null-valued cell.
It is independent of circle/number glyphs and respects triangle and diagonal
visibility. Numeric coefficient/value annotations are never generated for
missing cells.

The default legacy engine continues to delegate all seven glyph methods and
these options directly to corrplot 0.95. Scene mode currently supports circle
and number glyphs only; missing-cell styling is shared across those supported
methods. An all-missing general matrix is represented as a missing-only scene
with `matrix_semantics.has_observed_values = false`; the fallback domain is
not evidence of observed numeric values.
