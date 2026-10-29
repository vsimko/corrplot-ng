# Legacy visual regression snapshots

The 15 cases are selected from the original `taiyun/corrplot` tests and vignettes.
Their IDs, sources and intent are recorded in `legacy-visual-cases.json`.

## Generate or refresh the PNG baselines

Use a local R environment with the exact `corrplot` version in the manifest
(currently 0.95), plus `jsonlite` and `png`:

```sh
UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 Rscript scripts/update-legacy-visual-snapshots.R
```

The script writes one 900×900 PNG per case under
`implementations/r/tests/testthat/fixtures/legacy-visual/`. You can also run the
manual **Generate legacy visual snapshots** GitHub Actions workflow and download
the `legacy-visual-snapshots` artifact. Review all images and commit the complete
set together. Snapshot regeneration is deliberately opt-in so a routine test run
cannot silently rewrite approved references.

Until the PNGs are committed, CI retains the existing side-by-side comparison
against `corrplot::corrplot()`. Once all snapshots are committed, the R visual
regression test renders only `corrplotng::corrplot_ng()` and compares its
pixels to the saved baselines, avoiding the second legacy render on each run.
The generator and test use the same device, dimensions, resolution and plot
settings; exact pixel matching is intentional for the pinned CI environment.
