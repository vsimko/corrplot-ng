# corrplot 0.95 feature matrix

**Machine-readable source:** [legacy-api-feature-matrix.json](../r/corrplotng/tests/testthat/fixtures/legacy-api-feature-matrix.json)  
**Pinned oracle:** [corrplot 0.95](https://github.com/taiyun/corrplot/tree/62532f700e6c9d04b93ffbf0bf56e264b0e0368e)  
**Inventory:** [legacy-api-inventory.json](../r/corrplotng/tests/testthat/fixtures/legacy-api-inventory.json)
**Architecture:** [ADR 0004](adr/0004-unified-visualization-model-and-dual-api.md) · [scene-v1 contract](../contracts/scene-v1/README.md) · [visualization-model-v1](../contracts/visualization-model-v1/README.md) · [Rust core contract](../rust/crates/corrplot-core/README.md)

This matrix is the behavior-level comparison requested by #432. It is intentionally separate from the function/argument inventory: one row represents one independently testable behavior or output layer. It currently contains 152 atomic rows, including one row per exported R signature, each legacy glyph's model/SVG/raster evidence, and the separately testable scene-v1 requirements. The matrix reuses inventory function/argument references and existing fixture IDs instead of copying their definitions. It links ADR 0004 and the relevant scene-v1, visualization-model-v1, and Rust core contracts so contract status is traceable without being mistaken for implementation evidence.

## Columns and evidence semantics

- **Feature / behavior** identifies one observable behavior. Split rows when model construction, SVG output, raster output, or R graphics-device effects have different evidence.
- **Legacy support** and **Rust support** use `verified`, `unsupported`, `unknown`, or `not_applicable`.
  - `verified` requires a checked-in executable test whose identifier is recorded in `evidence`.
  - `unsupported` requires an executable negative test that demonstrates rejection or a stable unsupported diagnostic.
  - `unknown` means the current repository does not contain sufficient executable evidence. It is not equivalent to unsupported.
  - `not_applicable` means the behavior belongs to a host adapter rather than the Rust renderer, with the boundary explained in `notes`.
- **Scene-v1 contract status** records the contract's status only: `supported`, `contract-extension`, `r-adapter`, `deferred`, or `undocumented`. It never establishes implementation support. Scene-v1-specific schema, identity, display-mask, diagnostics, annotation, legend and geometry requirements are listed independently even when no legacy API equivalent exists.
- **Evidence / fixture** links to an exact test identifier and, when applicable, stable IDs from the pinned legacy visual fixture manifest.
- **Disposition** records the next compatibility treatment. It is not a support claim.

## Derived classifications

The validator derives the four requested classes from the two support columns:

| Classification | Legacy | Rust |
|---|---|---|
| `both` | verified | verified + comparative conformance evidence |
| `legacy-only` | verified | unsupported |
| `rust-only` | unsupported | verified |
| `neither` | unsupported | unsupported |

If either side is `unknown` or `not_applicable`, the row remains unclassified. A row with independent evidence on both sides also remains unclassified until a comparative conformance test passes. The JSON stores the derived value for reviewability; the validator rejects a hand-edited value that disagrees with the support columns and comparative test evidence.

**A `both` classification is a parity claim.** It requires independent executable support on both sides and a comparative test that directly asserts the legacy and Rust outputs for that behavior. Independent implementation tests alone do not qualify. No row in this initial matrix currently makes that claim.

## Scene-v1 requirement coverage

The JSON includes a separate `scene_v1_requirement_coverage` map with 32 tracked requirements. Each documented v1 requirement is linked to one or more atomic matrix rows and is marked:

- `verified`: an executable Rust conformance test exists. This includes explicit rejection tests for intentionally unsupported behavior such as plotmath.
- `gap`: the contract requirement is documented, but at least one required Rust behavior/output layer still lacks executable evidence.
- `not-in-contract`: the behavior is explicitly an adapter responsibility or a deliberate v1 limit.

The validator checks that every coverage entry points to existing feature rows and that a `verified` entry has Rust test evidence. Gaps are retained rather than inferred from schema declarations or implementation code.

## Current evidence gaps and decisions

- Rust ordering algorithms remain unverified. All seven glyphs have model/geometry tests, but SVG/RGBA renderer smoke evidence currently exercises only the default circle; the remaining six glyphs are unknown at those output layers. The requirement map also records gaps for hidden-cell output filtering, all-missing matrix state, fallback style colors, unknown glyph/unknown-property diagnostics, number/coefficient precedence, and legend layout/clipping.
- Legacy plotmath is characterized, while the current Rust scene path explicitly rejects it. A portable text-shaping decision remains open.
- Legacy significance modes, coefficient formatting, palette generation, mixed upper/lower composition, and warning/error compatibility need Rust conformance evidence before parity can be claimed.
- SVG and RGBA evidence is recorded per output layer and separately from legacy R graphics-device and PNG snapshot tests. They do not imply a cross-renderer pixel match or a tolerance policy.
- R-only statistical helpers, historical return values, and active graphics-device composition remain adapter responsibilities under ADR 0004.
- Some existing regression tests are executable but inline their inputs rather than consuming a dedicated fixture file. The matrix links those tests and identifies the separate visual fixture IDs where they exist; new corpus work should add durable fixtures when an interaction needs independent regeneration or reuse.

The issue discussion and open follow-up PRs remain authoritative for work not yet merged to `main`. In particular, a passing CI run on an unmerged PR is not treated as evidence in this matrix until its test is present on the target branch.

## Maintenance and validation

1. Keep the pinned package version and commit identical to the JSON inventory.
2. Add one row per independently testable behavior/layer; do not use one broad row to hide different renderer or device outcomes.
3. Reuse existing inventory references and visual fixture IDs. Add a fixture only when a stable input/output case adds value beyond an inline test.
4. Link evidence to an exact checked-in test identifier. A test file or capability manifest alone is not sufficient.
5. Use `unknown` until executable evidence exists. Do not promote a row based on a declared capability or implementation code inspection alone.
6. Keep `parity_status: not-claimed` until a comparative conformance test passes. Record its exact test under `parity_evidence` before changing it to `verified`.
7. Run:

```sh
python tools/repository/validate_legacy_feature_matrix.py
python -m unittest tools.repository.tests.test_legacy_feature_matrix -v
```

The repository foundation workflow also discovers `test_*.py` under `tools/repository/tests`.
