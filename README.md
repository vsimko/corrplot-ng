# corrplot-ng

**An experimental toolkit for correlation-matrix visualization and compatibility across runtimes.**

corrplot-ng explores a modern, renderer-neutral approach to correlation plots while preserving established R workflows. It is an early-stage project, not a production-ready replacement for the original R package.

## Current status

The repository contains several working components with different maturity levels:

- **R package:** a provisional adapter. Its default `corrplot_ng()` path delegates to upstream `corrplot`; the experimental `engine = "scene"` path prepares renderer-neutral scene-v1 data. The package also provides normalized-matrix and file adapters, `cor.mtest()`, and palette/color-legend helpers.
- **Normalized-matrix v1:** a language-neutral contract for already-computed square matrices, including labels, missing values, and source-index mappings. R matrix and CSV/TSV/JSON file adapters produce this representation.
- **Python scene/SVG prototype:** a reference implementation that builds a renderer-neutral scene and exports deterministic SVG. It demonstrates the seven prototype glyph methods, but it is not a production shared core.
- **Rust standalone CLI:** a compiled native executable that validates and renders supported publication configurations and matrix inputs to SVG. It has no Python or R runtime dependency and is tested on Linux, macOS, and Windows.
- **Native Node.js binding:** an experimental N-API package in `bindings/node/` that calls the shared Rust matrix validator and typed-scene SVG/RGBA renderers. Its source-build package layout is tested on Node.js 22, 24, and 26 across Linux, macOS, and Windows; platform-specific prebuilt npm publishing and the unified TypeScript facade are not yet implemented.
- **Scene-v1 and publication-config v1:** versioned interchange/configuration contracts used by the current prototypes and CLI.

**Rust is the selected strategic shared-core implementation** under [ADR 0001](docs/adr/0001-shared-core-architecture.md). Merged PR #180 proves that the CLI and the disposable R/rextendr adapter call the same `crates/corrplot-core` matrix-validation operation. PR #184 records the current toolchain and package-size evidence. This is a focused proof, not a production R migration or a complete shared visualization engine. Production R rollout remains gated by CRAN guidance on the remaining vendor-metadata NOTE and installed package size, plus a controlled measurement of maintainer effort. C++/Rcpp remains a possible R-only fallback if Rust packaging proves unacceptable. The cross-runtime TypeScript architecture is specified in [ADR 0002](docs/adr/0002-cross-runtime-typescript-architecture.md). The first native Node-API binding slice is implemented in `bindings/node/`; browser WASM, Canvas, the unified TypeScript facade, and additional output integrations remain in progress.

## Architecture

The project separates language-specific input adapters, versioned language-neutral contracts, shared Rust semantics, and output renderers. The Rust core is the strategic shared implementation; scene-v1 remains the language-neutral interchange and conformance contract.

See [the current architecture](docs/architecture.md), [ADR 0001](docs/adr/0001-shared-core-architecture.md), [ADR 0002](docs/adr/0002-cross-runtime-typescript-architecture.md), the [repository layout and ownership map](docs/repository-layout.md), [top-level requirements](docs/requirements.md), and the [future vision](docs/future-vision.md).

## Compatibility scope

The original [R corrplot package](https://github.com/taiyun/corrplot) remains the reference for legacy behavior. The current R adapter preserves the legacy plotting path by delegation and has an experimental scene-preparation path. Automated tests cover selected legacy cases, normalized-input behavior, scene/SVG semantics, and saved raster references. These tests are representative, not exhaustive; full behavioral compatibility is not claimed.

The Python scene/SVG prototype supports seven publication glyph methods. The current native CLI supports a smaller documented subset, including circle and number glyphs. Do not assume feature parity between the prototype, R adapter, and CLI.

## Local development

### Prerequisites

- Git and Bash (Linux, macOS, WSL, or Git Bash on Windows)
- Python 3.10+ and `jsonschema`
- R 4.x with `corrplot` 0.95, `testthat`, `jsonlite`, and `png`
- Rust stable toolchain (for the native CLI and Node/WASM bindings)
- Node.js 22+ and npm (for the native Node binding)
- `wasm-pack` (for browser-WASM checks)

Install the Python and R packages used by the local checks:

```sh
python3 -m pip install jsonschema
Rscript -e 'install.packages(c("corrplot", "testthat", "jsonlite", "png"), repos = "https://cloud.r-project.org")'
```

### Unified local commands

Run commands from the repository root through `bash scripts/dev-check.sh`:

```sh
# Fast Python foundation checks, including fixture/schema and SVG validation
bash scripts/dev-check.sh quick

# Focused checks for an individual component
bash scripts/dev-check.sh test-r
bash scripts/dev-check.sh test-r-legacy
bash scripts/dev-check.sh test-rust-cli
bash scripts/dev-check.sh test-node
bash scripts/dev-check.sh test-wasm

# Full locally supported correctness suite
bash scripts/dev-check.sh test-all

# Builds
bash scripts/dev-check.sh build-cli
bash scripts/dev-check.sh build-node

# Explicitly opt-in performance measurements
bash scripts/dev-check.sh benchmarks
bash scripts/dev-check.sh benchmarks-stress
```

`quick` runs the Python foundation checks without stress benchmarks. The
focused commands run the corresponding component checks; `test-all` combines
the quick, R, native CLI, Node, browser-WASM, and pinned legacy-oracle checks.
The R checks require the pinned `corrplot` 0.95 reference. Browser-WASM checks
require `wasm-pack`. The experimental WASM/R host workflow additionally
depends on the pinned Wasmtime C API and remains a separate opt-in workflow.

The benchmark commands write result files under the system temporary
directory. Stress workloads are never included in `quick` or `test-all`;
run `benchmarks-stress` only when explicitly measuring the 500x500 and
1000x1000 workloads. Set `PYTHON=/path/to/python` to select a non-default
Python executable.

The script is a Bash entry point; native Windows PowerShell is not supported.
Use WSL or Git Bash on Windows. Individual CI workflows still provide the
authoritative multi-platform checks.

See the [testing strategy](docs/testing-strategy.md), [regression test guide](tests/README.md), and [benchmark operations](benchmarks/README.md).

## Citation

For a released version, cite the exact corrplot-ng release identified by its
Git tag and use the machine-readable metadata in
[`CITATION.cff`](CITATION.cff). The repository-level citation metadata does
not set a project version yet because the repository has not established a
versioned project release; component package versions are not a substitute
for a corrplot-ng release version. The release process should add the release
version and date to `CITATION.cff` when the first project release is tagged.

If no tagged release is available, cite the exact source commit (full Git
commit SHA) used. No archival DOI is currently specified. If a release is
later archived and assigned a DOI, cite that DOI together with the archived
release version; do not infer or reuse a DOI from another version.

## License

This project is licensed under the [MIT License](LICENSE).
