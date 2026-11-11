test_that("colorlegend exports the corrplot 0.95 formal interface", {
  expect_true("colorlegend" %in% getNamespaceExports("corrplotng"))
  expect_identical(names(formals(corrplotng::colorlegend)),
                   names(formals(corrplot::colorlegend)))
  expect_identical(lapply(formals(corrplotng::colorlegend), deparse),
                   lapply(formals(corrplot::colorlegend), deparse))
})

test_that("colorlegend draws vertical and horizontal legends", {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({ grDevices::dev.off(); unlink(path) }, add = TRUE)
  graphics::plot.new()
  graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
  expect_invisible(colorlegend(grDevices::rainbow(5), seq(-1, 1, length.out = 5),
    align = "l", cex = 0.7))
  graphics::plot.new()
  graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
  expect_invisible(colorlegend(grDevices::rainbow(5), seq(-1, 1, length.out = 5),
    vertical = FALSE, at = seq(0, 1, length.out = 5),
    lim.segment = c(0.3, 0.5), cex = 0.7))
})

test_that("colorlegend supports hidden labels and automatic segment placement", {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({ grDevices::dev.off(); unlink(path) }, add = TRUE)
  graphics::plot.new()
  graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
  expect_invisible(colorlegend(c("red", "white", "blue"), c("low", "mid", "high"),
    addlabels = FALSE, at = NULL, lim.segment = NULL))
})

test_that("colorlegend validates normalized positions and geometry", {
  expect_error(colorlegend(c("red", "blue"), c("low", "high"), at = c(-0.1, 1)), "between 0 and 1")
  expect_error(colorlegend(c("red", "blue"), c("low", "high"), at = c(0, 1),
    lim.segment = c(0.2, 1.1)), "lim.segment")
  expect_error(colorlegend(c("red", "blue"), c("low", "high"), at = c(0, 1),
    lim.segment = 0.5), "lim.segment")
  expect_error(colorlegend(c("red", "blue"), c("low", "high"), at = 0), "same length")
  expect_error(colorlegend(c("invalid-color", "blue"), c("low", "high")), "invalid color")
  expect_error(colorlegend(c("red", "blue"), c("low", "high"), ratio.colbar = 0), "ratio.colbar")
})
