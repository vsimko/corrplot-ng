capture_legacy_corrplot_text_plot <- function(fun, args) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(do.call(fun, args))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy corrplot coefficient formatting and text styling match upstream", {
  M <- matrix(
    c(
      1,  0.314159, -0.456789,  0.234567,
      0.314159, 1,  0.678901, -0.123456,
     -0.456789, 0.678901, 1,  0.543210,
      0.234567, -0.123456, 0.543210, 1
    ),
    nrow = 4,
    dimnames = list(
      c("alpha", "beta", "gamma", "delta"),
      c("alpha", "beta", "gamma", "delta")
    )
  )

  configurations <- list(
    list(method = "number", number.digits = 0),
    list(method = "number", number.digits = 2, number.cex = 0.7,
         number.font = 2),
    list(method = "number", number.digits = 4, number.cex = 1.1,
         number.font = 3),
    list(method = "circle", addCoef.col = "black", number.digits = 3),
    list(method = "square", addCoef.col = "navy", number.digits = 1,
         number.cex = 0.8, number.font = 4),
    list(method = "ellipse", tl.col = "navy", tl.srt = 45,
         tl.cex = 0.8, tl.offset = 0.3, tl.pos = "lt"),
    list(method = "circle", tl.col = "darkgreen", tl.srt = 90,
         tl.cex = 1.1, tl.offset = 0.5, tl.pos = "d")
  )

  for (configuration in configurations) {
    args <- c(
      list(corr = M, type = "full", diag = TRUE, cl.pos = "n"),
      configuration
    )
    context <- paste(names(configuration), unlist(configuration),
                     collapse = ", ")

    expected <- capture_legacy_corrplot_text_plot(corrplot::corrplot, args)
    actual <- capture_legacy_corrplot_text_plot(corrplotng::corrplotng, args)
    expect_identical(actual$result, expected$result, info = context)
    expect_identical(actual$plot, expected$plot, info = context)
  }
})
