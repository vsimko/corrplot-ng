# ADR 0004: Unified visualization model, legacy compatibility, and modern API

- **Status:** Accepted — target architecture; implementation is tracked by #430 and its linked workstreams
- **Date:** 2026-11-13
- **Decision owners:** corrplot-ng maintainers
- **Related:** [ADR 0001: Shared-core architecture](0001-shared-core-architecture.md), [ADR 0002: Cross-runtime TypeScript architecture](0002-cross-runtime-typescript-architecture.md), [#430: Unified visualization architecture](https://github.com/vsimko/corrplot-ng/issues/430), [#431: General visualization model](https://github.com/vsimko/corrplot-ng/issues/431), [#432: Legacy compatibility corpus](https://github.com/vsimko/corrplot-ng/issues/432), [#433: Rust legacy semantics and rendering](https://github.com/vsimko/corrplot-ng/issues/433), [#434: Modern composable API](https://github.com/vsimko/corrplot-ng/issues/434)

## Context

corrplot-ng has accumulated a useful but incremental set of contracts and implementations: normalized-matrix v1, scene-v1, a Rust shared core, SVG and RGBA renderers, native Node and browser-WASM bindings, TypeScript adapters, and a production R package with an opt-in Rust scene path. The R package's default plotting path still delegates to upstream corrplot. Current scene-v1 and renderer-capability declarations do not cover the complete corrplot 0.95 API or a general visualization engine.

The project needs two distinct public API surfaces without two independent visualization engines: a compatibility surface for existing R corrplot users, and a modern composable API that can evolve beyond the historical corrplot function signature and beyond correlation matrices.

## Decision

### 1. Complete legacy compatibility is a product requirement

Use upstream R `corrplot` 0.95 at commit `62532f700e6c9d04b93ffbf0bf56e264b0e0368e` as the pinned behavioral oracle.

The migration goal is that a user can change only:

```r
library(corrplot)
```

to:

```r
library(corrplotng)
```

and leave existing plotting calls, arguments, and scripts unchanged. The resulting plots must be semantically and visually equivalent or better within documented platform/device tolerances.

Compatibility includes all pinned public exports and observable behavior: formal arguments and defaults, accepted values, ordering, matrix modes, seven glyph methods, labels, palettes, legends, coefficient formatting, significance, confidence intervals, missing values, mixed upper/lower displays, warnings/errors, return values, and relevant R graphics-device composition. R-only statistical and device integration may remain in the R adapter where genuinely host-specific; shared visualization semantics may not.

### 2. Rust is the sole production visualization authority

ADR 0001 remains in force: Rust is the sole selected native implementation. This ADR makes its visualization scope explicit: common semantics, validation, layout, scales, glyph construction, annotations, and production rendering belong in the shared Rust implementation.

R, CLI, Node, WASM, and future runtime integrations are thin adapters. They may convert host-language inputs, manage runtime resources, expose host-specific APIs, and integrate with graphics devices or DOM/Canvas. They must not maintain independent production implementations of common visualization semantics or a parallel R renderer.

### 3. Both public APIs compile into one general model

The legacy R adapter and the modern composable API must compile into the same versioned, renderer-neutral visualization model and use the same Rust core.

The model must generalize beyond correlation matrices. Its design should allow composable layers and glyphs, data sources, scales and visual encodings, labels, legends, annotations, multiple matrices/panels, layout and coordinate systems, accessibility metadata, and future chart/diagram families.

The model describes visualization meaning and renderer-independent geometry. It must not encode SVG markup, Canvas calls, device state, browser objects, or runtime-specific resource ownership.

### 4. Existing contracts evolve explicitly

Normalized-matrix v1 remains the contract for its existing input purpose. Scene-v1 remains valid for existing consumers and fixtures. Do not silently broaden or reinterpret either contract.

Issue #431 will decide whether the general model is a new scene-v2 contract or a distinct visualization-model contract and define explicit conversion/versioning rules. Existing renderer output and capability contracts remain backend-facing contracts; they do not define the full public visualization API.

### 5. The upstream R renderer is a temporary migration fallback and oracle

The current upstream delegation path may remain during migration so unsupported calls continue to work while Rust coverage expands. Fallback use must be observable and documented. The upstream implementation is also retained as a test oracle for semantic and visual regression.

The fallback is not the target architecture. It must not remain a permanent production renderer. Remove the upstream runtime dependency from production plotting only after the complete compatibility and supported-platform gates pass.

### 6. Capability declarations require executable evidence

A renderer capability manifest is a declaration, not proof of implementation. A feature may be advertised only when the relevant implementation and conformance tests exist for that backend. Unsupported features must fail explicitly or follow the documented temporary R fallback; they must never be silently dropped.

## Consequences

- The current scene-v1 matrix slice is a foundation, not the final model.
- The compatibility inventory and regression corpus must become exhaustive before full compatibility can be claimed.
- Rust core work must be driven by the general model and compatibility fixtures, not by ad hoc additions to a narrow scene-v1 structure.
- R-specific function signatures, data-frame/matrix conversion, statistical helper functions, return objects, and graphics-device integration remain adapter responsibilities where appropriate.
- The modern API is independent in syntax and composition, but not in semantics or rendering implementation.
- Python may remain a reference/oracle during migration but is not a production visualization engine.
- SVG, RGBA/raster, and Canvas consume the same model semantics; backend-specific drawing remains separate.
- No release date or compatibility completion date is implied.

## Implementation sequence

1. Complete the compatibility inventory and pinned oracle corpus (#432).
2. Design and validate the general renderer-neutral model (#431).
3. Implement shared Rust semantics and renderers against the model and fixtures (#433).
4. Migrate the legacy R adapter feature by feature, keeping fallback observable until full parity.
5. Design and stabilize the modern composable API over the same model and core (#434).
6. Remove the upstream runtime fallback only after all legacy acceptance criteria and platform checks pass.

The umbrella plan and acceptance gates are tracked in [#430](https://github.com/vsimko/corrplot-ng/issues/430).

## Rejected alternatives

### Keep upstream corrplot as the permanent R renderer

Rejected. This would leave R as a separate production visualization implementation and prevent one shared renderer across runtimes.

### Build a second modern engine beside the legacy adapter

Rejected. Two semantic engines would drift, duplicate bug fixes, and increase maintenance cost.

### Force the modern API to mirror legacy arguments

Rejected. The modern API should be composable and extensible; compatibility belongs in a dedicated legacy adapter.

### Treat scene-v1 as already general enough

Rejected. Its current matrix-bearing subset and implementation status do not establish a general visualization model. Evolve contracts explicitly rather than hiding incompatible concepts in v1.

## Revisit conditions

Revisit only if a demonstrated technical constraint prevents the single Rust implementation from serving a supported runtime. Any exception must be explicit, narrow, and conform to the same versioned model and conformance fixtures. A second independent production renderer is not an acceptable default.
