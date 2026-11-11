test_that("legacy visual case manifest matches executable regression cases", {
  manifest_path <- testthat::test_path("fixtures", "legacy-visual-cases.json")
  expect_true(file.exists(manifest_path), info = "Shared legacy visual manifest not found")
  manifest <- jsonlite::fromJSON(manifest_path)
  expect_identical(as.character(utils::packageVersion("corrplot")), manifest$reference_version)
  expect_setequal(manifest$cases$id, names(legacy_visual_cases))
})

render_legacy_case <- function(case, plotter, path) {
  grDevices::png(
    filename = path,
    width = 900,
    height = 900,
    units = "px",
    res = 120,
    bg = "white",
    type = "cairo"
  )
  on.exit(grDevices::dev.off(), add = TRUE)
  graphics::par(mar = c(2, 2, 2, 2), xaxs = "i", yaxs = "i")
  case(plotter)
  invisible(path)
}

test_that("corrplot-ng matches the saved legacy visual examples", {
  skip_if_not_installed("png")
  snapshot_dir <- testthat::test_path("fixtures", "legacy-visual")
  snapshot_files <- file.path(snapshot_dir, paste0(names(legacy_visual_cases), ".png"))
  expect_true(
    all(file.exists(snapshot_files)),
    info = "Legacy PNG snapshots are incomplete; regenerate and commit all 15 baselines."
  )

  candidate_plotter <- function(corr, ...) {
    corrplotng::corrplot_ng(corr = corr, ...)
  }

  for (case_id in names(legacy_visual_cases)) {
    candidate_file <- tempfile(fileext = ".png")
    render_legacy_case(legacy_visual_cases[[case_id]], candidate_plotter, candidate_file)
    expected_pixels <- png::readPNG(file.path(snapshot_dir, paste0(case_id, ".png")))
    candidate_pixels <- png::readPNG(candidate_file)
    expect_identical(
      dim(candidate_pixels),
      dim(expected_pixels),
      info = paste("PNG dimensions differ for legacy case:", case_id)
    )
    expect_equal(
      candidate_pixels,
      expected_pixels,
      tolerance = 0,
      info = paste("Rendered pixels differ from saved legacy baseline:", case_id)
    )
  }
})
