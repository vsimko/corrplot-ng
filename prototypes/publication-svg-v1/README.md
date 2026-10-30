# Publication SVG export, version 1

This prototype adds a deterministic publication-oriented SVG export boundary on top of the renderer-neutral scene-v1 contract. It does not redefine correlation semantics or add renderer geometry to the shared scene.

## API

Call `render_publication_svg(scene, width_mm=..., height_mm=..., font_family=..., font_size_pt=...)` with a scene produced by `prototypes/shared-svg/scene.py`.

- Width and height are required physical dimensions in millimetres and are written to the SVG root as `mm` lengths.
- The viewBox remains in scene logical units. `preserveAspectRatio="xMidYMid meet"` avoids distorting cells when the requested page aspect ratio differs from the scene.
- Font family and point size are explicit output settings. Text and attributes are XML-escaped.
- Label margins expand for larger requested fonts so row and rotated column labels remain inside the SVG viewport.
- Optional cell colors override fallback sign colors. An explicit zero value receives a small visible marker, while missing values retain their separate mapped fill and NA label.
- Optional renderer-neutral legend entries are drawn as ordered swatches below the plot. The logical viewBox expands in both dimensions to keep all swatches and labels inside the SVG.
- SVG element order, attribute order, and numeric formatting are stable for the same scene and options.

The font-width estimate is deliberately conservative and does not embed fonts. The SVG requests the selected family; viewers and publication pipelines must provide or substitute it. Cross-platform glyph metrics can therefore differ slightly. Review the rendered figure with the intended publication toolchain.

## Test

From the repository root:

```sh
python -m unittest discover -s prototypes/publication-svg-v1 -p 'test_*.py' -v
```

This is an initial export contract and regression suite, not a promise of pixel-identical rendering across operating systems. PDF and PNG export remain out of scope.
