# Legacy visual regression snapshots

The 15 committed PNGs are reference renders from `corrplot` version 0.95.
Their IDs, sources and intent are recorded in `legacy-visual-cases.json`.

The R regression test renders only `corrplotng::corrplot_ng()` and compares its
pixels to these saved baselines. It does not render the original implementation
a second time during routine CI runs.

## Intentionally regenerate the baselines

Use an R environment with the exact `corrplot` version in the manifest, plus
`jsonlite` and `png`. From the repository root run:

```sh
UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 Rscript scripts/update-legacy-visual-snapshots.R
```

The script writes 900×900 PNGs to
`implementations/r/tests/testthat/fixtures/legacy-visual/`. Review all images
and commit the complete set together. Regeneration is opt-in so normal tests
cannot silently rewrite approved references. Exact pixel matching assumes the
same R graphics device and environment as the CI runner.
