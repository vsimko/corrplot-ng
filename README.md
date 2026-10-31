# corrplot-ng

**An experimental toolkit for correlation-matrix visualization and compatibility across runtimes.**

corrplot-ng explores a modern, renderer-neutral approach to correlation plots while preserving established R workflows. It is an early-stage project, not a production-ready replacement for the original R package.

## Current status

The repository contains several working components with different maturity levels:

- **R package:** a provisional adapter. Its default `corrplot_ng()` path delegates to upstream `corrplot`; the experimental `engine = "scene"` path prepares renderer-neutral scene-v1 data. The package also provides normalized-matrix and file adapters, `cor.mtest()`, and palette/color-legend helpers.
- **Normalized-matrix v1:** a language-neutral contract for already-computed square matrices, including labels, missing values, and source-index mappings. R matrix and CSV/TSV/JSON file adapters produce this representation.
- **Python scene/SVG prototype:** a reference implementation that builds a renderer-neutral scene and exports deterministic SVG. It demonstrates the seven prototype glyph methods, but it is not a production shared core.
- **Rust standalone CLI:** a compiled native executable that validates and renders supported publication configurations and matrix inputs to SVG. It has no Python or R runtime dependency and is tested on Linux, macOS, and Windows.
- **Scene-v1 and publication-config v1:** versioned interchange/configuration contracts used by the current prototypes and CLI.

Rust is selected for the standalone CLI only. The production shared-core language remains deferred under [ADR 0001](docs/adr/0001-shared-core-architecture.md). Rust/rextendr and C++/Rcpp integrations are evaluation spikes, not production R backends. TypeScript, Node native bindings, browser WASM, Canvas, and additional renderers remain future work.

## Architecture

The project separates language-specific input adapters, versioned language-neutral contracts, scene construction, and output renderers. The contracts allow implementations to exchange data without requiring one shared compiled library.

See [the current architecture](docs/architecture.md), [ADR 0001](docs/adr/0001-shared-core-architecture.md), [top-level requirements](docs/requirements.md), and the [future vision (draft)](docs/future-vision.md).

## Compatibility scope

The original [R corrplot package](https://github.com/taiyun/corrplot) remains the reference for legacy behavior. The current R adapter preserves the legacy plotting path by delegation and has an experimental scene-preparation path. Automated tests cover selected legacy cases, normalized-input behavior, scene/SVG semantics, and saved raster references. These tests are representative, not exhaustive; full behavioral compatibility is not claimed.

The Python scene/SVG prototype supports seven publication glyph methods. The current native CLI supports a smaller documented subset, including circle and number glyphs. Do not assume feature parity between the prototype, R adapter, and CLI.

## Local development

### Prerequisites

- Git
- Python 3.10 or later
- R 4.x
- Rust stable toolchain (for the native CLI)

Install the R packages used by the package and compatibility tests:

```sh
R -q -e 'install.packages(c("corrplot", "testthat", "jsonlite", "png"), repos = "https://cloud.r-project.org")'
```

Install the Python schema validator used by the prototype tests:

```sh
python -m pip install jsonschema
```

### Run tests

From the repository root:

```sh
# Shared fixture integrity
python -m unittest discover -s tests -v

# Normalized-matrix v1 schema and semantic tests
python -m unittest discover -s prototypes/normalized-matrix-v1 -p 'test_*.py' -v

# Python scene/SVG prototype
python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v

# R package, including adapter and legacy regression tests
R CMD check implementations/r

# Native Rust CLI tests and release build (offline, locked dependencies)
cargo test --locked --offline --manifest-path tools/corrplot-ng/Cargo.toml
cargo build --locked --offline --release --manifest-path tools/corrplot-ng/Cargo.toml
```

For a local CLI smoke test after building:

```sh
tools/corrplot-ng/target/release/corrplot-ng --help
tools/corrplot-ng/target/release/corrplot-ng render --config prototypes/publication-config-v1/examples/basic.json --output /tmp/corrplot-ng.svg
python scripts/validate_svg.py /tmp/corrplot-ng.svg
```

The current CI workflows run path-filtered correctness checks. Performance benchmarks are manual and are not part of ordinary pull-request CI. See the [testing strategy](docs/testing-strategy.md), [regression test guide](tests/README.md), and [benchmark operations](benchmarks/README.md).

## License

This project is licensed under the [MIT License](LICENSE).
