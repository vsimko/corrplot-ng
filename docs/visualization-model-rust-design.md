# Rust type design for visualization-model-v1

This document translates the JSON contract in [visualization-model-v1](../contracts/visualization-model-v1/README.md) into Rust ownership boundaries and validation stages. It is a design contract, not a claim that these types are implemented already.

## Module boundary

Proposed module: `rust/crates/corrplot-core/src/visualization/`

- `model.rs`: public semantic document types and stable IDs.
- `validate.rs`: structural and cross-reference validation with stable diagnostic codes.
- `data.rs`: typed matrix/table access and row/column identity.
- `scale.rs`: domain validation and deterministic data-to-visual mapping.
- `layout.rs`: plot/panel allocation and coordinate systems in logical units.
- `compile.rs`: semantic model to renderer-neutral display/geometry model.
- `diagnostics.rs`: path-aware errors and warnings.

The first implementation may use fewer files, but these responsibilities must remain separate. Existing `scene` and `output` modules remain valid scene-v1 and renderer-output-v1 APIs; do not rename or silently repurpose their public types.

## Core Rust types

Illustrative shape (names may change before implementation, semantics may not):

```rust
struct VisualizationDocument {
    schema_version: u32,
    kind: DocumentKind,
    metadata: DocumentMetadata,
    layout: DocumentLayout,
    data_sources: Vec<DataSource>,
    scales: Vec<Scale>,
    plots: Vec<Plot>,
}

enum DataSource {
    Matrix(MatrixSource),
    Table(TableSource),
}

struct MatrixSource {
    id: Id,
    row_labels: Vec<String>,
    column_labels: Vec<String>,
    values: Vec<Vec<Option<f64>>>,
    semantics: MatrixSemantics,
}

enum MatrixSemantics { Correlation, GeneralNumeric }

struct Plot {
    id: Id,
    coordinates: CoordinateSystem,
    layers: Vec<Layer>, // vector order is paint order
    panels: Vec<Panel>,
    legends: Vec<Legend>,
    annotations: Vec<Annotation>,
    axis_labels: AxisLabels,
    accessibility: Accessibility,
}

struct Layer {
    id: Id,
    kind: MarkKind,
    data: DataSourceId,
    encoding: Encoding,
    display: DisplayOptions,
    style: Style,
}

enum MarkKind { Matrix, Bar, Line, Point, Rect, Text }
```

Use typed newtypes for IDs and indices. Use enums for closed semantic vocabularies. Keep arbitrary style maps out of core semantics where possible; style properties that affect geometry or accessibility must be typed fields. JSON parsing must reject unknown semantic fields rather than silently ignore them.

## Data semantics

- Matrix data has independent row and column labels; square correlation matrices are a constrained subset, not the universal matrix shape.
- Values are row-major and missing values remain explicit `None`; never coerce NA to zero.
- Correlation validation is conditional on `semantics = correlation`. General numeric matrices may be rectangular and need not be symmetric or bounded to [-1, 1].
- Table columns have a declared scalar type and equal row count. Category order is data order unless an explicit scale domain overrides it.
- Data source IDs are unique within a document. A layer's data reference must resolve to exactly one source.

## Scale and encoding semantics

- A scale ID has exactly one definition. Every channel reference must resolve and be compatible with its scale kind.
- Numeric domains must be finite and ordered; log domains must be strictly positive. Band/point domains must contain unique category values.
- A color scale's range must contain valid color tokens and enough anchors for its interpolation rule.
- A field channel must exist on the referenced source and be compatible with the target mark/channel.
- A constant channel and a field channel are mutually exclusive.
- Missing-value policy is explicit: preserve, omit, or error. A renderer must not silently choose another policy.
- Matrix glyph size, ellipse orientation, pie angle, and number formatting are semantic encodings; their geometry is derived by Rust.

## Coordinate and layout semantics

- All model dimensions and coordinates are logical units, not device pixels.
- Cartesian, polar, and matrix coordinate systems are distinct semantic types. Do not encode SVG viewBox or Canvas backing-store dimensions in this model.
- Panel rectangles must be positive and lie within the document layout. Panel layer IDs must exist and must not be assigned ambiguously.
- Shared scales are shared only by identical scale IDs. Separate IDs mean independent domains even if their definitions happen to match.
- Layout order is deterministic. Backends may apply device scaling and font metrics but cannot change data ordering or semantic placement.

## Labels, annotations and accessibility

- Row and column labels are independent. Each may specify visibility, side, rotation, offset, font attributes, color and text-vs-plotmath format.
- Plotmath source is semantic input, not an SVG fragment. It must be parsed/shaped by a defined text adapter or explicitly rejected; it must never be inserted as untrusted markup.
- Annotations use typed anchors: plot, data identity, or logical coordinate. An unresolved data anchor is a validation error.
- Legends reference existing scales and have stable IDs.
- Accessibility summaries are required for public documents. Long descriptions and data-table exposure are optional but explicit.

## Validation pipeline

Validation must run before compilation and return deterministic, path-aware diagnostics:

1. Parse JSON and reject duplicate object keys, unknown fields and unsupported schema versions.
2. Validate required fields, enum values, finite numbers, IDs and local object invariants.
3. Build ID indexes and validate uniqueness and every cross-reference.
4. Validate data shapes: matrix row/column counts and labels; equal table-column lengths; valid scalar values.
5. Validate scales, field existence, channel/mark compatibility and missing-value policies.
6. Validate plot/panel membership, coordinate domains, layout bounds, annotations and legend references.
7. Compile only a fully valid semantic document. No renderer may reinterpret invalid or unsupported semantics.

Diagnostic codes should be stable machine-readable identifiers (for example `model.duplicate_id`, `model.reference.not_found`, `model.matrix.shape_mismatch`, `model.scale.incompatible_channel`). Messages may improve; codes and JSON paths are the programmatic contract.

## Relationship to existing contracts

- `normalized-matrix-v1` remains a focused matrix input contract.
- `scene-v1` remains a prepared, geometry-bearing matrix scene contract.
- `visualization-model-v1` is a higher-level semantic input document. It is not an alias for scene-v1.
- Conversion to scene-v1 is lossless only for the subset whose semantics scene-v1 can represent. Otherwise the compiler must target the new renderer-neutral geometry/display-list layer or return an explicit unsupported-feature diagnostic.
- Renderer-output-v1 remains an output contract for raster buffers; it is not a visualization document.
- Capability manifests describe executable backend support and must be generated/verified against implementation and conformance tests, not inferred from this schema.

## Implementation sequence

1. Freeze schema and fixtures only after schema validation and review.
2. Implement typed parsing and semantic validation in Rust with positive and negative fixtures.
3. Implement one deterministic matrix-layer compiler and prove parity with existing scene-v1 for the overlap subset.
4. Add general matrix and non-matrix marks incrementally with conformance fixtures.
5. Make both R legacy adapter and modern API compile to this model; keep host-only behavior outside it.
