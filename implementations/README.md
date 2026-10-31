# Implementations

The `implementations/` directory currently contains the provisional R package:

- [R implementation](r/README.md) — legacy-compatible plotting delegation, normalized-matrix and file adapters, statistical/palette helpers, and an experimental scene-v1 preparation path.

Other working components live outside this directory:

- [Standalone Rust CLI](../tools/corrplot-ng/README.md) — a native executable with no Python or R runtime dependency.
- [Python scene/SVG prototype](../prototypes/shared-svg/README.md) — a reference implementation and architecture experiment, not a production shared core.
- [Versioned contracts](../prototypes/normalized-matrix-v1/README.md) — language-neutral normalized-matrix input contract; scene-v1 and publication configuration contracts are documented in their respective prototype directories.

Rust is the selected shared-core implementation under [ADR 0001](../docs/adr/0001-shared-core-architecture.md). The reusable library lives in [crates/corrplot-core](../crates/corrplot-core/README.md); the standalone CLI and disposable R/rextendr adapter both call it for matrix validation. The rextendr adapter remains an experiment, not a production R backend, and the provisional R package has not migrated to Rust. C++/Rcpp remains an isolated R-only comparison spike.

See the [repository structure and ownership map](../docs/repository-structure.md) for the role of this directory and package-local fixture copies.
