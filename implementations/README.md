# Implementations

The `implementations/` directory currently contains the provisional R package:

- [R implementation](r/README.md) — legacy-compatible plotting delegation, normalized-matrix and file adapters, statistical/palette helpers, and an experimental scene-v1 preparation path.

Other working components live outside this directory:

- [Standalone Rust CLI](../tools/corrplot-ng/README.md) — a native executable with no Python or R runtime dependency.
- [Python scene/SVG prototype](../prototypes/shared-svg/README.md) — a reference implementation and architecture experiment, not a production shared core.
- [Versioned contracts](../prototypes/normalized-matrix-v1/README.md) — language-neutral normalized-matrix input contract; scene-v1 and publication configuration contracts are documented in their respective prototype directories.

Rust is the selected strategic shared-core implementation. The shared library lives in [crates/corrplot-core](../crates/corrplot-core/README.md) and is already used by the standalone CLI and the disposable R/rextendr proof adapter for matrix validation. The R package in this directory has not migrated to that binding. Production rollout remains gated by CRAN guidance and measured maintainer effort; C++/Rcpp is only a possible R-specific fallback. See [ADR 0001](../docs/adr/0001-shared-core-architecture.md) and the [repository layout](../docs/repository-layout.md).
