# Legacy R adapter design

**Status:** Initial implementation slice; the adapter remains experimental.

This document defines how the legacy R API is introduced to the shared scene model without silently changing established corrplot behavior. The pinned upstream API inventory in [legacy-api-compatibility.md](legacy-api-compatibility.md) remains the source of truth for names, formals, defaults, and accepted values.

## Routing contract

corrplot_ng() exposes an explicit engine selector:

- engine = "legacy" is the default and delegates to corrplot::corrplot(). The adapter draws on the caller's active device, does not open or close devices, and restores writable graphics parameters after successful calls and errors. The upstream `add = TRUE` overlay semantics are retained.
- engine = "scene" validates the currently supported subset and returns a corrplot_ng_scene object conforming to scene-v1. It prepares data; it does not draw a plot or select a glyph renderer.
- An explicitly requested scene operation never falls back to the legacy engine. Unsupported options produce an actionable error naming the unsupported arguments.

The scene selector is intentionally explicit. It avoids guessing whether a call can be translated and makes the returned value's type predictable.

## Standalone color legend helper

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
| add, mar, device state and helper drawing | R graphics-device behavior remains legacy-only; calls compose on the active device and restore writable graphics parameters | Legacy-only |

General-matrix mode is selected explicitly with `is.corr = FALSE`. It preserves original cell values and records `matrix_semantics` with the selected mode, resolved color limits, palette family/name, and normalization parameters. One-sided matrices use the legacy `YlOrBr` sequential palette; mixed-sign matrices use `RdBu`. Each observed cell carries its mapped color and visual-only normalized value, leaving `value` unchanged. Automatic general-matrix limits use observed values and exclude the diagonal when `diag = FALSE`; a constant-valued matrix receives finite padding of max(0.5, 5% of the absolute constant). An all-missing general matrix remains a valid missing-only scene. It carries has_observed_values = false and a neutral fallback color domain that must not be presented as observed numeric data. Explicit `col.lim` must be finite, strictly increasing, and cover every observed value. Correlation mode retains its [-1, 1] validation and defaults to [-1, 1].

The scene mode accepts the adapter-specific annotation option, coefficient/value text options, explicit matrix semantic mode and color limits, and the `method`, `type`, and `diag` subset described above. The adapter returns renderer-neutral mapped cell metadata but does not itself invoke an SVG or graphics-device renderer. Other arguments are rejected rather than ignored.

## Migration sequence

1. Establish explicit routing and preserve the default legacy path.
2. Implement and test a narrow scene-v1 input slice (matrix, labels, missing values, annotation).
3. Extend the versioned scene contract before routing additional legacy semantics. Coefficient/value text is the first annotation slice; ordering, the remaining glyph methods, significance, and confidence intervals remain separate work.
4. Add a renderer integration before describing scene mode as a plotting backend.
5. Expand compatibility fixtures and compare semantic results against the pinned upstream implementation for each newly routed capability.
6. Keep R graphics-device operations and unsupported options on the legacy path until an equivalent behavior is specified and tested.

## Graphics-device lifecycle

The legacy adapter treats the active graphics device as caller-owned. It records the active device and writable graphics parameters before delegating to upstream corrplot, then restores the original device selection (if it is still open) and parameters on both success and error. It never opens or closes a device itself. `add = TRUE` remains an upstream overlay operation and draws into the caller's existing plot; it is not silently changed into a new plot. Pixel-identical results across graphics devices are not promised.

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
