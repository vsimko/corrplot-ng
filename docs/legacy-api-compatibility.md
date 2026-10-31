# Legacy corrplot API compatibility baseline

**Reference:** `corrplot` 0.95, commit `62532f700e6c9d04b93ffbf0bf56e264b0e0368e`  
**Upstream:** [taiyun/corrplot](https://github.com/taiyun/corrplot/tree/62532f700e6c9d04b93ffbf0bf56e264b0e0368e)  
**Machine-readable inventory:** [legacy-api-inventory.json](../implementations/r/tests/testthat/fixtures/legacy-api-inventory.json)

This is a compatibility baseline, not a promise that every legacy option belongs in the cross-language API. The pinned upstream `NAMESPACE` exports nine functions; the inventory records each function's formal arguments, defaults, and documented choice sets. The inventory is deliberately versioned separately from the renderer-neutral scene contract.

## Exported API

| Export | Purpose | Migration classification |
|---|---|---|
| `corrplot()` | Main matrix visualization | Mixed: shared semantics plus R graphics adapter |
| `corrplot.mixed()` | Different methods in upper/lower triangles | R adapter convenience wrapper; shared upper/lower semantics |
| `corrMatOrder()` | AOE, FPC, hierarchical, alphabetical ordering | Contract extension for explicit variable permutation |
| `cor.mtest()` | Pairwise p-values and confidence intervals | R/statistics adapter; not renderer functionality |
| `colorlegend()` | Standalone color legend drawing | R-adapter drawing helper; shared legend semantics need specification |
| `COL1()`, `COL2()` | Sequential and diverging palettes | Contract extension for portable palette definitions |
| `corrRect()` | Add rectangles using indices or names | R graphics adapter; annotation semantics could be shared |
| `corrRect.hclust()` | Add rectangles from hierarchical clusters | Split: clustering/permutation semantics vs R drawing helper |

The provisional package exposes `corrplot_ng()` and `corrplot_ng_mixed()` as thin legacy wrappers for the two plotting entry points. Their R-specific return objects are specified in [legacy R return values](legacy-r-return-values.md). The complete formal signatures, defaults, and accepted values are in the JSON inventory. In particular, `corrplot()` has a broad surface: seven glyph methods, three matrix display types, five ordering choices, label and legend controls, coefficient annotations, significance handling, confidence intervals, missing-value labels, and general-matrix support.

## Behavioral areas and compatibility status

The following classifications describe where behavior belongs architecturally. They do not claim that the current scene-v1 prototype already implements it.

| Legacy behavior | Expected compatibility semantics | Scene-v1 status |
|---|---|---|
| Input matrix and values | Square or rectangular matrices; preserve missing values; correlation mode validates values in [-1, 1] | **contract-extension** — v1 example supports only a small square correlation matrix |
| Seven plot methods | circle, square, ellipse, number, shade, color, pie | **contract-extension** — glyph semantics are not represented in v1 |
| Triangle and diagonal | full, upper, lower; independent diagonal visibility | **contract-extension** |
| Variable ordering | original, AOE, FPC, hclust with selected linkage, alphabet | **contract-extension** — v1 preserves input order only |
| Labels | row/column names, plotmath prefixes, placement, rotation, size and color | **contract-extension** for shared text semantics; **r-adapter** for R plotmath/device details |
| Color scales | sequential/diverging palettes, explicit colors and limits, legend | **contract-extension** |
| Coefficient annotations | numeric labels, percentage formatting, digits, font and size | **contract-extension** |
| Significance | p-value matrix, scalar/vector thresholds, pch, p-values, blanking, label_sig | **contract-extension** |
| Confidence intervals | lower/upper matrices and n/square/circle/rect display modes | **contract-extension** |
| Missing values | NA stays distinct from zero; configurable label and color | **supported** only for null cell values in the v1 data model; display styling is **contract-extension** |
| General matrices | `is.corr=FALSE`, `col.lim`, sign-preserving transforms | **contract-extension** |
| Cluster rectangles | number of clusters and custom/name/index-defined rectangles | **r-adapter** for drawing; clustering metadata may be a **contract-extension** |
| Mixed upper/lower methods | two coordinated plots with independent glyph methods and shared ordering | **r-adapter** wrapper; shared composition needs a **contract-extension** |
| Graphics-device integration | `add=TRUE`, active device, `par()`, follow-up `text()`/`segments()`/rectangle drawing | **r-adapter** |
| Return values | `corrplot()` invisibly returns reordered matrix, cell positions and argument metadata; helpers consume this structure | **r-adapter** initially; do not promise cross-language object identity |

### Public behavior versus implementation detail

Treat as compatibility-relevant:

- Exported names and formal argument names, documented defaults, accepted choices, and documented return values.
- Observable matrix ordering, cell selection, diagonal behavior, missing-value handling, labels, legends, annotations, and confidence/significance semantics.
- Graphics-device behavior that existing scripts can observe, including overlays and helper functions that draw after `corrplot()`.
- The ability to use the returned `corrplot()` result with `corrRect()`.

Do not freeze as cross-language API:

- Private `draw_*` functions and other unexported helpers.
- Internal temporary variables, exact intermediate data-frame construction, and undocumented implementation ordering.
- Pixel-identical raster output across operating systems or graphics devices. Font availability, graphics engines, antialiasing, and device settings can change pixels independently of plot meaning.

## Fixture strategy

The companion visual manifest pins upstream version 0.95 and the reference commit. Its 15 named cases cover:

1. Default circle and each of the other six methods.
2. AOE ordering and upper/lower triangle behavior.
3. Coefficient labels and mixed circle/ellipse output.
4. Missing values and plotmath labels.
5. Significance marks, confidence intervals, and non-correlation matrices.

The PNG snapshots are a visual oracle, not the only compatibility test. The executable cases and manifest must stay in sync. Semantic/API tests separately check exported symbols and formal argument names so a raster match cannot hide an API regression. Visual comparisons run with identical device settings; platform/font-dependent differences must be diagnosed separately rather than silently weakening semantic assertions.

Regenerate reference images only from the pinned upstream package, using the repository script `scripts/update-legacy-visual-snapshots.R`. Do not regenerate expected images from `corrplotng` itself.

## Relationship to scene contract v1 and adapter work

The current scene-v1 contract is intentionally smaller than the legacy API. It preserves a matrix's input order, labels, missing-value distinction, and basic semantic values. It does not yet specify glyph methods, ordering algorithms, typography, significance, confidence intervals, legends, arbitrary annotations, or R graphics-device behavior.

Use this baseline as input to [issue #50](https://github.com/vsimko/corrplot-ng/issues/50), the legacy R adapter design. Implement shared capabilities only after the contract defines their semantics. Keep R-specific behavior in the adapter or fallback; never silently drop an option that the scene cannot represent. This issue inventories and tests the baseline; it does not implement the full compatibility layer.

## Reproduction and verification

From the repository root, install the pinned upstream reference package and the test dependencies, then run:

```sh
R -e 'remotes::install_github("taiyun/corrplot@62532f700e6c9d04b93ffbf0bf56e264b0e0368e")'
R -e 'install.packages(c("testthat", "jsonlite", "png"))'
R CMD check implementations/r
```

The manifest check fails if the installed reference version differs from 0.95 or the fixture IDs drift from the executable case list. The API inventory test compares every exported function's formal argument names against the pinned inventory. The visual test checks candidate output against committed upstream snapshots under a fixed Cairo PNG configuration.
