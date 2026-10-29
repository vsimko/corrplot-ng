legacy_visual_cases <- local({
  M <- stats::cor(datasets::mtcars)
  pmat <- 1 - abs(M)^0.2
  ci <- corrplot::cor.mtest(datasets::mtcars, conf.level = 0.95)

  list(
    "default-circle" = function(plotter) {
      plotter(M)
    },
    "method-number" = function(plotter) {
      plotter(M, method = "number")
    },
    "method-square" = function(plotter) {
      plotter(M, method = "square", order = "AOE")
    },
    "method-ellipse" = function(plotter) {
      plotter(M, method = "ellipse", order = "AOE")
    },
    "method-shade" = function(plotter) {
      plotter(M, method = "shade", order = "AOE", diag = FALSE)
    },
    "method-color" = function(plotter) {
      plotter(M, method = "color", order = "AOE", addCoef.col = "grey30")
    },
    "method-pie" = function(plotter) {
      plotter(M, method = "pie", order = "AOE")
    },
    "order-aoe-upper" = function(plotter) {
      plotter(M, order = "AOE", type = "upper")
    },
    "lower-with-coefficients" = function(plotter) {
      plotter(
        M, type = "lower", order = "hclust", diag = FALSE,
        addCoef.col = "black", number.cex = 0.8
      )
    },
    "mixed-circle-ellipse" = function(plotter) {
      plotter(M, type = "upper", method = "circle", order = "AOE", tl.pos = "d")
      plotter(
        M, type = "lower", method = "ellipse", order = "AOE",
        diag = FALSE, tl.pos = "n", cl.pos = "n", add = TRUE
      )
    },
    "missing-values" = function(plotter) {
      with_na <- M
      diag(with_na) <- NA_real_
      with_na[4, 2] <- NA_real_
      plotter(with_na, na.label = "NA", na.label.col = "black")
    },
    "significance" = function(plotter) {
      plotter(
        M, p.mat = pmat, method = "circle", type = "lower",
        insig = "blank", addCoef.col = "black", order = "AOE", diag = FALSE
      )
    },
    "confidence-interval" = function(plotter) {
      plotter(
        M, low = ci$lowCI, upp = ci$uppCI,
        plotCI = "circle", cl.pos = "n"
      )
    },
    "non-correlation-matrix" = function(plotter) {
      N <- matrix(seq_len(16), nrow = 4)
      dimnames(N) <- list(letters[1:4], letters[1:4])
      plotter(
        N, is.corr = FALSE, col.lim = c(1, 16),
        method = "color", tl.pos = "n"
      )
    },
    "math-labels" = function(plotter) {
      labelled <- M[1:5, 1:5]
      colnames(labelled) <- c("alpha", "beta", ":alpha+beta", ": a[0]", "=a[beta]")
      rownames(labelled) <- c("alpha", "beta", NA, "$a[0]", "$ a[beta]")
      plotter(labelled, method = "circle")
    }
  )
})

test_that("legacy visual case manifest matches executable regression cases", {
  manifest_path <- testthat::test_path(
    "..", "..", "..", "..", "tests", "legacy-visual", "cases.json"
  )
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

test_that("corrplot-ng matches the original corrplot visual examples", {
  skip_if_not_installed("png")
  reference_plotter <- function(corr, ...) {
    corrplot::corrplot(corr = corr, ...)
  }
  candidate_plotter <- function(corr, ...) {
    corrplotng::corrplot_ng(corr = corr, ...)
  }

  for (case_id in names(legacy_visual_cases)) {
    reference_file <- tempfile(fileext = ".png")
    candidate_file <- tempfile(fileext = ".png")
    render_legacy_case(legacy_visual_cases[[case_id]], reference_plotter, reference_file)
    render_legacy_case(legacy_visual_cases[[case_id]], candidate_plotter, candidate_file)

    reference_pixels <- png::readPNG(reference_file)
    candidate_pixels <- png::readPNG(candidate_file)
    expect_identical(
      dim(candidate_pixels),
      dim(reference_pixels),
      info = paste("PNG dimensions differ for legacy case:", case_id)
    )
    expect_equal(
      candidate_pixels,
      reference_pixels,
      tolerance = 0,
      info = paste("Rendered pixels differ for legacy case:", case_id)
    )
  }
})
