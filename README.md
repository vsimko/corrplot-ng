# corrplot-ng

**An experimental toolkit for correlation-matrix visualization and compatibility across runtimes.**

corrplot-ng explores a modern, renderer-neutral approach to correlation plots while preserving established R workflows. It is an early-stage project, not a production-ready replacement for the original R package.

## Current status

The repository contains working components at different maturity levels:

- **R package:** a provisional adapter. Its default `corrplot_ng()` path delegates to upstream `corrplot`; the experimental `engine = "scene"` path prepares renderer-neutral scene-v1 data. The package also provides normalized-matrix and file adapters, `cor.mtest()`, and palette/color-legend helpers.
- **Normalized-matrix v1:** a language-neutral contract for already-computed square matrices, including labels, missing values, and source-index mappings. R matrix and CSV/TSV/JSON file adapters produce this representation.
- **Python scene/SVG reference:** `python/reference/` contains a non-production implementation that builds a renderer-neutral scene and exports deterministic SVG. It demonstrates seven prototype glyph methods, but it is not a production shared core.
- **Rust shared core and CLI:** `rust/crates/corrplot-core` is the strategic shared implementation. It provides matrix validation, an initial scene-v1 matrix parser and normalized-matrix-to-scene builder, and typed-scene SVG/RGBA rendering. The standalone CLI consumes this core and exports supported publication configurations and matrix inputs to SVG. Scene-v1 construction is incremental; complete presentation fidelity and full legacy feature coverage are not established.
- **Native Node.js binding:** the experimental Node-API package in `npm/packages/native-node/` (npm package; Rust crate in `rust/bindings/node/`) consumes the same Rust core. It exposes matrix validation, typed-scene SVG/RGBA rendering, and matrix-bearing scene-v1 to SVG/RGBA operations. The binding is tested on Node.js 22, 24, and 26; PR CI uses Linux x64 on Node 24, while comprehensive CI covers Linux, macOS, and Windows on all three Node versions.
- **Browser WebAssembly binding:** `npm/packages/wasm/` (npm package; Rust crate in `rust/bindings/wasm/`) compiles the same Rust core for `wasm32-unknown-unknown`. It exposes validation, typed-scene SVG/RGBA, and matrix-bearing scene-v1 to SVG/RGBA operations. Node-based binding tests run on Node 24/26; browser smoke tests cover Chromium, Firefox, and WebKit on Ubuntu. This is a focused browser integration, not full browser/API parity.
- **TypeScript facade:** `npm/packages/typescript/` provides runtime-specific Node and browser exports, backend selection, matrix validation, Rust-backed normalized-matrix scene construction, typed-scene SVG/RGBA rendering, matrix-bearing scene-v1 JSON compatibility rendering, renderer capabilities, diagnostic errors, adapter injection, and disposal. Numeric typed arrays are supported with an explicit missing-cell mask. Full scene-v1 presentation parity remains separate work; direct Canvas rendering remains a separate browser integration.
- **React adapter:** `npm/packages/react/` provides a browser-only Corrplot component and useCorrplot hook. It depends directly on the WASM package, keeps React as a peer dependency, and is tested under Strict Mode and in Chromium, Firefox, and WebKit. Its Vite and esbuild production bundles are checked for Node-only dependencies.
- **Vue adapter:** `npm/packages/vue/` provides a browser-only Corrplot component and useCorrplot composable. It depends directly on the WASM package, keeps Vue as a peer dependency, and is tested for reactive updates, mount/unmount, errors, and recovery in Chromium, Firefox, and WebKit.
- **Versioned contracts and shared fixtures:** canonical language-neutral schemas live under [contracts/](contracts/README.md), with cross-runtime conformance data under [fixtures/](fixtures/README.md).

**Rust is the selected strategic shared-core implementation** under [ADR 0001](docs/adr/0001-shared-core-architecture.md). The CLI, Node binding, browser-WASM binding, and disposable R/rextendr adapter consume the shared Rust core for the operations implemented in each integration. This is incremental shared implementation, not proof of full semantic parity or a complete visualization engine. Production R rollout remains gated by CRAN/package-footprint evidence and sustainable maintenance. C++/Rcpp has been explicitly rejected and is not a maintained fallback.

The cross-runtime TypeScript architecture is specified in [ADR 0002](docs/adr/0002-cross-runtime-typescript-architecture.md). The native Node and browser-WASM bindings are separate runtime backends, selected through runtime-specific TypeScript exports. The normalized-matrix scene workflow in #145 and direct browser Canvas rendering in #144 are implemented. The initial cross-backend parity, visual-conformance, and informational performance-baseline suite was completed under #146; full scene-v1 presentation semantics remain incomplete beyond the matrix-bearing slice. The React adapter (#291) and Vue adapter (#292) are implemented as separate browser-only packages. See the [npm package distribution plan](docs/npm-package-distribution.md). Package builds, tests, versioning, and tarball staging may be prepared before release, but public npm publication is explicitly deferred.

## Architecture

The project separates language-specific input adapters, versioned language-neutral contracts, shared Rust semantics, and output renderers. The Rust core is the strategic shared implementation; scene-v1 remains the language-neutral interchange and conformance contract.

See [the current architecture](docs/architecture.md), [ADR 0001](docs/adr/0001-shared-core-architecture.md), [ADR 0002](docs/adr/0002-cross-runtime-typescript-architecture.md), the [repository layout and ownership map](docs/repository-layout.md), [top-level requirements](docs/requirements.md), and the [future vision](docs/future-vision.md).

## Compatibility scope

The original [R corrplot package](https://github.com/taiyun/corrplot) remains the reference for legacy behavior. The current R adapter preserves the legacy plotting path by delegation and has an experimental scene-preparation path. Automated tests cover selected legacy cases, normalized-input behavior, scene/SVG semantics, and saved raster references. These tests are representative, not exhaustive; full behavioral compatibility is not claimed.

The Python scene/SVG prototype supports seven publication glyph methods. The current native CLI and Rust-backed bindings expose narrower, documented operation slices. Do not assume complete feature or presentation parity between the prototype, R adapter, CLI, Node, and browser-WASM integrations.

## Local development

Each ecosystem has its own project root, dependency manifest, and targeted
commands. You do not need to install every toolchain to work on one component.
See the [developer setup guide](docs/development.md) for clean setup, build,
test, and package-artifact commands for Python, R, Rust, and npm.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. It
covers project ownership, focused validation, generated artifacts, Changesets,
and release boundaries.

For optional repository-wide orchestration, run from the repository root:

```sh
bash scripts/dev-check.sh quick
bash scripts/dev-check.sh test-r
bash scripts/dev-check.sh test-rust-cli
bash scripts/dev-check.sh test-node
bash scripts/dev-check.sh test-wasm
bash scripts/dev-check.sh test-all
```

The script is Bash-only; use WSL or Git Bash on Windows. Each command requires
only its listed ecosystem toolchains. `test-all` requires all of them.
See the [testing strategy](docs/testing-strategy.md), [CI validation tiers](docs/ci-validation-tiers.md),
[regression test guide](tests/README.md), and [benchmark operations](benchmarks/README.md).

## Citation

For a released version, cite the exact corrplot-ng release identified by its
Git tag and use the machine-readable metadata in
[`CITATION.cff`](CITATION.cff). The repository-level citation metadata
does not set a project version yet because the repository has not established
a versioned project release; component package versions are not a substitute
for a corrplot-ng release version. The release process should add the release
version and date to `CITATION.cff` when the first project release is tagged.

If no tagged release is available, cite the exact source commit (full Git
commit SHA) used. No archival DOI is currently specified. If a release is
later archived and assigned a DOI, cite that DOI together with the archived
release version; do not infer or reuse a DOI from another version.

## License

This project is licensed under the [MIT License](LICENSE).
