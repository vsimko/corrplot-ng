---
type: Design Specification
title: Interactive correlation-matrix exploration v1
description: Interaction model, matrix-only versus raw-data workflows, and a minimal TypeScript controller API for interactive exploration.
tags:
  - interaction
  - typescript
  - scene-v1
  - design
status: proposed
---

# Interactive correlation-matrix exploration v1

## Purpose

Define a small interactive experience that complements deterministic publication
rendering. The interactive view inspects and navigates a matrix. It does not
replace the source matrix, publication configuration, or shared Rust semantics.

This is a design specification for issue #51. It does not implement a browser
application or commit the project to a UI framework.

## Design principles

1. **One semantic source.** Matrix validation, ordering, scene construction,
   and rendering semantics remain in the shared Rust core. TypeScript owns
   interaction state and browser integration only.
2. **Read-only by default.** Exploration changes view state. It never edits
   matrix values or writes changes back to a publication configuration.
3. **Explicit data boundaries.** A correlation matrix supports cell inspection.
   Observation-level analysis requires raw observations or a caller-provided
   data source.
4. **Static output remains deterministic.** The same matrix and render options
   can still produce a reproducible SVG or raster artifact.
5. **Accessible interaction.** Every pointer action has a keyboard equivalent.
   Hover must not be the only way to discover a value or action.
6. **Capability-aware rendering.** A backend must declare supported scene
   features. The UI must not silently omit unsupported semantics.

## Primary workflows

### 1. Inspect a matrix

- Show row and column labels and the selected cell encoding.
- Hovering or focusing a cell displays its row label, column label, exact
  numeric value, sign, and any supplied per-cell metadata.
- Clicking or pressing Enter pins the cell in a details panel.
- Missing cells are identified as missing; they are not presented as zero.
- A clear-selection action returns to the overview.

### 2. Change the view

- Switch between full, upper-triangle, and lower-triangle display.
- Change the supported ordering without changing source data.
- Search row or column labels. Search highlights and navigates to matches; it
  does not delete or rewrite matrix rows.
- Reset all view state to the initial options.

### 3. Navigate a large matrix

- Keep row and column headers visible while the matrix viewport scrolls.
- Provide zoom controls and a fit-to-view action.
- Keep selection attached to the same row/column identities when ordering
  changes. It must not remain attached to an old screen coordinate.
- Provide keyboard navigation and a visible focus indicator.
- Use viewport virtualization for large matrices; do not create a DOM node for
  every cell when that would make the page unusable.

### 4. Export a publication figure

- Export uses the existing engine render operation and renderer output
  contract.
- Export options are explicit and separate from interactive state.
- The initial MVP can export SVG. Raster and direct Canvas output are used only
  when the selected backend declares those capabilities.

## What a matrix alone can support

The following information can be shown without raw observations:

- row and column labels;
- the exact matrix value, sign, and absolute magnitude;
- whether a cell is missing;
- a selected row/column pair and its position in the current ordering;
- ordering, display mode, and viewport changes;
- metadata explicitly supplied with the matrix, such as sample size,
  p-value, or confidence interval.

Supplied metadata must be labelled as metadata. The UI must not imply that it
was calculated from the displayed correlation value.

## What requires raw observations

The following actions are unavailable from a correlation matrix alone:

- scatterplots of the paired observations;
- point-level inspection, outlier identification, or distribution plots;
- recalculation of a correlation after excluding observations;
- sample counts, confidence intervals, or p-values that were not supplied as
  metadata;
- observation-level filters and regression diagnostics.

If the caller provides raw observations, expose them through an explicit
provider interface. Selecting a cell may request its paired observations from
that provider. The controller must not invent observations or silently infer
them from the matrix.

## Minimal TypeScript-facing API

The API extends the asynchronous engine boundary in
[ADR 0002](adr/0002-cross-runtime-typescript-architecture.md). It is a proposed
controller API, not a new rendering backend.

~~~ts
type ExplorerOrdering = "original" | "alphabetical" | "aoe" | "hclust";
type ExplorerDisplay = "full" | "upper" | "lower";

interface ExplorerOptions {
  matrix: NormalizedMatrixInput;
  initialView?: {
    ordering?: ExplorerOrdering;
    display?: ExplorerDisplay;
  };
  cellMetadata?: CellMetadataProvider;
  observations?: ObservationProvider;
}

interface CellDetails {
  rowIndex: number;
  columnIndex: number;
  rowLabel: string;
  columnLabel: string;
  value: number | null;
  metadata?: CellMetadata;
}

interface ExplorerController {
  getState(): Readonly<ExplorerState>;
  getScene(): Promise<SceneV1>;
  setOrdering(ordering: ExplorerOrdering): Promise<void>;
  setDisplay(display: ExplorerDisplay): Promise<void>;
  setLabelQuery(query: string): void;
  selectCell(rowIndex: number, columnIndex: number): void;
  clearSelection(): void;
  getCellDetails(rowIndex: number, columnIndex: number): CellDetails;
  resetView(): Promise<void>;
  subscribe(listener: (event: ExplorerEvent) => void): () => void;
}
~~~

The intended creation flow is:

~~~ts
const engine = await createCorrplotEngine({ backend: "auto" });
const explorer = engine.createExplorer({
  matrix,
  initialView: { ordering: "original", display: "full" },
});

const unsubscribe = explorer.subscribe((event) => {
  if (event.type === "cell-selected") {
    showCellDetails(event.cell);
  }
});

await explorer.setOrdering("alphabetical");
const scene = await explorer.getScene();
const svg = await engine.render(scene, { format: "svg" });
~~~

The exact method names and type names require review before implementation.
The important boundary is that the controller owns view state and delegates
scene construction and rendering to the engine. It does not calculate
correlations, reorder matrix values independently, or implement a second
renderer.

### State and events

ExplorerState contains only transient view state:

- ordering and display mode;
- selected row/column identity, if any;
- label-search query;
- viewport zoom and scroll position.

The source matrix and publication configuration are immutable inputs. A
selection is represented by stable row/column indices or labels, not by
display coordinates, so reordering does not change which pair is selected.

The initial event set is:

- cell-selected: selected cell details, including optional supplied metadata;
- selection-cleared: no cell is pinned;
- view-changed: ordering, display, search, or viewport state changed.

The controller returns an unsubscribe function. It does not own a DOM node,
Canvas context, worker, or framework component.

### Optional data providers

A CellMetadataProvider may return values already calculated by the caller,
such as sample size, p-value, or confidence interval. Missing metadata remains
absent.

An ObservationProvider may asynchronously return paired observations for a
selected row/column pair. This is an opt-in integration. The core controller
must work when no observation provider is present.

## Accessibility and input behavior

- Use a semantic grid or an equivalent accessible representation alongside
  Canvas rendering.
- Support arrow keys to move the active cell, Home/End to move within a row,
  and Page Up/Page Down to move by a viewport.
- Enter or Space pins the active cell. Escape clears the selection.
- Keep focus visible after scrolling, zooming, or reordering.
- Announce the selected labels and exact value to assistive technology.
- Do not encode sign or magnitude by color alone; provide a textual value and
  missing-value description.
- Provide buttons or keyboard commands for zoom, fit, ordering, display, and
  reset.

The accessibility representation may be virtualized, but it must preserve
logical row/column coordinates and selected-cell semantics.

## Prioritized MVP

### MVP

1. Accept normalized matrix input and create the initial scene through the
   engine.
2. Show labels, values, missing cells, and the current view.
3. Support cell hover/focus, pinned selection, and an exact-value details
   panel.
4. Support arrow-key navigation, full/upper/lower display, original and
   alphabetical ordering, label search, and reset.
5. Keep selected cell identity stable across ordering changes.
6. Export SVG through the existing renderer contract.
7. Test matrix-only behavior without any raw observation provider.

### Follow-up

- AOE and hierarchical-clustering ordering when exposed by the shared core.
- Direct Canvas and OffscreenCanvas rendering through the browser adapter.
- Zoom and viewport virtualization tuned for very large matrices.
- Optional observation-provider integration and scatterplot views.
- Supplied per-cell statistical metadata and richer details panels.
- Persisted/shareable view state after a separate versioned state contract is
  agreed.

## Explicit non-goals

- A full dashboard, notebook, or collaborative application.
- Editing matrix values or mutating publication configuration.
- Computing correlation, significance, or confidence intervals in TypeScript.
- Scatterplots without raw observations.
- A second JavaScript implementation of ordering, scene construction, or
  rendering.
- Requiring Canvas, WebGL, or WebGPU for the initial MVP.
- Binding the API to React, Vue, Svelte, or another UI framework.

## Dependencies and implementation order

- Use normalized-matrix v1 and scene-v1 as the existing language-neutral
  contracts.
- Follow the backend and engine boundary in ADR 0002.
- Use renderer capability declarations before enabling optional visual
  features.
- Implement the controller only after the native/WASM engine operations needed
  for validation and scene construction are available.
- Direct browser Canvas rendering remains in #144. Native Node bindings and
  browser WASM remain separate backend work.

Issue #51 is satisfied by this specification. Implementation of the MVP should
be planned separately after its backend dependencies are available.
