# Legacy palette compatibility

The R compatibility package exports `COL1()` and `COL2()` with the corrplot
0.95 palette families, defaults, and interpolation behavior. Both helpers
default to 200 colors. `COL1()` provides the ten sequential families and
`COL2()` provides the six diverging families; diverging palettes retain the
white neutral anchor used by corrplot 0.95.

Palette anchors and behavior are pinned to upstream commit
`62532f700e6c9d04b93ffbf0bf56e264b0e0368e`. Regression tests compare every
family at lengths 1, 2, 3, 11, and 200 with the pinned upstream package. The
palette helpers return ordinary hexadecimal color vectors; they do not define
breaks, interpolation policy, or legend semantics.

For publication configuration, pass a legacy color vector through
`legacy_palette_scale()` in `color_scale.py`. It adapts the vector to the
existing discrete color-scale contract, preserving the order and colors.
When an odd number of colors would place zero inside a bin, the adapter splits
that bin at zero and repeats its color, so the resulting scale has the required
explicit neutral boundary without changing the mapped colors. Generic scale
mapping and legend behavior remain owned by the shared color-scale contract.
