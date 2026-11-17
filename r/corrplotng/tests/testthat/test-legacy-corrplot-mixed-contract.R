capture_legacy_mixed <- function(draw) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 6)
  device <- grDevices::dev.cur()
  on.exit({
    devices <- grDevices::dev.list()
    if (!is.null(devices) && device %in% devices) {
      grDevices::dev.off(which = device)
    }
    unlink(path)
  }, add = TRUE)

  visible <- withVisible(draw())
  list(visible = visible$visible, value = visible$value,
       plot = grDevices::recordPlot())
}

test_that("corrplot.mixed preserves pinned upper/lower method combinations", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat", "wt")])
  configurations <- list(
    list(lower = "number", upper = "circle", diag = "n", tl.pos = "n"),
    list(lower = "ellipse", upper = "square", diag = "l", tl.pos = "lt"),
    list(lower = "pie", upper = "color", diag = "u", tl.pos = "d"),
    list(lower = "shade", upper = "number", diag = "n", tl.pos = "n"),
    list(lower = "circle", upper = "ellipse", diag = "l", tl.pos = "n")
  )

  for (configuration in configurations) {
    args <- c(list(corr = data, cl.pos = "n"), configuration)
    expected <- capture_legacy_mixed(function() {
      do.call(corrplot::corrplot.mixed, args)
    })
    actual <- capture_legacy_mixed(function() {
      do.call(corrplotng::corrplot.mixed, args)
    })

    context <- paste(configuration, collapse = ", ")
    expect_identical(actual$visible, expected$visible, info = context)
    expect_identical(actual$value, expected$value, info = context)
    expect_identical(actual$plot, expected$plot, info = context)
  }
})

test_that("corrplot.mixed preserves pinned default and color forwarding", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat")])
  configurations <- list(
    list(),
    list(lower.col = "darkred", upper.col = "navy"),
    list(lower = "square", upper = "pie",
         lower.col = c("white", "red"), upper.col = c("blue", "white"))
  )

  for (configuration in configurations) {
    args <- c(list(corr = data, cl.pos = "n", tl.pos = "n"), configuration)
    expected <- capture_legacy_mixed(function() {
      do.call(corrplot::corrplot.mixed, args)
    })
    actual <- capture_legacy_mixed(function() {
      do.call(corrplotng::corrplot.mixed, args)
    })
    expect_identical(actual$visible, expected$visible)
    expect_identical(actual$value, expected$value)
    expect_identical(actual$plot, expected$plot)
  }
})
