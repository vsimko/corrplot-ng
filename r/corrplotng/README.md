# R implementation

This directory is the root of the provisional R package.

- DESCRIPTION, NAMESPACE, and LICENSE define the R package.
- R/ contains the R interface.
- man/ contains package help pages.
- tests/ contains R-specific tests.

## Adapter status

`corrplot_ng()` has two explicit paths:

- The default `engine = "legacy"` delegates to the original corrplot package and preserves its plotting behavior.
- Experimental `engine = "scene"` converts a supported square correlation matrix into a renderer-neutral scene-v1 object. It prepares scene data; it does not draw a plot yet. Unsupported plotting arguments are rejected instead of being ignored.

See [the adapter design](../../docs/legacy-r-adapter.md) for argument mapping, current limitations, and the migration sequence.

## Legacy return values

The legacy `corrplot_ng()` and `corrplot_ng_mixed()` wrappers return the
upstream invisible objects unchanged. The main result contains the reordered
`corr` matrix, the `corrPos` cell-position data frame, and minimal `arg`
metadata; the mixed result contains `corr` and combined `corrPos` only. See
[the return-value contract](../../docs/legacy-r-return-values.md) for
coordinate conventions and preserved diagonal behavior.

## Graphics device ownership and composition

The legacy path is a thin delegation to `corrplot::corrplot`. The caller owns
the active graphics device: `corrplot_ng()` does not open, select, or close
devices. With the default `add = FALSE`, the upstream function creates its
plot on the current device. With `add = TRUE`, it draws into the current
plot window and preserves that window's user coordinates so base-graphics
layers can be composed before or after the corrplot.

The upstream implementation temporarily changes `par(mar, bg)` and restores
those parameters with `on.exit()` on both successful and failed rendering.
The adapter deliberately does not snapshot and restore every graphics
parameter: doing so would restore plot coordinates and invalidate the
existing plot window used by `add = TRUE`. The adapter also does not call
`dev.off()`; closing a device is always the caller's responsibility.

The experimental `engine = "scene"` path only prepares a scene-v1 object and
does not access a graphics device. Regression tests in
`tests/testthat/test-graphics-device.R` cover current-device ownership,
`add = TRUE` composition, restoration after an error, and the scene path's
absence of graphics side effects.

## Cluster rectangles

`corrRect()` preserves the legacy index, name, and `namesMat` drawing workflows. Index/name ranges and diagonal square `namesMat` inputs are normalized to the shared scene-v1 cluster-region request shape; arbitrary `namesMat` rectangles remain R-only because they are not diagonal cluster regions. `corrRect.hclust()` creates the same requests from contiguous dendrogram leaf groups. Both functions draw on the active device and never open or close it. Index/name ranges require identical row and column names; `namesMat` can address matrices with distinct row and column labels. The `corrplot_ng()` legacy pass-through retains `addrect`, `rect.col`, and `rect.lwd` unchanged.

See [the adapter design](../../docs/legacy-r-adapter.md) and the `corrRect` / `corrRect.hclust` help pages.

Run checks from this directory:

```sh
R CMD check .
```

From the repository root, use:

```sh
R CMD check implementations/r
```

## Normalized matrix input adapter

`corrplot_ng_normalize_matrix(x, metadata = NULL)` converts an already-computed
square numeric matrix or numeric data frame into the language-neutral
normalized-matrix v1 object defined in
`prototypes/normalized-matrix-v1/schema.json`.

- If both row and column names are supplied, they must match exactly. If only
  one axis has names, those labels are used for both; otherwise `V1` through
  `Vn` are generated.
- Unicode labels are preserved. Empty, duplicate, or mismatched labels fail.
- R `NA` becomes `NULL`; numeric zero remains zero. `NaN`, infinities,
  non-square inputs, and non-numeric data-frame columns are rejected.
- All-missing and constant matrices are valid normalized inputs. Whether an
  all-missing matrix can be rendered is decided by the later scene stage.
- Source indices are zero-based and preserve the original row/column order.

This helper does not calculate correlations. The file adapter emits the same
normalized-matrix v1 contract, with source format/path retained as provenance.
It remains a separate input helper; it does not implicitly feed the scene
engine or render plots.

### File input adapter

`corrplot_ng_normalize_file(path, format = "auto", header = TRUE, row_labels = TRUE)`
reads CSV/TSV matrices using the publication-config layout or reads an existing
normalized-matrix v1 JSON document (requires the suggested `jsonlite` package).
CSV/TSV header and row-label options are explicit: if both axes are labelled,
they must match exactly; if only one is labelled, that axis supplies both
normalized labels; if neither is labelled, `V1` through `Vn` are generated.
Missing values stay `NULL`, numeric zero stays zero, and source format/path
are retained as provenance. This is a normalization adapter only; it does not
calculate correlations or render plots.

## Pairwise correlation testing helper

The exported `cor.mtest(mat, ...)` helper computes pairwise p-values and
confidence intervals with `stats::cor.test()`. Rows are observations and
columns are features; additional test arguments such as `method`,
`alternative`, `exact`, `continuity`, and `conf.level` are forwarded.

The returned `p`, `lowCI`, and `uppCI` matrices are symmetric and retain
feature names. The diagonal uses corrplot 0.95 conventions (p = 0 and CI
limits = 1). Pairwise complete observations are used independently for each
pair. Pairs with fewer than three complete observations or a constant feature
return missing results; off-diagonal confidence limits remain missing for
methods that do not provide them. Non-numeric columns and infinite values are
rejected with actionable errors.

This helper computes statistical metadata only. The plotting scene and
renderers consume supplied p-value/CI matrices and do not perform statistical
tests.

## Structured diagnostics

Input validation errors from the normalized-matrix adapter use the
language-neutral diagnostic-v1 contract. They are signalled as conditions
with class `corrplot_ng_error`; read the stable code, stage, path, and optional
zero-based matrix coordinates with `corrplot_ng_diagnostic(condition)`. The
human-readable message is for display, not programmatic branching. See the
[diagnostic catalogue](../../docs/diagnostics.md).
