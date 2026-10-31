# Normalized matrix input v1

This directory defines the input boundary between source adapters and scene
construction. It is deliberately independent of R, Rust, Python, CSV parsing,
and any renderer.

- Schema: `schema.json`
- Valid examples and shared fixtures: `fixtures/valid-cases.json`
- Invalid adapter cases: `fixtures/invalid-cases.json`
- Python reference semantic validator: `validate.py`

## Contract

A normalized input is an already-computed **square numeric matrix**. This
contract does not calculate correlations from raw observations.

- `labels` contains one stable display label per row and column. Labels are
  preserved exactly, including Unicode; they are not trimmed or normalized.
- Labels must be non-empty and unique using exact, case-sensitive comparison.
- If both row and column names are supplied by an adapter, they must match
  exactly in the same order. If only one axis has names, that set is used for
  both axes. If neither axis has names, the adapter generates `V1` through
  `Vn`. Partially missing/empty names are rejected rather than mixed with
  generated names.
- `values` is an N×N array. Finite JSON numbers are numeric values. JSON
  `null` is the only missing-value representation; numeric zero remains zero.
- R `NA` maps to `null`. R `NaN`, positive/negative infinity, and non-numeric
  cells are rejected by adapters; they are not silently converted to missing.
- `source_indices` is a zero-based mapping back to the source row/column order.
  Input adapters initially emit `[0, 1, ..., n-1]`; later ordering belongs to
  scene construction and must preserve this mapping.
- `source` records the input format and whether row/column labels were present.
  Optional `path` is provenance only; it is not interpreted by the renderer.
- `metadata` is optional JSON-compatible provenance. It must not contain R
  objects, pointers, or renderer-specific attributes.

## Degenerate input behavior

- Non-square or empty matrices are rejected at the input boundary.
- Duplicate, empty, or mismatched row/column labels are rejected.
- All-missing square matrices are valid normalized inputs: normalization must
  preserve their dimensions and missingness. A later scene/color-scale stage
  may reject them as non-renderable because there are no numeric values.
- Constant and all-zero matrices are valid normalized inputs.
- A missing cell and a numeric zero are always distinguishable.

The JSON Schema checks shape and primitive types. The semantic validator adds
cross-field checks that JSON Schema cannot express portably, such as square
dimensions and matching label/source-index lengths.

## Example

```json
{
  "schema_version": 1,
  "kind": "normalized_matrix",
  "labels": ["Δ temperature", "湿度"],
  "values": [[1.0, null], [null, 1.0]],
  "source": {
    "format": "r_matrix",
    "row_labels_present": true,
    "column_labels_present": true
  },
  "source_indices": [0, 1],
  "metadata": {"example": "Unicode labels and explicit missing values"}
}
```

## Validation

From the repository root:

```sh
python -m unittest discover -s prototypes/normalized-matrix-v1 -p 'test_*.py' -v
```

The R and file adapters in the follow-up issues #131 and #132 must produce
objects conforming to this schema and the same semantic fixtures.
