# Tests

## Regression layers

The compatibility suite separates three kinds of evidence. A passing layer is not a claim of full corrplot API compatibility.

| Layer | What it verifies | CI / command |
| --- | --- | --- |
| **Semantic** | The pinned legacy `corrplot` oracle preserves matrix ordering, labels, coordinates, values, and missingness; the same oracle output is passed through the renderer-neutral scene model. | `.github/workflows/test-foundation.yml`, job `legacy-r-oracle`; run `Rscript prototypes/shared-svg/test_legacy_corrplot_oracle.R` followed by `python prototypes/shared-svg/test_legacy_scene_bridge.py /tmp/corrplot-legacy-scene-input.json`. |
| **Structural SVG** | Each legacy-derived matrix is rendered with all seven implemented methods. Tests parse the SVG XML and verify dimensions, cell count, labels, annotation placement, missing-cell labels, glyph-specific elements, and deterministic serialization. | The same `legacy-r-oracle` job runs the bridge test. The broader scene/SVG tests run with `python -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v`. |
| **Pixel baseline** | Fifteen selected 900×900 PNGs are compared to saved renders from `corrplot` 0.95. This currently validates the R compatibility adapter, not the independent Python SVG renderer or a future production engine. | `R CMD check implementations/r`, test `test-visual-regression.R`. Pixel mismatches include the fixture ID in the failure. |

Semantic bridge failures are prefixed `SEMANTIC`; SVG/XML/layout failures are prefixed `STRUCTURAL SVG`; saved-raster mismatches are reported as `Rendered pixels differ from saved legacy baseline`. This keeps the source of a regression visible instead of conflating data semantics with renderer markup or rasterization.

## Pinned reference and regeneration

The reference is `corrplot` 0.95 at commit `62532f700e6c9d04b93ffbf0bf56e264b0e0368e`. The reference commit, source files, case IDs, and intent are recorded in `tests/legacy-visual/cases.json` and `implementations/r/tests/testthat/fixtures/legacy-visual-cases.json`.

Regeneration is opt-in and refuses to run unless the installed package version matches the pinned manifest:

```sh
UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 Rscript scripts/update-legacy-visual-snapshots.R
```

Review all generated PNGs and commit the complete set together. Ordinary tests never rewrite baselines.

## Platform and font limitations

The semantic and structural tests are the cross-platform contract; they do not require pixel-identical output across operating systems. PNG equality depends on the same R graphics device, Cairo build, fonts, and graphics environment. The current saved-raster test is intentionally run only in the pinned R CI environment. Do not loosen semantic or SVG structural assertions to accommodate font differences.

## Current scope

- The R package remains a provisional scaffold. Its default exported `corrplot_ng()` delegates to `corrplot::corrplot()`; the PNG baseline therefore verifies that adapter path.
- The Python scene/SVG prototype is independent and experimental. The bridge runs actual matrices exported by the pinned legacy oracle through all seven prototype glyph methods, but this is not proof that the R scene engine or every legacy option is compatible.
- The shared JSON fixture tests validate schema and matrix properties; they do not replace the pinned legacy oracle.

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
