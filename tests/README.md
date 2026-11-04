# Tests

## Regression layers

The compatibility suite separates three kinds of evidence. A passing layer is not a claim of full corrplot API compatibility.

| Layer | What it verifies | CI / command |
| --- | --- | --- |
| **Semantic** | The pinned legacy `corrplot` oracle preserves matrix ordering, labels, coordinates, values, and missingness; the same oracle output is passed through the renderer-neutral scene model. | `.github/workflows/test-foundation.yml`, job `legacy-r-oracle`; run `Rscript prototypes/shared-svg/test_legacy_corrplot_oracle.R` followed by `python prototypes/shared-svg/test_legacy_scene_bridge.py /tmp/corrplot-legacy-scene-input.json`. |
| **Structural SVG** | Each legacy-derived matrix is rendered with all seven implemented methods. Tests parse the SVG XML and verify dimensions, cell count, labels, annotation placement, missing-cell labels, glyph-specific elements, and deterministic serialization. | The same `legacy-r-oracle` job runs the bridge test. The broader scene/SVG tests run with `python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v`. |
| **Pixel baseline** | Fifteen selected 900×900 PNGs are compared to saved renders from `corrplot` 0.95. This validates the R compatibility adapter in the pinned graphics environment. | `R CMD check implementations/r`, test `test-visual-regression.R`. Pixel mismatches include the fixture ID in the failure. |
| **R scene adapter fixture parity** | The R scene adapter consumes all four shared correlation cases and the packaged general-matrix/normalized-matrix fixtures, checking scene kind, labels, missingness, and cell values. The packaged correlation corpus is kept byte-identical to the canonical fixture. | `R CMD check implementations/r`; tests `test-scene-adapter.R` and `test-normalized-matrix.R`. |
| **Native CLI fixture parity** | The native CLI consumes every valid case in the shared correlation and general-numeric corpora. It validates each matrix and verifies normalized JSON and renderer-neutral scene JSON preserve labels, missingness, values, and matrix kind. | `.github/workflows/test-rust-cli.yml`; run `python tools/corrplot-ng/tests/verify_shared_fixtures.py --binary tools/corrplot-ng/target/release/corrplot-ng` after building the CLI. |
| **Cross-runtime parity** | Native Node and Node-hosted WASM consume the common correlation and general-numeric matrix fixtures. The test compares validation, normalized-matrix scene construction, SVG, and RGBA for all seven cell methods, plus a deterministic 64×64 correlation matrix. | `.github/workflows/test-node-wasm-parity.yml`; run `node --test bindings/cross-backend/test-node-wasm-parity.cjs` after building the native and Node-WASM bindings. |
| **Scene-v1 statistical overlays** | The shared Rust SVG/RGBA renderers draw supplied confidence-interval geometry and per-cell significance annotations. Blank cells suppress their glyph and interval, hidden cells are omitted, and missing p-values do not generate annotations. | Core renderer tests and `.github/workflows/test-node-wasm-parity.yml`; run `cargo test --locked --offline --manifest-path crates/corrplot-core/Cargo.toml` and the cross-runtime parity command above. |
| **Browser Canvas conformance** | A checked-in scene fixture compares stable interior pixels from direct Canvas 2D output with the shared Rust RGBA renderer and checks number/shade glyph ink. It deliberately avoids byte-for-byte comparison across different text rasterizers. | Browser smoke in `.github/workflows/test-npm-workspace.yml` on Chromium, Firefox, and WebKit. |

Semantic bridge failures are prefixed `SEMANTIC`; SVG/XML/layout failures are prefixed `STRUCTURAL SVG`; saved-raster mismatches are reported as `Rendered pixels differ from saved legacy baseline`. This keeps the source of a regression visible instead of conflating data semantics with renderer markup or rasterization.

## Pinned reference and regeneration

The reference is `corrplot` 0.95 at commit `62532f700e6c9d04b93ffbf0bf56e264b0e0368e`. The reference commit, source files, case IDs, and intent are recorded in `tests/legacy-visual/cases.json` and `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json`.

Regeneration is opt-in and refuses to run unless the installed package version matches the pinned manifest:

```sh
UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 Rscript scripts/update-legacy-visual-snapshots.R
```

Review all generated PNGs and commit the complete set together. Ordinary tests never rewrite baselines.

## Platform and font limitations

The semantic and structural tests are the cross-platform contract; they do not require pixel-identical output across operating systems. PNG equality depends on the same R graphics device, Cairo build, fonts, and graphics environment. The saved-raster test is intentionally run only in the pinned R CI environment. Node and WASM exact SVG/RGBA equality is appropriate because both adapters execute the same Rust core. Browser Canvas is compared to Rust RGBA through stable interior samples and glyph-presence checks, not byte equality, because browser text rasterization differs.

## Current scope

- The R package remains a provisional scaffold. Its default exported `corrplot_ng()` delegates to `corrplot::corrplot()`; the PNG baseline therefore verifies that adapter path.
- The Python scene/SVG prototype is independent and experimental. The bridge runs actual matrices exported by the pinned legacy oracle through all seven prototype glyph methods, but this is not proof that the R scene engine or every legacy option is compatible.
- The native CLI and Node/WASM parity workflows consume the shared correlation and general-numeric matrix corpora. The CLI verifies normalized input and scene JSON; Node/WASM additionally compare scene construction, SVG, and RGBA across all seven cell methods. The R scene adapter also consumes all four shared correlation cases, alongside the normalized-matrix and general-matrix fixture sets, through its own adapter tests.
- The shared JSON fixture tests validate schema and matrix properties; they do not replace the pinned legacy oracle or exercise every R-specific `col_limits`, mapping, and palette normalization option.
- The Node/WASM parity workflow also generates a deterministic 64×64 correlation matrix to cover a larger input without introducing a timing threshold.
- A dedicated scene-v1 fixture exercises confidence intervals, significant/non-significant/unknown status, and blank cells through native Node and WASM SVG/RGBA rendering. The renderers display supplied annotations and geometry; they do not calculate statistical results.
- Browser Canvas has a separate checked-in visual fixture. Stable cell-interior samples are compared with the shared Rust RGBA output; number and shade glyphs are checked for visible ink. This is not a cross-platform pixel-baseline claim.

Run the general fixture checks with Python 3.10 or later:

```sh
python -m unittest discover -s tests -v
```

The fixture integrity tests use only the Python standard library. The shared SVG prototype suite additionally installs `jsonschema` as described in `prototypes/shared-svg/README.md`.

## Normalized matrix input contract

The language-neutral input boundary is defined in
`prototypes/normalized-matrix-v1/`. Its schema and valid/invalid fixtures
specify square dimensions, unique Unicode labels, zero-based source-index
mapping, and `null` missing values. All-missing and constant matrices remain
valid at normalization time; scene construction decides whether they can be
rendered.

Run the schema and semantic fixture suite with:

```sh
python -m unittest discover -s prototypes/normalized-matrix-v1 -p 'test_*.py' -v
```

The suite is part of `Test foundation`. R object and file adapters must
produce the same representation before any renderer is invoked.

## SVG artifact validation and interoperability

`scripts/validate_svg.py` validates the repository's supported self-contained SVG subset using only Python's standard library. It checks well-formed XML, the SVG namespace, positive absolute dimensions, a finite positive viewBox, allowed element types, unique IDs, local references, accessibility references, and the absence of scripts, event handlers, external assets, and external CSS URLs.

Run it against an exported artifact:

```sh
python scripts/validate_svg.py figures/correlation.svg
```

CI validates both the shared Python SVG prototype and a native Rust CLI export. The `svg-interoperability` job also rasterizes the Python SVG with `rsvg-convert` (librsvg), so XML parsing is not the only acceptance check.

A manual LaTeX embedding smoke test can be run with librsvg and a TeX installation:

```sh
rsvg-convert --format png --output figure.png figures/correlation.svg
cp tests/fixtures/svg-publication-example.tex paper.tex
pdflatex -interaction=nonstopmode -halt-on-error paper.tex
```

Run the commands from a directory containing both `paper.tex` and `figure.png`. The LaTeX fixture uses the standard `graphicx` package. The validator intentionally checks a documented subset, not complete SVG 2 conformance or identical rendering across every editor.

## Shared fixtures and unsupported behavior

`tests/fixtures/correlation-cases.json` contains language-neutral correlation matrices. `tests/legacy-visual/cases.json` records the pinned visual case inventory. Use JSON `null` for missing values, explicit labels/dimensions, stable case IDs, and documented expected properties.

Unsupported legacy features remain explicit gaps in the compatibility inventory. Passing these fixtures does not establish complete API compatibility; add a fixture and a corresponding assertion before claiming support for a new behavior.
