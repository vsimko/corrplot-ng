# R legacy return-value contract

This contract is R-specific. It preserves the upstream objects returned by
`corrplot::corrplot()` and `corrplot::corrplot.mixed()`; these R lists are
not part of scene-v1 or the shared Rust API.

## Adapter entry points

- `corrplot_ng(..., engine = "legacy")` delegates directly to
  `corrplot::corrplot()` and returns its invisible result unchanged.
- `corrplot_ng_mixed(..., engine = "legacy")` delegates directly to
  `corrplot::corrplot.mixed()` and returns its invisible result unchanged.
- `corrplot_ng(..., engine = "scene")` returns a visible scene-v1 list and
  does not promise legacy `corrPos` compatibility.
- `corrplot_ng_mixed(engine = "scene")` fails explicitly; mixed scene
  composition is not implemented.

## corrplot return object

The legacy result is an invisible list with exactly these top-level fields:

| Field | Meaning |
| --- | --- |
| `corr` | Correlation/general matrix after the requested ordering permutation; row and column names reflect that reordered matrix |
| `corrPos` | Data frame of finite numeric cells selected by `type` and `diag` |
| `arg` | Minimal argument metadata; currently `arg$type` records `full`, `upper`, or `lower` |

The base `corrPos` columns are `xName`, `yName`, `x`, `y`, and
`corr`. If `p.mat` is supplied, a `p.value` column is appended.

### Coordinate convention

Coordinates are one-based and refer to the reordered matrix:

- `x` is the column index, increasing left to right.
- `y` is `nrow(corr) - row_index + 1`; the first matrix row is at the top
  (highest y), and the last row is at the bottom (y = 1).
- `xName` is the reordered column name at `x`; `yName` is the reordered
  row name corresponding to the source matrix row.
- Rows are sorted by increasing `x`, then decreasing `y`.
- `type = "full"` retains all cells; `"upper"` retains row <= column;
  `"lower"` retains row >= column. `diag = FALSE` removes diagonal cells
  before positions are constructed.

Missing (`NA`) cells are not included in `corrPos`: the upstream result
contains finite numeric values only. Missing-cell text/fill metadata is a
separate rendering concern and is not retrofitted into this legacy data frame.

## corrplot.mixed return object

The mixed result is an invisible list with exactly two top-level fields:
`corr` (the reordered matrix from the upper call) and `corrPos` (the
combined upper/lower position data frame). It has no `arg` field. The
combined positions are sorted by increasing `x`, then decreasing `y`.

The upstream mixed helper has observable diagonal quirks that are preserved:

- `diag = "n"` calls the upper renderer with its diagonal enabled and the
  lower renderer with it disabled, then may overdraw the diagonal with blank
  rectangles. The returned `corrPos` still contains the upper call's
  diagonal positions.
- `diag = "l"` includes diagonal positions from both upper and lower
  results, so duplicate diagonal coordinate rows are retained.
- `diag = "u"` includes the upper result's diagonal positions only.

The adapter does not deduplicate, reorder, rename, or otherwise normalize
these upstream return objects. This keeps existing scripts and the upcoming
R rectangle adapter (#148) aligned with the actual plotted coordinates.

## Unsupported/internal fields

The adapter does not synthesize fields for colors, glyph geometry, hidden
cells, missing-cell positions, or the complete call argument list. The
`arg` list is not a complete call record; currently it contains only
`type`. Consumers must use the documented columns and not depend on private
upstream temporaries or undocumented data-frame construction details.

The contract is tested against the pinned corrplot 0.95 implementation in
`implementations/r/tests/testthat/test-return-values.R`.
