# corrplot-ng CLI (first native slice)

This is a standalone Rust executable. It has **zero third-party Cargo dependencies** and does not invoke Python or R.

## Build

From the repository root:

```sh
cargo build --release --manifest-path tools/corrplot-ng/Cargo.toml
./tools/corrplot-ng/target/release/corrplot-ng --help
```

The executable is self-contained with respect to the application: no Python, R, or external CLI runtime is needed. Platform C runtime requirements depend on the Rust target and linker configuration.

## Release binaries

Pushing a tag named `cli-vX.Y.Z` that matches the version in
`Cargo.toml` triggers the native release workflow. For example:

```sh
git tag cli-v0.1.0
git push origin cli-v0.1.0
```

Each GitHub release contains one archive per supported platform, a
`SHA256SUMS` file, and a `BUILD-INFO.txt` inside each archive.

| Asset | Target | Runtime requirements |
| --- | --- | --- |
| `corrplot-ng-linux-x86_64-musl.tar.gz` | Linux x86_64, statically linked musl | No external C runtime; tested in a minimal Alpine container without Python |
| `corrplot-ng-macos-universal.tar.gz` | macOS universal binary (Intel x86_64 + Apple Silicon arm64) | macOS 11 or later; system libraries only |
| `corrplot-ng-windows-x86_64.zip` | Windows x86_64, MSVC with static CRT | Windows system libraries; no Python or R runtime |

Download the archive for your platform and verify it against the published
checksums before installation. On Linux or macOS:

```sh
sha256sum -c SHA256SUMS
tar -xzf corrplot-ng-linux-x86_64-musl.tar.gz
install -m 0755 corrplot-ng ~/.local/bin/corrplot-ng
corrplot-ng --help
corrplot-ng version
```

For macOS, substitute `corrplot-ng-macos-universal.tar.gz`. On Windows,
extract `corrplot-ng-windows-x86_64.zip`, place `corrplot-ng.exe` in a
directory on `PATH`, and verify the archive with:

```powershell
(Get-FileHash .\corrplot-ng-windows-x86_64.zip -Algorithm SHA256).Hash
```

Compare that value with the corresponding line in `SHA256SUMS`. The
`BUILD-INFO.txt` file records the release ref, commit, target triple,
Rust/Cargo versions, license, and runtime assumptions. The release workflow
also runs the compiled executable's help/version smoke tests; the Linux static
binary is additionally executed inside a minimal container without Python.

Each platform archive also contains `RELEASE-AUDIT.json`. It records the
executable size, version-command startup latency (30 samples, median and p95),
runtime link inspection, and peak child-process RSS where the host standard
library exposes it. Windows RSS is explicitly marked unavailable rather than
inferred. The audit script is build-time tooling only; it is not a runtime
dependency and is never launched by the native executable.


### Toolchain audit snapshot

The PR release-build run on 2026-10-31 produced the following measurements.
Startup is the median and p95 of 30 launches of `corrplot-ng version`;
RSS is the maximum resident set size across those child processes where
available.

| Target | Binary size | Startup median / p95 | Peak child RSS | Runtime linkage |
| --- | ---: | ---: | ---: | --- |
| Linux x86_64 musl | 664,288 B | 0.202 / 0.241 ms | 14.3 MiB | Static; no dynamic libraries |
| macOS universal | 1,058,816 B | 3.614 / 4.281 ms | 1.4 MiB | macOS `libSystem` only |
| Windows x86_64 MSVC | 519,168 B | 5.823 / 6.352 ms | Not measured | Static MSVC CRT; Windows system DLLs |

These are measurements from the corresponding GitHub-hosted runners, not
portable performance guarantees. Windows peak RSS is deliberately reported
as unavailable because the standard-library measurement used by the audit
script is not supported there. Cross-target build and smoke-test feasibility
was verified for all three platform packages.


A manual run of **Release native CLI** in GitHub Actions builds and stores
workflow artifacts but does not publish a GitHub release. Publishing is
triggered only by a matching `cli-vX.Y.Z` tag. DEB/RPM and distribution
repository packaging remain deferred to issue #99.


### Renderer capabilities and diagnostics

Print the machine-readable capability declaration for the native SVG output:

\`\`\`sh
corrplot-ng capabilities
\`\`\`

For a single command, add \`--diagnostics json\` to emit a diagnostic-v1
record on failure while preserving the nonzero exit status:

\`\`\`sh
corrplot-ng render --input data/correlation.csv --diagnostics json
\`\`\`

### Batch rendering

Create a UTF-8 text manifest with one publication-config v1 JSON path per
line. Paths are relative to the manifest file. Blank lines and lines beginning
with \`#\` are ignored.

\`\`\`text
configs/experiment-a.json
configs/experiment-b.json
configs/experiment-c.json
\`\`\`

Render each configuration independently:

\`\`\`sh
corrplot-ng render-batch --manifest figures.txt --output-dir figures
\`\`\`

Successful outputs are retained even when another item fails. Output names
use a three-digit manifest index and the configuration file stem to avoid
collisions. The process exits with status 1 if any item fails. With
\`--diagnostics json\`, stdout contains a report with a \`successes\` array and
a \`diagnostics\` array. Each diagnostic includes the manifest index and input
path in \`details\`.

### Make

After installing the binary, a Make target can render figures only when the
input changes:

```make
figures/correlation.svg: data/correlation.csv
	mkdir -p figures
	corrplot-ng render --input $< --output $@

.PHONY: figures
figures: figures/correlation.svg
```

### LaTeX

The CLI writes SVG directly. With the LaTeX `svg` package and Inkscape
available in the document build environment, include the generated figure:

```tex
\usepackage{svg}

\begin{document}
\includesvg[width=\linewidth]{figures/correlation}
\end{document}
```

Compile with shell escape enabled when required by the `svg` package
(for example, `lualatex --shell-escape paper.tex`).

### CI

A CI job can download the versioned Linux archive, verify its checksum, and
render a figure without installing Python or R:

```yaml
- name: Install corrplot-ng
  env:
    VERSION: "0.1.0"
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    gh release download "cli-v${VERSION}" \
      --repo vsimko/corrplot-ng \
      --pattern 'corrplot-ng-linux-x86_64-musl.tar.gz' \
      --pattern SHA256SUMS
    sha256sum --ignore-missing -c SHA256SUMS
    tar -xzf corrplot-ng-linux-x86_64-musl.tar.gz
    sudo install -m 0755 corrplot-ng /usr/local/bin/corrplot-ng
- name: Render figure
  run: corrplot-ng render --input data/correlation.csv --output figures/correlation.svg
```



## Input format

The first version accepts a square labelled CSV or TSV matrix. The first cell is empty (or `label`), followed by column labels. Each subsequent row begins with the corresponding row label. Empty cells, `NA`, `NaN`, and `null` mean missing values.

```csv
,alpha,beta,gamma
alpha,1,0.65,-0.4
beta,0.65,1,NA
gamma,-0.4,NA,1
```

Correlation mode (default) requires finite values in [-1, 1], symmetry, and a unit diagonal. General-matrix mode accepts arbitrary finite values and does not impose correlation semantics.

The native CLI also accepts renderer-neutral **scene-v1 JSON** as an input.
It reads the label text and complete cell matrix from `labels` and `cells`;
layout and style metadata are ignored because the CLI creates its own output
scene. The imported values pass through the same correlation/general-matrix
validation used for CSV and TSV inputs.

Input format detection is automatic by default: `.json` selects scene-v1,
`.tsv` selects tab-separated input, and other files default to CSV. For stdin
or ambiguous extensions, use `--input-format auto|csv|tsv|scene-json` to
override detection. CSV and TSV delimiters are strict when explicitly selected.


## Commands

```sh
corrplot-ng validate --input matrix.csv
corrplot-ng inspect --input matrix.csv
corrplot-ng render --input matrix.csv --output figure.svg
corrplot-ng render --input matrix.csv --format scene-json --output scene.json
corrplot-ng render --input matrix.csv --kind general --format normalized-json --output normalized.json
corrplot-ng render --input matrix.csv --method number --title "My correlations" > figure.svg
cat matrix.csv | corrplot-ng render --input - > figure.svg
corrplot-ng render --input scene.json --input-format scene-json --output imported.svg
cat scene.json | corrplot-ng validate --input - --input-format scene-json
corrplot-ng presets
corrplot-ng capabilities
corrplot-ng render --input matrix.csv --diagnostics json
corrplot-ng version
```

Supported render methods are currently `circle` and `number`. The `normalized-json` output serializes the parsed matrix to the versioned, renderer-independent normalized-matrix v1 contract (including explicit `null` missing cells and zero-based source indices). It reuses the same CSV/TSV/scene-v1 input adapters; use `--kind general` when normalizing a matrix without correlation-specific constraints. The scene-json output follows the current experimental scene-v1 shape. This first CLI slice deliberately does not claim support for the full publication-config v1 document, ordering algorithms, all seven legacy glyphs, incremental rebuilds, or batch rendering; those remain follow-up work against the existing contracts.

## Dependency policy

Do not add a third-party crate without documenting why the standard library is insufficient, the crate's license/provenance, build-time dependency impact, and whether it adds runtime dependencies. Keep Cargo.lock committed even while the dependency graph is empty.


## Publication configuration v1 (native CLI)

The native CLI accepts a publication configuration with the following commands:

    corrplot-ng validate --config figure.json
    corrplot-ng render --config figure.json
    corrplot-ng render --config figure.json --output figure.svg
    corrplot-ng render --config figure.json --format scene-json --output scene.json

The configuration loader validates strict object fields, inline or CSV-backed matrices, matrix semantics, missing-value policy, supported plot settings, colors, physical dimensions, typography, and output format. CSV paths are resolved relative to the configuration file. The renderer currently supports correlation/general numeric matrices, circle/number glyphs, full/upper/lower display, and original/alphabetical ordering. Correlation matrices additionally support deterministic AOE and average-linkage hierarchical ordering. The scene-json format emits the renderer-neutral scene-v1 shape, including cell visibility, color, typography, legend, ordering metadata, and general-matrix normalization metadata.

Unsupported v1 semantics are rejected explicitly rather than ignored. General numeric matrices now support automatic color limits for inline input and explicit `matrix.col_limits` for inline or CSV-backed input. Explicit limits must be finite, strictly increasing, and cover every observed value; constant automatic ranges are padded deterministically. One-sided values use the sequential YlOrBr palette, while mixed-sign values use the configured negative/neutral/positive diverging colors. CSV-backed general matrices require explicit limits because the config contract validates them before data loading. Custom `colors.scale` definitions and significance overlays remain unsupported by the native CLI.


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


## SVG export validation

The repository includes a standard-library validator for the supported
self-contained SVG subset:

```sh
python scripts/validate_svg.py figure.svg
```

It checks XML well-formedness, namespace and dimensions, viewBox geometry,
unique IDs and local references, accessibility references, and rejects
scripts, event handlers, remote assets, and external CSS dependencies. This
is test/build tooling only; the native executable has no Python dependency.

CI validates a generated native CLI SVG on Ubuntu and rasterizes the shared
Python renderer output with librsvg. For a manual LaTeX smoke test, rasterize
the SVG with `rsvg-convert`, place the resulting `figure.png` beside
`tests/fixtures/svg-publication-example.tex`, and compile the document with
`pdflatex`. The supported subset and limitations are documented in
`tests/README.md`.
