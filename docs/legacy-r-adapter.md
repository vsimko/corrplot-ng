# Legacy R adapter design

**Status:** Initial implementation slice; the adapter remains experimental.

This document defines how the legacy R API is introduced to the shared scene model without silently changing established corrplot behavior. The pinned upstream API inventory in [legacy-api-compatibility.md](legacy-api-compatibility.md) remains the source of truth for names, formals, defaults, and accepted values.

## Routing contract

corrplot_ng() exposes an explicit engine selector:

- engine = "legacy" is the default and delegates to corrplot::corrplot(). Existing calls therefore retain the established plotting behavior and graphics-device side effects.
- engine = "scene" validates the currently supported subset and returns a corrplot_ng_scene object conforming to scene-v1. It prepares data; it does not draw a plot or select a glyph renderer.
- An explicitly requested scene operation never falls back to the legacy engine. Unsupported options produce an actionable error naming the unsupported arguments.

The scene selector is intentionally explicit. It avoids guessing whether a call can be translated and makes the returned value's type predictable.

## Initial argument mapping

| Legacy concept / input | Scene-v1 mapping | Initial status |
|---|---|---|
| corr | Numeric square matrix; observed values must be finite and in [-1, 1] | Supported |
| rownames(corr) / colnames(corr) | One ordered labels vector; both sets must match when both are present | Supported |
| Missing matrix values | Cell value: null; never coerced to zero | Supported |
| annotation (corrplot-ng adapter option) | Scene-level plain-text annotation | Supported |
| method | No glyph/method field in scene-v1 | Rejected in scene mode |
| type, diag | Triangle and diagonal visibility are not represented in scene-v1 | Rejected in scene mode |
| order, hclust.method | No permutation/order semantics in scene-v1 | Rejected in scene mode |
| col, col.lim, is.corr | Semantic color-scale and general-matrix rules are not represented sufficiently | Rejected in scene mode |
| Labels (tl.*) | Text styling, plotmath parsing, placement and rotation are not represented | Rejected in scene mode |
| Coefficients, significance and confidence intervals | No corresponding annotation metadata in scene-v1 | Rejected in scene mode |
| add, mar, device state and helper drawing | R graphics-device behavior remains legacy-only | Legacy-only |

The scene mode accepts only the adapter-specific annotation option in addition to corr. All other ... arguments are rejected rather than ignored.

## Migration sequence

1. Establish explicit routing and preserve the default legacy path.
2. Implement and test a narrow scene-v1 input slice (matrix, labels, missing values, annotation).
3. Extend the versioned scene contract before routing additional legacy semantics, starting with ordering and glyph/display semantics.
4. Add a renderer integration before describing scene mode as a plotting backend.
5. Expand compatibility fixtures and compare semantic results against the pinned upstream implementation for each newly routed capability.
6. Keep R graphics-device operations and unsupported options on the legacy path until an equivalent behavior is specified and tested.

## Compatibility guarantees and non-goals

- The default engine remains the legacy implementation.
- Scene mode fails closed when it receives unsupported arguments; it does not silently drop options or silently switch engines.
- This slice does not claim visual parity, full API compatibility, or a production renderer.
- No legacy function is removed or redefined, and this work does not decide the production implementation language.
