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

## Shared fixtures and unsupported behavior

`tests/fixtures/correlation-cases.json` contains language-neutral correlation matrices. `tests/legacy-visual/cases.json` records the pinned visual case inventory. Use JSON `null` for missing values, explicit labels/dimensions, stable case IDs, and documented expected properties.

Unsupported legacy features remain explicit gaps in the compatibility inventory. Passing these fixtures does not establish complete API compatibility; add a fixture and a corresponding assertion before claiming support for a new behavior.
