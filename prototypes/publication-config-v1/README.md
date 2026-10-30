# Publication figure configuration, version 1

This directory defines the first declarative configuration format for reproducible corrplot-ng figures. It is an input contract, separate from the renderer-neutral prepared-scene contract in `prototypes/shared-svg/SCENE-CONTRACT.md`.

## Contract

- `config_version` identifies this format. Incompatible changes require a new major configuration version.
- `input` is either an inline labelled matrix or a path to a CSV matrix. CSV paths are resolved relative to the configuration file's directory by consumers.
- `matrix.kind` identifies the semantic matrix type. Version 1 supports correlation matrices; `method` records the correlation method used to produce the matrix.
- Correlation values are in [-1, 1]; JSON null represents a missing correlation and is never interpreted as zero.
- `plot` contains semantic display choices: glyph method, displayed triangle, ordering, coefficient labels, and optional significance metadata. The p-value matrix is required when significance is enabled.
- `colors` contains the legacy negative, neutral, positive, and missing colors. Optional `colors.scale` defines an explicit continuous or discrete signed-correlation scale; see [COLOR-SCALES.md](COLOR-SCALES.md).
- `dimensions` are physical output dimensions, not logical scene units. Consumers convert mm or inches to the renderer's output coordinate system.
- `typography` records the requested font family and base size in points.
- `output` selects SVG and a destination path. PDF/PNG and renderer-specific options are deliberately deferred.

## Validation and normalization

The JSON Schema rejects unknown fields and validates local field constraints. The Python validator additionally checks square dimensions, unique labels, matrix symmetry, diagonal values, and color-scale coverage, ordering, zero handling, and signed-domain symmetry. CSV parsing and validation are performed by the eventual data-loading consumer.

Normalized configurations preserve supplied values and source matrix/label order. Scene adaptation applies the requested display permutation without mutating the source config. Canonical JSON uses lexicographically sorted object keys, compact separators, UTF-8 text, and no numeric rounding. This provides stable semantic comparisons without changing numeric precision. A JSON Schema validator does not mutate input documents.

## Scene adaptation

`to_scene_config(config)` converts an inline-matrix publication config into the renderer-neutral scene input. It supplies one mapped color per cell and ordered legend entries. Missing values use `colors.missing`; zero remains a valid value and is not treated as missing. CSV-backed configs are validated at the config boundary but require a data-loading step before scene adaptation.

## Validation rules beyond JSON Schema

1. Each inline matrix has exactly N rows and each row exactly N values, where N is the number of labels.
2. Inline correlation matrices are symmetric within an absolute tolerance of 1e-8. A missing value must have a matching missing value in its symmetric position.
3. If significance is enabled, `p_values` is an N by N symmetric matrix with values in [0, 1].
4. Continuous scale stops or discrete breaks cover [-1, 1], are strictly increasing, symmetric around zero, and include zero.
5. Discrete intervals are left-closed/right-open, except for the final interval, which includes its upper endpoint.
6. CSV input resolves to a square numeric matrix with unique, non-empty labels.
7. Ordering changes display order only; it must not rewrite source data or labels.

## Ordering semantics

- `original` preserves input order.
- `alphabetical` sorts labels by Unicode case-folded text, then original text, then original index.
- `hclust` uses average linkage over `d(i,j) = 1 - r(i,j)`. Cluster-pair distance is the arithmetic mean of observed cross-cluster distances; missing correlations are omitted. If a variable has no observed off-diagonal correlation, or remaining clusters have no observed cross-pair, clustering fails with a diagnostic. Equal distances are broken by the smallest original index in each cluster, and child orientation uses that same key.
- `aoe` remains in the schema for compatibility but is rejected during scene adaptation until its eigenvector sign, tie, and degenerate-matrix semantics are specified.

## Evolution

A change is backward-compatible only when existing version-1 documents retain the same meaning and remain valid. Changes to matrix interpretation, defaults, required fields, or value ranges require a new `config_version`. Additions to this strict schema require an explicit compatibility decision and schema revision.

## Examples

- `examples/basic.json`: basic full-matrix correlation plot.
- `examples/significance.json`: upper triangle with coefficient and significance annotations.
- `examples/publication-size.json`: CSV-backed, publication-sized figure with explicit typography and physical dimensions.
- `examples/diverging-scale.json`: continuous diverging scale with a neutral midpoint.
- `examples/discrete-scale.json`: explicit symmetric value breaks and interval colors.

The format is intentionally a small starting point. It does not promise complete coverage of legacy R arguments or define renderer geometry.
