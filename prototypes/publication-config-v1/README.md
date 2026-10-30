# Publication figure configuration, version 1

This directory defines the first declarative configuration format for reproducible corrplot-ng figures. It is an input contract, separate from the renderer-neutral prepared-scene contract in `prototypes/shared-svg/SCENE-CONTRACT.md`.

## Contract

- `config_version` identifies this format. Incompatible changes require a new major configuration version.
- `input` is either an inline labelled matrix or a path to a CSV matrix. CSV paths are resolved relative to the configuration file's directory by consumers.
- `matrix.kind` explicitly selects `correlation` or `general_numeric`; semantics are never inferred from dimensions or values. `method` records the correlation method when applicable.
- Correlation values are in [-1, 1]. General numeric matrices accept finite values outside that range. JSON null represents a missing value and is never interpreted as zero.
- General numeric matrices derive automatic limits from observed values or accept an explicit increasing `matrix.col_limits` pair that must cover all observations. All-missing matrices are rejected; constant ranges are padded deterministically.
- General numeric values are normalized only for visual mapping: mixed-sign observed values use the legacy `RdBu` diverging palette; one-sided values use the legacy `YlOrBr` sequential palette. Explicit custom scales override the default palette. Original values remain unchanged in the prepared scene.
- `plot` contains semantic display choices: glyph method, displayed triangle, ordering, coefficient labels, and optional significance metadata. The p-value matrix is required when significance is enabled.
- `colors` contains the legacy negative, neutral, positive, and missing colors. Optional `colors.scale` defines a continuous or discrete signed scale, or a sequential scale over [0, 1] for general numeric matrices. See [COLOR-SCALES.md](COLOR-SCALES.md).
- `dimensions` are physical output dimensions, not logical scene units. Consumers convert mm or inches to the renderer's output coordinate system.
- `typography` records the requested font family and base size in points.
- `output` selects SVG and a destination path. PDF/PNG and renderer-specific options are deliberately deferred.

## Significance and confidence intervals

When `plot.significance.enabled` is true, `p_values` must be a symmetric matrix. Missing p-values are represented as `null` and remain unknown rather than being treated as nonsignificant. The default thresholds are `[0.001, 0.01, 0.05]`, in strictly increasing order; the default mode is `pch` with the `×` marker.

Supported significance modes:

- `pch`: place the configured symbol on cells with p-values above the largest threshold.
- `p-value`: print the p-value on nonsignificant cells.
- `blank`: suppress nonsignificant cells while preserving cells with missing p-values.
- `stars`: annotate significant cells with `***`, `**`, or `*` according to the configured thresholds.

Optional `plot.confidence_intervals` contains symmetric `lower` and `upper` matrices. Both bounds must be present or both null, each lower bound must not exceed its upper bound, and correlation intervals must lie in [-1, 1]. The current publication adapter accepts confidence intervals for inline correlation matrices; the scene stores original bounds plus normalized whisker geometry for deterministic SVG rendering.

## Validation and normalization

The JSON Schema rejects unknown fields and validates local field constraints. The Python validator additionally checks square dimensions, unique labels, finite values, correlation-only symmetry and unit diagonal, and color-scale coverage, ordering, zero handling, and signed-domain symmetry. General numeric matrices do not require symmetry or a unit diagonal. CSV parsing and validation are performed by the eventual data-loading consumer.

Normalized configurations preserve supplied values and source matrix/label order. Scene adaptation applies the requested display permutation without mutating the source config. Canonical JSON uses lexicographically sorted object keys, compact separators, UTF-8 text, and no numeric rounding. This provides stable semantic comparisons without changing numeric precision. A JSON Schema validator does not mutate input documents.

## Scene adaptation

`to_scene_config(config)` converts an inline-matrix publication config into the renderer-neutral scene input. It supplies one mapped color per cell and ordered legend entries. Missing values use `colors.missing`; zero remains a valid value and is not treated as missing. CSV-backed configs are validated at the config boundary but require a data-loading step before scene adaptation.

## Validation rules beyond JSON Schema

1. Each inline matrix has exactly N rows and each row exactly N values, where N is the number of labels.
2. Inline correlation matrices are symmetric within an absolute tolerance of 1e-8. A missing value must have a matching missing value in its symmetric position.
3. If significance is enabled, `p_values` is a non-null N by N symmetric matrix with values in [0, 1]; individual null entries remain unknown and are not treated as nonsignificant. Thresholds default to `[0.001, 0.01, 0.05]` and must be strictly increasing. `stars` annotates significant cells at each threshold, `pch` marks nonsignificant cells with the configured symbol, `p-value` prints nonsignificant p-values, and `blank` hides nonsignificant glyphs. The renderer does not calculate p-values.
4. Supplied confidence-interval lower and upper bounds must both be numeric or both null, be symmetric, lie in [-1, 1], contain the corresponding correlation estimate, and be null for missing correlations.
5. Continuous scale stops or discrete breaks cover [-1, 1], are strictly increasing, symmetric around zero, and include zero. Sequential scale stops cover [0, 1] and are strictly increasing.
6. Discrete intervals are left-closed/right-open, except for the final interval, which includes its upper endpoint.
7. CSV input resolves to a square numeric matrix with unique, non-empty labels.
8. Ordering changes display order only; it must not rewrite source data or labels.

## Ordering semantics

- `original` preserves input order.
- `alphabetical` sorts labels by Unicode case-folded text, then original text, then original index.
- `hclust` is supported for correlation matrices and uses average linkage over `d(i,j) = 1 - r(i,j)`. Cluster-pair distance is the arithmetic mean of observed cross-cluster distances; missing correlations are omitted. If a variable has no observed off-diagonal correlation, or remaining clusters have no observed cross-pair, clustering fails with a diagnostic. Equal distances are broken by the smallest original index in each cluster, and child orientation uses that same key.
- `aoe` is supported for complete correlation matrices. It orders variables by the angle `atan2(v₂, v₁)` from the first two eigenvectors of the correlation matrix, with eigenvectors selected by descending eigenvalue. Each eigenvector's sign is fixed by making its largest-magnitude component positive (ties use the lowest original index). Equal leading eigenvalues are rejected because their eigenspace does not define a unique two-axis orientation; missing correlations and non-correlation matrices are also rejected.

## Evolution

A change is backward-compatible only when existing version-1 documents retain the same meaning and remain valid. Changes to matrix interpretation, defaults, required fields, or value ranges require a new `config_version`. Additions to this strict schema require an explicit compatibility decision and schema revision.

## Examples

- `examples/basic.json`: basic full-matrix correlation plot.
- `examples/significance.json`: upper triangle with coefficient and significance annotations.
- `examples/publication-size.json`: CSV-backed, publication-sized figure with explicit typography and physical dimensions.
- `examples/diverging-scale.json`: continuous diverging scale with a neutral midpoint.
- `examples/discrete-scale.json`: explicit symmetric value breaks and interval colors.
- `legacy_palette_scale(colors)` adapts a legacy diverging corrplot color vector to the existing discrete scale contract without changing its interval-to-color mapping.
- `tests/fixtures/general-matrix-cases.json` records reference limits, transforms, and palette families for positive-only, negative-only, mixed-sign, custom-limit, constant, hidden-diagonal, and invalid matrices.

The format is intentionally a small starting point. It does not promise complete coverage of legacy R arguments or define renderer geometry.
