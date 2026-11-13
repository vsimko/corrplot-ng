# Visualization model v1

**Status:** initial canonical contract for the unified legacy and modern API pipeline  
**Schema:** [schema.json](schema.json)  
**Fixtures:** [visualization-model-v1 fixtures](../../fixtures/visualization-model-v1/README.md)

## Purpose

Visualization model v1 is the renderer-neutral semantic document produced by both public API surfaces:

1. The legacy R adapter translates pinned `corrplot` 0.95 calls into this model.
2. The modern composable API builds the same model directly.

The Rust core is the authority for validation, data-to-visual encoding, coordinate/layout semantics, geometry, and rendering. Host adapters own host-language conversion and lifecycle only. The model is not an SVG DOM, Canvas command stream, R graphics object, or pixel buffer.

## Contract boundary

The document separates:

- **Data:** inline matrix or tabular values, independent row/column labels (or a shared `labels` shorthand for square matrices), and stable source identity.
- **Semantics:** plot type, layers, marks, visual encodings, annotations, and accessibility descriptions.
- **Layout:** logical dimensions, plot/panel arrangement, and coordinate systems.
- **Scales:** data domains and semantic visual ranges.
- **Rendering:** intentionally absent. A renderer derives paths, glyph geometry, clipping, and output operations from the model.

Coordinates use logical document units. They are not device pixels. Backends may map them to pixels while preserving aspect ratio and applying host-specific device scaling. Font metrics and antialiasing may vary, but data, ordering, encoding, layer composition, and annotation meaning may not.

## Core objects

- A document has versioned data sources, scales, plots, and metadata. Matrix sources may be rectangular; row and column labels are independent.
- A plot has one coordinate system, ordered layers, optional panels, legends, annotations, and accessibility metadata.
- A layer references data by stable ID and expresses visual mappings declaratively.
- Layer order is paint order: later layers are composited above earlier layers.
- A panel is a plot-local composition region; multiple panels do not imply shared scales unless they reference the same scale ID.
- An annotation is semantic content anchored to data, a coordinate, or a plot region. It is not a renderer-specific primitive.

The first schema deliberately supports a focused set of common data and mark forms. Matrix shape validation and cross-reference validation are required in addition to JSON Schema. The current Python conformance test exercises these invariants; the Rust core must implement the same checks before the model is adopted by production adapters. New semantic capabilities require an explicit compatible schema evolution decision; renderer-specific fields do not belong here.

## Mapping legacy corrplot

The legacy `corrplot()` adapter maps:
- input matrix and independent row/column labels to a matrix data source;
- `type`, `diag`, and ordering to selected data indices / layer visibility;
- `method` to the matrix layer's glyph encoding;
- `col`, `col.lim`, and palette helpers to a shared color scale;
- coefficient labels, significance, confidence intervals, and cluster rectangles to semantic overlay layers or annotations;
- title and descriptive text to plot metadata/accessibility.

R-only behavior remains outside the model: R argument matching and coercion, `cor.mtest()` execution through R's statistical APIs, historical return objects, active graphics-device ownership, `par()`, and follow-up base-graphics operations. The adapter must preserve these observable contracts without becoming a second renderer.

## Mapping modern compositional API

The modern API creates plots and ordered layers directly. A matrix layer is one possible layer, not the document's top-level type. The same document can contain a bar, point, line, text, matrix, or annotation layer, with scales and panels shared explicitly by IDs.

## Versioning and migration

- `schema_version: 1` is immutable once released to consumers.
- Unknown schema versions and unknown fields are errors; no best-effort reinterpretation.
- Additive semantics that change accepted document shape or interpretation require a new major contract directory and version.
- Scene-v1 remains unchanged and valid. It is a prepared matrix-scene interchange with renderer-neutral geometry already present; visualization-model-v1 is a higher-level semantic input model. There is no implicit equivalence between their JSON payloads.
- Migration from scene-v1 is an explicit adapter: reconstruct source data and semantic mappings only where the scene retains enough information. Geometry-only details that cannot be recovered remain geometry, not invented semantics.
- A renderer capability claim is valid only with executable implementation and conformance evidence. Unsupported required features fail with stable diagnostics; they are never silently dropped.

## Validation

The JSON Schema validates document shape and basic types. Rust validation must additionally check cross-references, unique IDs, matrix dimensions, finite values, scale compatibility, mapping field existence, panel membership, and mark-specific constraints. Canonical fixtures are validated by schema and Rust tests. The fixtures are synthetic contract examples, not upstream visual oracles.
