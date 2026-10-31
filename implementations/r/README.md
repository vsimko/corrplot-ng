# R implementation

This directory is the root of the provisional R package.

- DESCRIPTION, NAMESPACE, and LICENSE define the R package.
- R/ contains the R interface.
- man/ contains package help pages.
- tests/ contains R-specific tests.

## Adapter status

corrplot_ng() has two explicit paths:

- The default engine = "legacy" delegates to the original corrplot package and preserves its plotting behavior.
- Experimental engine = "scene" converts a supported square correlation matrix into a renderer-neutral scene-v1 object. It does not draw a plot yet. Unsupported plotting arguments are rejected instead of being ignored.

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

This helper does not calculate correlations and does not change the behavior
of `corrplot_ng()`. The file adapter and integration into the scene pipeline
are tracked separately in issues #132 and #68.
