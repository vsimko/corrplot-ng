capture_legacy_corrplot <- function(draw) {
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
  list(
    visible = visible$visible,
    value = visible$value,
    plot = grDevices::recordPlot()
  )
}

test_that("corrplot preserves pinned glyph and triangle behavior", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat", "wt")])
  methods <- c("circle", "square", "ellipse", "number", "shade", "color", "pie")
  types <- c("full", "upper", "lower")

  for (method in methods) {
    for (type in types) {
      args <- list(
        corr = data,
        method = method,
        type = type,
        diag = FALSE,
        tl.pos = "n",
        cl.pos = "n",
        addCoef.col = "black",
        number.digits = 2
      )
      expected <- capture_legacy_corrplot(function() {
        do.call(corrplot::corrplot, args)
      })
      actual <- capture_legacy_corrplot(function() {
        do.call(corrplotng::corrplot, args)
      })

      context <- paste("method =", method, "type =", type)
      expect_identical(actual$visible, expected$visible, info = context)
      expect_identical(actual$value, expected$value, info = context)
      expect_identical(actual$plot, expected$plot, info = context)
    }
  }
})

test_that("corrplot preserves pinned ordering, labels, and confidence/significance arguments", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat", "wt")])
  p <- matrix(0.01, nrow(data), ncol(data), dimnames = dimnames(data))
  low <- data - 0.1
  high <- data + 0.1

  configurations <- list(
    list(order = "original", tl.pos = "lt", tl.srt = 45),
    list(order = "alphabet", tl.pos = "d", tl.cex = 0.7),
    list(order = "AOE", tl.pos = "n", plotCI = "rect",
         lowCI.mat = low, uppCI.mat = high),
    list(order = "FPC", tl.pos = "n", p.mat = p,
         insig = "pch", pch = 8, pch.cex = 1.5),
    list(order = "hclust", hclust.method = "average",
         tl.pos = "n", p.mat = p, insig = "blank")
  )

  for (configuration in configurations) {
    args <- c(list(
      corr = data,
      method = "circle",
      type = "lower",
      diag = TRUE,
      cl.pos = "n"
    ), configuration)

    expected <- capture_legacy_corrplot(function() {
      do.call(corrplot::corrplot, args)
    })
    actual <- capture_legacy_corrplot(function() {
      do.call(corrplotng::corrplot, args)
    })

    expect_identical(actual$visible, expected$visible)
    expect_identical(actual$value, expected$value)
    expect_identical(actual$plot, expected$plot)
  }
})

test_that("corrplot preserves pinned add=TRUE graphics composition", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat")])

  draw <- function(fun) {
    graphics::plot.new()
    graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
    fun(data, method = "number", type = "upper", add = TRUE,
        tl.pos = "n", cl.pos = "n", diag = FALSE)
    graphics::text(0.5, 0.5, "overlay")
    grDevices::recordPlot()
  }

  expected <- capture_legacy_corrplot(function() draw(corrplot::corrplot))
  actual <- capture_legacy_corrplot(function() draw(corrplotng::corrplot))
  expect_identical(actual$plot, expected$plot)
})
