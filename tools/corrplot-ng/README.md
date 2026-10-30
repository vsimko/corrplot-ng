# corrplot-ng CLI (first native slice)

This is a standalone Rust executable. It has **zero third-party Cargo dependencies** and does not invoke Python or R.

## Build

From the repository root:

```sh
cargo build --release --manifest-path tools/corrplot-ng/Cargo.toml
./tools/corrplot-ng/target/release/corrplot-ng --help
```

The executable is self-contained with respect to the application: no Python, R, or external CLI runtime is needed. Platform C runtime requirements depend on the Rust target and linker configuration.

## Input formats and matrix contract

The CLI accepts labelled CSV, TSV, and JSON matrix input. CSV and TSV use the same labelled-square layout: the first cell is empty (or \`label\`), followed by column labels; each data row starts with its matching row label. Empty cells, \`NA\`, \`NaN\`, and \`null\` represent missing values. Quoted CSV fields support delimiters, escaped quotes, and embedded newlines.

\`\`\`csv
,alpha,beta,gamma
alpha,1,0.65,-0.4
beta,0.65,1,NA
gamma,-0.4,NA,1
\`\`\`

JSON matrix input uses the compact \`labels\` / \`values\` contract. Values are numbers or \`null\`; \`kind\` is optional and may be \`correlation\` or \`general\`.

\`\`\`json
{
  "kind": "correlation",
  "labels": ["alpha", "beta"],
  "values": [[1, 0.4], [0.4, 1]]
}
\`\`\`

The CLI also accepts renderer-neutral scene-v1 JSON output as input. It reconstructs the matrix from each label's \`text\` and the cells' \`row\`, \`column\`, and \`value\` fields; scene kind determines correlation or general-matrix semantics.

Correlation mode (default for CSV/TSV and compact JSON without \`kind\`) requires finite values in [-1, 1], symmetry, and a unit diagonal. General-matrix mode accepts arbitrary finite values and does not impose correlation semantics.

Input format detection is deterministic: \`.csv\`, \`.tsv\` / \`.tab\`, and \`.json\` extensions take precedence; otherwise JSON is detected from an object/array prefix and delimited text from its first non-empty row. If comma and tab detection is ambiguous, specify \`--input-format\`. Use \`--input-format csv|tsv|json\` to override detection. UTF-8 is required; a UTF-8 BOM is accepted.

## Commands

```sh
corrplot-ng validate --input matrix.csv
corrplot-ng inspect --input matrix.csv
corrplot-ng render --input matrix.csv --output figure.svg
corrplot-ng render --input matrix.csv --format scene-json --output scene.json
corrplot-ng render --input matrix.tsv --input-format tsv --method number --title "My correlations" --output - > figure.svg
corrplot-ng render --input matrix.json --input-format json --format scene-json --output - > scene.json
cat matrix.csv | corrplot-ng render --input - --input-format csv --output - > figure.svg
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
encoding, displayed region, variable ordering, color palette, label font family,
value annotations, and output paths. It prints a complete configuration preview, validates that exact
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


## Wizard performance check

The wizard is deliberately excluded from ordinary command execution. To verify
that separation, compare the same release profile built from the base revision
and the wizard branch on the same machine:

```sh
git worktree add /tmp/corrplot-ng-base origin/main
cargo build --release --manifest-path /tmp/corrplot-ng-base/tools/corrplot-ng/Cargo.toml
cargo build --release --manifest-path tools/corrplot-ng/Cargo.toml
python benchmarks/cli_startup.py \
  --baseline /tmp/corrplot-ng-base/tools/corrplot-ng/target/release/corrplot-ng \
  --candidate tools/corrplot-ng/target/release/corrplot-ng \
  --output cli-performance.json
```

The harness measures fresh-process `version` startup, `validate --config`,
and end-to-end SVG rendering of a deterministic 40x40 matrix. It reports
median, p95, minimum, and maximum wall-clock time. Run both binaries on the
same host with the same Rust toolchain; repeat on each supported OS when
making cross-platform claims. These are measurements, not brittle CI timing
assertions. Record the baseline and candidate JSON artifacts with the PR
before claiming that the performance acceptance criterion has passed.


### Initial before/after measurement

The first comparison ran on the Ubuntu GitHub Actions runner with Rust 1.99.1,
a deterministic 40x40 matrix, 30 fresh-process startup/validation samples, and
10 render samples. Baseline: PR base `027b2166f8ba`; candidate: wizard
implementation `16fe9a0eef72`.

| Operation | Baseline median | Candidate median | Change |
| --- | ---: | ---: | ---: |
| `version` startup | 0.5102 ms | 0.4955 ms | -2.87% |
| `validate --config` | 0.7450 ms | 0.7499 ms | +0.66% |
| `render --config` (40x40 SVG) | 7.9698 ms | 7.9182 ms | -0.65% |

All three measurements were within the baseline-relative budget. The budget is
`max(5% of baseline median, 0.5 ms)` for startup/validation and
`max(5% of baseline median, 2 ms)` for rendering. This is one runner's
measurement, not a cross-platform performance guarantee. The CI workflow
uploads a fresh comparison artifact for each pull-request revision.

Measurement run: https://github.com/vsimko/corrplot-ng/actions/runs/41145752180


## Stream and output behavior

Use \`--input -\` to read stdin and \`--output -\` (or omit \`--output\`) to write a rendered artifact to stdout. The render stream contains only SVG or scene-v1 JSON; diagnostics are written to stderr. File output is staged in a temporary file and renamed only after rendering succeeds, so malformed input does not truncate an existing destination.

\`--format\` selects the output format: \`svg\` (default) or \`scene-json\`. PNG, PDF, and HTML are not implemented and fail explicitly. Parquet is not supported. No Python or R process is invoked for CSV, TSV, JSON, stdin, stdout, or file output.
