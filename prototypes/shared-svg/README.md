# Shared SVG renderer spike

This is a small architecture experiment, not production code and not a commitment to Python, Rust, or SVG as the final design. The generated scene is checked against the experimental JSON Schema in `scene-v1.schema.json`.

It accepts a language-neutral JSON object and renders a correlation matrix as deterministic SVG. The example includes positive, negative, and missing values, plus one user-defined text annotation.

Run from the repository root:

```sh
python prototypes/shared-svg/renderer.py prototypes/shared-svg/example.json
python prototypes/shared-svg/renderer.py prototypes/shared-svg/figure-text-example.json
python -m pip install jsonschema
python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v
Rscript prototypes/shared-svg/test_r_scene_consumer.R
```

## Label layout

The scene builder uses deterministic, renderer-independent estimates rather
than browser font measurement. Row-label margins expand to fit the longest
label, and vertical column labels receive dedicated top space. Optional
`layout` settings can control physical width/height, cell size bounds, label
font family/size/weight, column-label orientation, and collision policy.

Example:

```json
{
  "labels": ["customer satisfaction", "monthly recurring revenue"],
  "matrix": [[1.0, 0.42], [0.42, 1.0]],
  "layout": {
    "width": 720,
    "height": 560,
    "min_cell_size": 18,
    "max_cell_size": 96,
    "label_font_family": "serif",
    "label_font_size": 14,
    "label_font_weight": "bold",
    "column_label_orientation": "horizontal",
    "label_collision": "truncate"
  }
}
```

Supported collision policies are `preserve`, `wrap`, `truncate`, and `omit`.
Horizontal labels can wrap or truncate to the available cell width; vertical
labels avoid neighboring columns by construction and can also be truncated. Font metrics are estimates,
so output should be visually reviewed with the target font when labels contain
unusually wide glyphs. If requested dimensions cannot fit the matrix at the
minimum cell size, the minimum remains authoritative and the canvas expands.

## Figure-level publication text

Use the optional `figure_text` array for visible publication text. Each
entry has one semantic role: `title`, `subtitle`, `caption`, or
`footnote`. The scene builder orders roles consistently, wraps long and
multi-line text without truncation, and expands the canvas when required.
The SVG backend escapes XML-sensitive text and preserves Unicode.

Each role may override `alignment` (`left`, `center`, or `right`),
`font_family`, `font_size`, `font_weight`, `font_style`, and
`spacing_after`. Defaults are role-specific: title and subtitle are centered
above the matrix; caption and footnote are left-aligned below it.

```json
{
  "title": "Accessible figure name",
  "description": "A concise description for assistive technology.",
  "figure_text": [
    {"role": "title", "text": "Treatment response correlations"},
    {"role": "subtitle", "text": "Pearson correlations across study groups"},
    {"role": "caption", "text": "Figure 2. Pairwise correlations."},
    {"role": "footnote", "text": "Synthetic data; values are illustrative."}
  ]
}
```

The top-level `title` and `description` remain accessible SVG metadata
(`<title>` and `<desc>`); they are not visible figure text. This keeps
screen-reader identification separate from the visible title and caption.
See [the complete publication example](figure-text-example.json).

## Cluster-region annotations

The optional `cluster_regions` array describes rectangles around groups of
variables. Each region has a stable `id`, a six-digit hex `color`, and a
positive `line_width`. Identify members with exactly one of:

- `variables`: unique variable names;
- `indices`: unique zero-based source indices.

When `variable_order` is present, its `labels` and `indices` arrays map
each displayed position back to the stable source variable identity. Regions
are resolved against that mapping after ordering. A region must occupy a
contiguous range in the displayed order; unknown or ambiguous members fail
validation instead of producing a misleading rectangle.

The resulting scene stores both stable variable names and source indices,
plus inclusive row/column cell boundaries and renderer-neutral geometry.
The SVG backend draws the rectangle as an overlay. Geometry is unchanged for
full, upper, lower, and diagonal-only display layouts.

```json
{
  "variable_order": {
    "strategy": "hclust",
    "indices": [2, 0, 1],
    "labels": ["gamma", "alpha", "beta"]
  },
  "cluster_regions": [
    {"id": "cluster-ab", "variables": ["alpha", "beta"], "color": "#123456", "line_width": 2}
  ]
}
```

Index-based input uses the original source order:

```json
{
  "cluster_regions": [
    {"id": "cluster-ab", "indices": [0, 1], "color": "#123456"}
  ]
}
```

The complete [cluster-region fixture](cluster-region-example.json) demonstrates name-based resolution after a reordered leaf order. The legacy R `corrRect()` and `corrRect.hclust()` adapters are tracked separately; this scene contract contains no R graphics-device behavior.

## Cell encodings

The scene and SVG backend support all seven publication methods:

| Method | Visual mapping | Example |
| --- | --- | --- |
| `circle` | Circle radius grows linearly with absolute value; sign selects positive/negative color | `{"method":"circle"}` |
| `square` | Centered square side grows linearly with absolute value | `{"method":"square"}` |
| `ellipse` | Eccentricity grows with magnitude; diagonal orientation distinguishes sign | `{"method":"ellipse"}` |
| `number` | Prints the formatted numeric value instead of a shape | `{"method":"number"}` |
| `shade` | Magnitude controls tint/line strength; diagonal direction distinguishes sign | `{"method":"shade"}` |
| `color` | Fills the whole cell, interpolating neutral toward the signed color by magnitude | `{"method":"color"}` |
| `pie` | Sector sweep is proportional to absolute value; sweep direction and color distinguish sign | `{"method":"pie"}` |

For every method, `null` remains an explicit missing-value cell labelled `NA`; it is never treated as zero. A supplied per-cell mapped color takes precedence over fallback semantic colors. The same method can be selected globally or independently for upper, lower, and diagonal regions (`upper_method`, `lower_method`, and `diagonal_method`).

## What this validates

- A small input model can be rendered without language-specific API objects.
- The same input produces byte-for-byte stable SVG in this prototype.
- Missing values and a user annotation can be represented in the output.
- Basic input constraints can be checked before rendering.
- Long row and column labels receive dedicated layout space and regression coverage.
- The legacy-output oracle exercises exact -1/+1 correlations, a constant-like
  variable with undefined off-diagonal correlations, and a fully missing
  row/column (including its diagonal). The bridge checks that JSON nulls remain
  missing rather than becoming zero.
- These are targeted semantic edge cases, not a claim that every malformed or
  statistically degenerate matrix is accepted by the legacy plotting API.

## What this does not validate

- Visual parity with the original R package.
- The full corrplot feature set or a final shared-core schema.
- Cross-language bindings, Rust, performance, or production packaging.
- Drawing on the active R graphics device or follow-up calls such as `text()` and `segments()`.

The R graphics integration question remains open and needs a separate R-based experiment. This spike should be used to inform the architecture, not treated as a compatibility implementation.

## Accessible SVG metadata

The input may provide an accessible `title` and concise `description`; the scene always carries both fields. The SVG backend emits a referenced `<title>` and, when a non-empty description is supplied, a referenced `<desc>`. Visible title, subtitle, caption, and footnote content is carried separately in `figure_text`. The root SVG uses `role="img"`, `aria-labelledby`, and (when applicable) `aria-describedby`. Metadata and visible text are XML-escaped. These labels improve figure identification but do not make a dense correlation matrix fully accessible; provide an accompanying data table when users need to inspect individual values.
