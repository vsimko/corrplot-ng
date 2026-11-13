# corrplot 0.95 compatibility audit

**Oracle:** upstream `corrplot` 0.95, commit `62532f700e6c9d04b93ffbf0bf56e264b0e0368e`  
**Inventory:** [legacy-api-inventory.json](../r/corrplotng/tests/testthat/fixtures/legacy-api-inventory.json)  
**Purpose:** executable migration checklist for issue #432. This is an audit snapshot, not a claim that corrplot-ng is already drop-in compatible.

## Executive finding

The current R package is **not yet a drop-in replacement** for `corrplot` 0.95. The existing `corrplot_ng()` and `corrplot_ng_mixed()` adapters delegate to upstream for the legacy engine, but their names and exports differ from the upstream API. In particular, the package namespace does not currently export `corrplot`, `corrplot.mixed`, or `corrMatOrder`. A user cannot yet change only `library(corrplot)` to `library(corrplotng)` and keep all original calls unchanged.

This is an API-surface blocker, separate from renderer parity. It should be fixed before any production renderer cutover.

## Pinned public API inventory

The fixture records every export in the pinned upstream `NAMESPACE`, its formal argument names, defaults, and documented choice sets. The public exports are:

| Upstream export | Purpose | Current corrplot-ng surface | Compatibility work |
|---|---|---|---|
| `corrplot` | Primary plotting API | `corrplot` (re-exported upstream fallback) plus `corrplot_ng` | Validate attached-package behavior and preserve fallback until Rust parity |
| `corrplot.mixed` | Upper/lower mixed plot | `corrplot.mixed` (re-exported upstream fallback) plus `corrplot_ng_mixed` | Validate attached-package behavior and preserve invisible result |
| `corrMatOrder` | Matrix ordering | `corrMatOrder` (re-exported upstream fallback) | Validate exact ordering API; later move shared ordering to Rust |
| `cor.mtest` | Pairwise p-values and CIs | `cor.mtest` | Verify formals, errors, missing data, result matrices and all cor.test methods |
| `colorlegend` | Standalone legend drawing | `colorlegend` | Verify full graphics semantics and `...` forwarding |
| `COL1` | Sequential palettes | `COL1` | Verify all palette names, interpolation, edge values and exact colors |
| `COL2` | Diverging palettes | `COL2` | Verify all palette names, interpolation, edge values and exact colors |
| `corrRect` | Cluster rectangles on existing plot | `corrRect` | Verify accepted inputs, coordinates, graphics side effects and return visibility |
| `corrRect.hclust` | Hierarchical-cluster rectangles | `corrRect.hclust` | Verify hclust methods, ordering, graphics and return visibility |

The package must export all nine names exactly. Additional `corrplot_ng_*` names may remain as additive APIs, but they do not substitute for upstream names.

## Primary plotting contract

The exact upstream `corrplot()` formal argument order and defaults are in the pinned JSON inventory. The argument families below are the minimum semantic audit groups; the inventory remains the machine-readable source of truth.

| Area | Upstream arguments / behavior | Required oracle coverage |
|---|---|---|
| Input and scale | `corr`, `is.corr`, `col`, `col.lim`, `transKeepSign` | Matrix and data-frame inputs; correlation and general numeric matrices; asymmetric and rectangular cases where upstream accepts them; custom palette and limits; transforms |
| Glyph and domain | `method`, `type`, `diag`, `outline`, `win.asp`, `addgrid.col` | All seven glyph methods; full/upper/lower; diagonal visibility; aspect constraints; grid/outline behavior |
| Plot lifecycle | `bg`, `title`, `add`, `mar` | Active device; existing plot composition; title; margins/background restoration; no device ownership changes |
| Variable ordering | `order`, `hclust.method`, `addrect`, `rect.col`, `rect.lwd` | Original/AOE/FPC/hclust/alphabet order; every supported clustering method; ties and degenerate matrices |
| Text labels | `tl.pos`, `tl.cex`, `tl.col`, `tl.offset`, `tl.srt` | Independent row/column names; all placement choices; rotation, offset, size, color; plotmath prefixes |
| Color legend | `cl.pos`, `cl.length`, `cl.cex`, `cl.ratio`, `cl.align.text`, `cl.offset` | All positions; automatic and explicit breaks/labels; alignment, ratio, offset |
| Coefficients | `addCoef.col`, `addCoefasPercent`, `number.cex`, `number.font`, `number.digits` | Numeric formatting, percent formatting, font/size, rounding and NA labels |
| Shading | `addshade`, `shade.lwd`, `shade.col` | Negative/positive/all and custom line/color semantics |
| Significance | `p.mat`, `sig.level`, `insig`, `pch`, `pch.col`, `pch.cex` | Every insig mode, vector/multiple significance thresholds where accepted, p-value display and glyph styling |
| Confidence intervals | `plotCI`, `lowCI.mat`, `uppCI.mat` | No/square/circle/rect; bounds, missing and incompatible combinations |
| Missing values | `na.label`, `na.label.col` | Missing cells stay missing; label visibility/color and interaction with triangles/diagonal |

## Observable contracts beyond pixels

Compatibility tests must cover all of the following, not just screenshot similarity:

1. **Namespace:** exact export names, formal argument names and order, documented defaults, accepted `match.arg` choices, and `...` forwarding.
2. **Errors and warnings:** validation order, condition class where relevant, stable message behavior where callers rely on it, and warning visibility.
3. **Return values:** invisible/visible status, list names and shapes, reordered `corr`, `corrPos` columns and row ordering, and optional `p.value` field behavior.
4. **Graphics state:** active device, `add=TRUE`, coordinate system, `par()` changes/restoration, margins, background, and helper overlays.
5. **Data semantics:** data-frame coercion, dimnames, rectangular inputs, NA placement, correlation versus general numeric mode, order and tie handling.
6. **Visual semantics:** cell glyph geometry, palettes and color interpolation, label/plotmath behavior, legends, significance, confidence intervals, cluster rectangles.
7. **Cross-platform tolerance:** compare semantic output strictly; compare raster images with documented device/font/platform tolerances. Never weaken semantic tests to hide rendering differences.

## Rust ownership boundary

The production Rust core must own shared semantics: matrix validation and normalization for the supported contract, ordering algorithms, data-to-visual mappings, scales, layout, glyph geometry, annotations, and renderer-neutral draw operations.

The R adapter may retain only genuinely R-specific behavior:
- R argument matching/coercion and R condition translation;
- statistical helper execution through R's `stats::cor.test()` when preserving exact R semantics is required;
- historical R return objects and data-frame shapes;
- base graphics device ownership, `par()`, and compositional calls such as `add=TRUE`;
- R plotmath parsing/shaping until a portable equivalent is specified.

The adapter must not independently calculate production geometry, scales, ordering, or glyph rules. The pinned upstream package is a temporary fallback and regression oracle, not a permanent second renderer.

## Current evidence and known gaps

Existing tests already compare legacy delegation return objects and graphics-device behavior, exercise all seven upstream glyph methods through the legacy path, and pin the upstream API inventory. The following gaps remain explicit:

- This branch adds exact upstream exports for `corrplot`, `corrplot.mixed`, and `corrMatOrder`; package-level attached-session tests and cross-platform `R CMD check` evidence are still required.
- The modern Rust scene renderer supports only a documented subset; it is not a drop-in implementation of the full legacy API.
- The modern/scene path does not yet cover all methods, ordering, labels, legends, significance, confidence intervals, or mixed plots.
- The inventory is currently strongest on signatures/defaults/choice sets; it must be extended with behavior-level oracle cases for warnings/errors, rectangular inputs, ordering edge cases, palette values, helper return behavior and all annotation modes.
- Existing visual fixtures are useful baselines but are not yet a complete parameter-interaction corpus.

## Required next steps

1. Validate the exact legacy re-exports in installed/attached package workflows and cross-platform `R CMD check`; keep them explicitly classified as temporary upstream fallbacks.
2. Expand the JSON inventory with behavior-level cases and expected outputs, keeping upstream source/ref pinned.
3. Generate oracle fixtures by running upstream 0.95 and corrplot-ng side by side on the same deterministic inputs and devices.
4. Classify each behavior as `rust-core`, `r-adapter`, `temporary-upstream-fallback`, or `deferred-with-explicit-error`.
5. Do not switch the default legacy renderer until all public API and observable-contract gates pass.

Related architecture: [unified visualization model v1](../contracts/visualization-model-v1/README.md), ADR 0004, and issues #430–#435.
