# corrplot-ng CLI (first native slice)

This is a standalone Rust executable. It has **zero third-party Cargo dependencies** and does not invoke Python or R.

## Build

From the repository root:

```sh
cargo build --release --manifest-path tools/corrplot-ng/Cargo.toml
./tools/corrplot-ng/target/release/corrplot-ng --help
```

The executable is self-contained with respect to the application: no Python, R, or external CLI runtime is needed. Platform C runtime requirements depend on the Rust target and linker configuration.

## Input format

The first version accepts a square labelled CSV or TSV matrix. The first cell is empty (or `label`), followed by column labels. Each subsequent row begins with the corresponding row label. Empty cells, `NA`, `NaN`, and `null` mean missing values.

```csv
,alpha,beta,gamma
alpha,1,0.65,-0.4
beta,0.65,1,NA
gamma,-0.4,NA,1
```

Correlation mode (default) requires finite values in [-1, 1], symmetry, and a unit diagonal. General-matrix mode accepts arbitrary finite values and does not impose correlation semantics.

## Commands

```sh
corrplot-ng validate --input matrix.csv
corrplot-ng inspect --input matrix.csv
corrplot-ng render --input matrix.csv --output figure.svg
corrplot-ng render --input matrix.csv --format scene-json --output scene.json
corrplot-ng render --input matrix.csv --method number --title "My correlations" > figure.svg
cat matrix.csv | corrplot-ng render --input - > figure.svg
corrplot-ng presets
corrplot-ng version
```

Supported render methods are currently `circle` and `number`. The scene-json output follows the current experimental scene-v1 shape. This first CLI slice deliberately does not claim support for the full publication-config v1 document, ordering algorithms, all seven legacy glyphs, incremental rebuilds, or batch rendering; those remain follow-up work against the existing contracts.

## Dependency policy

Do not add a third-party crate without documenting why the standard library is insufficient, the crate's license/provenance, build-time dependency impact, and whether it adds runtime dependencies. Keep Cargo.lock committed even while the dependency graph is empty.


## Publication configuration v1 (native CLI)

The native CLI accepts a publication configuration with the following commands:

    corrplot-ng validate --config figure.json
    corrplot-ng render --config figure.json
    corrplot-ng render --config figure.json --output figure.svg
    corrplot-ng render --config figure.json --format scene-json --output scene.json

The configuration loader validates strict object fields, inline or CSV-backed matrices, matrix semantics, missing-value policy, supported plot settings, colors, physical dimensions, typography, and output format. CSV paths are resolved relative to the configuration file. The renderer currently supports correlation/general numeric matrices, circle/number glyphs, full/upper/lower display, and original/alphabetical ordering. Correlation matrices additionally support deterministic AOE and average-linkage hierarchical ordering. The scene-json format emits the renderer-neutral scene-v1 shape, including cell visibility, color, typography, legend, ordering metadata, and general-matrix normalization metadata.

Unsupported v1 semantics are rejected explicitly rather than ignored. This initial native slice does not yet implement color scales, significance overlays, or col_limits; these remain follow-up work.


## Interactive configuration wizard

Run `corrplot-ng init` in an interactive terminal to create a reusable
`publication-config v1` file. The wizard asks for the labelled CSV/TSV input,
matrix semantics, supported correlation method, missing-value policy, cell
encoding, displayed region, variable ordering, value annotations, and output
paths. It prints a complete configuration preview, validates that exact
configuration through the same `Publication::load` path used by
`validate --config`, and asks before writing. Existing files are not
overwritten without confirmation.

```sh
corrplot-ng init --output figures/correlation.json
corrplot-ng validate --config figures/correlation.json
corrplot-ng render --config figures/correlation.json
```

Relative input paths are resolved relative to the configuration file's
directory. Use an absolute path when the configuration should be portable
across different working directories. The initial wizard exposes only options
supported by the native CLI; it does not enable significance overlays or
unsupported color-scale features.

The wizard is opt-in and lives in its own Rust module with no third-party
dependencies. It is called only after the `init` command is selected.
`render`, `validate`, `inspect`, `presets`, and `version` do not prompt
or initialize terminal UI. When stdin or stdout is not a TTY, `init` exits
with an actionable error before displaying prompts, so scripts and CI cannot
hang.
