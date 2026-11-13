# Visualization model v1 fixtures

These fixtures exercise distinct semantic uses of the shared renderer-neutral model:

- `legacy-corrplot.json`: a small correlation matrix represented as the legacy adapter should produce it.
- `general-matrix.json`: a non-correlation numeric matrix with a sequential scale.
- `modern-bar-chart.json`: a non-correlation, composable visualization with multiple semantic layers.

All examples are synthetic. They define data and visual meaning, not SVG/Canvas paths or device pixels. Schema validation is necessary but not sufficient: Rust conformance tests must validate references, dimensions, and semantic constraints as well.
