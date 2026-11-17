capture_legacy_rectangle_plot <- function(draw) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(draw())
  list(result = result, plot = grDevices::recordPlot())
}

legacy_rectangle_matrix <- function() {
  labels <- c("alpha", "beta", "gamma", "delta", "epsilon")
  matrix(
    c(
      1,  0.2, -0.3,  0.4,  0.1,
      0.2, 1,    0.5, -0.2,  0.3,
     -0.3, 0.5,  1,    0.6, -0.4,
      0.4, -0.2, 0.6,  1,    0.7,
      0.1, 0.3, -0.4,  0.7,  1
    ),
    nrow = 5,
    dimnames = list(labels, labels)
  )
}

capture_corrplot_with_rectangle <- function(plotter, rectangle, args) {
  capture_legacy_rectangle_plot(function() {
    result <- do.call(plotter, args)
    rectangle(result)
  })
}

test_that("legacy corrRect index and name rectangles match the pinned oracle", {
  M <- legacy_rectangle_matrix()
  configurations <- list(
    list(index = c(1, 3, 5), col = "steelblue", lwd = 2),
    list(name = c("alpha", "gamma", "epsilon"), col = "firebrick", lwd = 3)
  )

  for (type in c("full", "upper", "lower")) {
    args <- list(
      corr = M, type = type, method = "circle",
      diag = FALSE, tl.pos = "n", cl.pos = "n"
    )

    for (configuration in configurations) {
      draw <- function(plotter, rect_fun) {
        capture_corrplot_with_rectangle(
          plotter,
          function(result) do.call(rect_fun, c(list(corrRes = result),
                                                configuration)),
          args
        )
      }

      expected <- draw(corrplot::corrplot, corrplot::corrRect)
      actual <- draw(corrplotng::corrplot, corrplotng::corrRect)
      context <- paste("type:", type, "input:", names(configuration)[[1]])
      expect_identical(actual$result$visible, expected$result$visible,
                       info = context)
      expect_identical(actual$result$value, expected$result$value,
                       info = context)
      expect_identical(actual$plot, expected$plot, info = context)
    }
  }
})

test_that("legacy corrRect namesMat rectangles match the pinned oracle", {
  M <- legacy_rectangle_matrix()
  configurations <- list(
    c("alpha", "alpha", "gamma", "gamma"),
    rbind(
      c("alpha", "alpha", "gamma", "gamma"),
      c("beta", "beta", "delta", "delta")
    )
  )

  for (names_mat in configurations) {
    draw <- function(plotter, rect_fun) {
      capture_corrplot_with_rectangle(
        plotter,
        function(result) rect_fun(
          corrRes = result, namesMat = names_mat,
          col = "darkgreen", lwd = 2
        ),
        list(corr = M, method = "circle", tl.pos = "n", cl.pos = "n")
      )
    }

    expected <- draw(corrplot::corrplot, corrplot::corrRect)
    actual <- draw(corrplotng::corrplot, corrplotng::corrRect)
    expect_identical(actual$result$value, expected$result$value)
    expect_identical(actual$plot, expected$plot)
  }
})

test_that("legacy corrRect preserves validation errors", {
  M <- legacy_rectangle_matrix()
  result <- corrplotng::corrplot(M, tl.pos = "n", cl.pos = "n")

  expect_error(
    corrplotng::corrRect(corrRes = result, index = c(1, 3), name = c("a", "c")),
    "You should just input one of index, name and namesMat!"
  )
  expect_error(
    corrplotng::corrRect(corrRes = NULL, index = c(1, 3)),
    "List 'corrRes' must be inputted!"
  )
  expect_error(
    corrplotng::corrRect(corrRes = result, name = "not-a-variable"),
    "Non-existent name found!"
  )
})

test_that("legacy corrRect.hclust matches all pinned linkage methods", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp",
                                       "drat", "wt", "qsec", "gear")])
  methods <- c(
    "complete", "ward", "ward.D", "ward.D2", "single",
    "average", "mcquitty", "median", "centroid"
  )

  for (method in methods) {
    draw <- function(rect_fun) {
      capture_legacy_rectangle_plot(function() {
        graphics::plot.new()
        graphics::plot.window(
          xlim = c(0.5, ncol(M) + 0.5),
          ylim = c(nrow(M) + 0.5, 0.5)
        )
        do.call(rect_fun, list(
          corr = M, k = 3, col = "purple", lwd = 2, method = method
        ))
      })
    }

    expected <- draw(corrplot::corrRect.hclust)
    actual <- draw(corrplotng::corrRect.hclust)
    expect_identical(actual$result$visible, expected$result$visible,
                     info = paste("method:", method))
    expect_identical(actual$plot, expected$plot,
                     info = paste("method:", method))
  }
})
