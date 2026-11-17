capture_legacy_corrplot_labels <- function(fun, args) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(do.call(fun, args))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy corrplot plotmath labels and percentage coefficients match upstream", {
  labels <- c(":alpha + beta", "=gamma[2]", "$delta^2", "plain")
  M <- matrix(
    c(
      1, 0.314159, -0.456789, 0.234567,
      0.314159, 1, 0.678901, -0.123456,
      -0.456789, 0.678901, 1, 0.543210,
      0.234567, -0.123456, 0.543210, 1
    ),
    nrow = 4,
    dimnames = list(labels, labels)
  )

  configurations <- list(
    list(method = "circle", type = "full", tl.pos = "lt",
         tl.col = "navy", tl.srt = 0),
    list(method = "ellipse", type = "lower", diag = FALSE, tl.pos = "ld",
         tl.col = "darkgreen", tl.srt = 45),
    list(method = "square", type = "upper", tl.pos = "td",
         tl.col = "purple", tl.srt = 90),
    list(method = "number", addCoef.col = "black",
         addCoefasPercent = TRUE, number.digits = 0),
    list(method = "circle", addCoef.col = "navy",
         addCoefasPercent = TRUE, number.digits = 1)
  )

  for (configuration in configurations) {
    args <- c(
      list(corr = M, cl.pos = "n"),
      configuration
    )
    context <- paste(names(configuration), unlist(configuration),
                     collapse = ", ")

    expected <- capture_legacy_corrplot_labels(corrplot::corrplot, args)
    actual <- capture_legacy_corrplot_labels(corrplotng::corrplot, args)
    expect_identical(actual$result, expected$result, info = context)
    expect_identical(actual$plot, expected$plot, info = context)
  }
})
