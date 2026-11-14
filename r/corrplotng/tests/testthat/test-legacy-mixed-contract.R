with_legacy_mixed_pdf <- function(code) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)
  force(code)
}

test_that("legacy corrplot.mixed results match the pinned upstream wrapper", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp", "wt")])

  configurations <- list(
    list(),
    list(lower = "number", upper = "ellipse", tl.pos = "lt", diag = "n"),
    list(lower = "square", upper = "color", tl.pos = "d", diag = "l"),
    list(lower = "circle", upper = "number", tl.pos = "n", diag = "u"),
    list(lower = "shade", upper = "pie", tl.pos = "lt", diag = "n")
  )

  with_legacy_mixed_pdf({
    for (configuration in configurations) {
      expected <- do.call(corrplot::corrplot.mixed, c(list(corr = M), configuration))
      actual <- do.call(corrplotng::corrplot.mixed, c(list(corr = M), configuration))

      expect_identical(actual$corr, expected$corr)
      expect_identical(actual$corrPos, expected$corrPos)
      expect_identical(names(actual), c("corr", "corrPos"))
      expect_true(is.matrix(actual$corr))
      expect_true(is.data.frame(actual$corrPos) || is.matrix(actual$corrPos))
    }
  })
})

test_that("legacy corrplot.mixed restores graphics parameters after drawing", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp")])

  with_legacy_mixed_pdf({
    grDevices::par(mar = c(2, 3, 4, 5), bg = "grey95")
    before <- grDevices::par(c("mar", "bg"))

    result <- corrplotng::corrplot.mixed(
      M,
      lower = "number",
      upper = "circle",
      mar = c(1, 2, 3, 4),
      bg = "white"
    )

    after <- grDevices::par(c("mar", "bg"))
    expect_identical(after, before)
    expect_true(is.list(result))
    expect_named(result, c("corr", "corrPos"))
  })
})
