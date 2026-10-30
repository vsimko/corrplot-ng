# Shared SVG renderer spike

This is a small architecture experiment, not production code and not a commitment to Python, Rust, or SVG as the final design. The generated scene is checked against the experimental JSON Schema in `scene-v1.schema.json`.

It accepts a language-neutral JSON object and renders a correlation matrix as deterministic SVG. The example includes positive, negative, and missing values, plus one user-defined text annotation.

Run from the repository root:

```sh
python prototypes/shared-svg/renderer.py prototypes/shared-svg/example.json
python -m pip install jsonschema
python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v
Rscript prototypes/shared-svg/test_r_scene_consumer.R
```

## Label layout

Row-label space expands to accommodate the longest label using a conservative
sans-serif width estimate. Column labels are rotated vertically so neighboring
columns cannot overlap; the top margin expands to fit them. The estimate is a
layout heuristic rather than font measurement, so unusually wide glyphs may
still need visual review with the target font.

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
