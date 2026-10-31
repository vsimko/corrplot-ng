# R implementation

This directory is the root of the provisional R package.

- DESCRIPTION, NAMESPACE, and LICENSE define the R package.
- R/ contains the R interface.
- man/ contains package help pages.
- tests/ contains R-specific tests.

## Adapter status

`corrplot_ng()` has two explicit paths:

- The default `engine = "legacy"` delegates to the original corrplot package, draws on the caller's active device, preserves `add = TRUE` overlays, and restores writable graphics parameters after success or failure. The adapter does not open or close caller-owned devices.
- Experimental `engine = "scene"` converts a supported square correlation matrix into a renderer-neutral scene-v1 object. It prepares scene data; it does not draw a plot yet. Unsupported plotting arguments are rejected instead of being ignored.

See [the adapter design](../../docs/legacy-r-adapter.md) for argument mapping, current limitations, and the migration sequence.

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
