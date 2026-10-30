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
