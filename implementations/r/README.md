# R implementation

This directory is the root of the provisional R package.

- `DESCRIPTION`, `NAMESPACE`, and `LICENSE` define the R package.
- `R/` contains the R interface.
- `man/` contains package help pages.
- `tests/` contains R-specific tests.

For now, `corrplot_ng()` delegates to the original `corrplot` package. It is a compatibility shim, not a new plotting engine.

Run checks from this directory:

```sh
R CMD check .
```

From the repository root, use:

```sh
R CMD check implementations/r
```
