capture_legacy_helper_plot <- function(expr) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(force(expr))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy colorlegend drawing matches the pinned upstream helper", {
  palette <- grDevices::hcl.colors(9, "YlOrRd")
  labels <- c("low", "medium", "high")
  configurations <- list(
    list(vertical = TRUE, align = "c"),
    list(vertical = TRUE, align = "l", lim.segment = c(0.35, 0.55)),
    # With labels disabled, upstream still draws tick segments when at is
    # supplied; leaving it NULL creates an empty coordinate vector and errors.
    list(vertical = TRUE, align = "r", addlabels = FALSE, at = c(0, 0.5, 1)),
    list(vertical = FALSE),
    list(vertical = FALSE, at = c(0, 0.4, 1), ratio.colbar = 0.3)
  )

  for (configuration in configurations) {
    draw <- function(fun) {
      graphics::plot.new()
      graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
      do.call(fun, c(
        list(colbar = palette, labels = labels, xlim = c(0, 1), ylim = c(0, 1)),
        configuration
      ))
    }

    expected <- capture_legacy_helper_plot(draw(corrplot::colorlegend))
    actual <- capture_legacy_helper_plot(draw(corrplotng::colorlegend))
    expect_identical(actual$plot, expected$plot)
  }
})

test_that("legacy corrRect index and name selectors match pinned upstream drawing", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp", "wt")])
  configurations <- list(
    list(index = c(1, 3, 5), type = "full"),
    list(index = c(1, 3, 5), type = "upper"),
    list(index = c(1, 3, 5), type = "lower"),
    list(name = c("mpg", "disp", "wt"), type = "full"),
    list(namesMat = c("mpg", "cyl", "disp", "hp"), type = "full")
  )

  for (configuration in configurations) {
    draw <- function(corrplot_fun, corrRect_fun) {
      corr_res <- corrplot_fun(M, type = configuration$type)
      args <- configuration[setdiff(names(configuration), "type")]
      do.call(corrRect_fun, c(list(corrRes = corr_res), args))
    }

    expected <- capture_legacy_helper_plot(
      draw(corrplot::corrplot, corrplot::corrRect)
    )
    actual <- capture_legacy_helper_plot(
      draw(corrplotng::corrplot, corrplotng::corrRect)
    )

    expect_identical(actual$result$visible, FALSE)
    expect_identical(actual$result$value$corr, expected$result$value$corr)
    expect_identical(actual$result$value$corrPos, expected$result$value$corrPos)
    expect_identical(actual$plot, expected$plot)
  }
})

test_that("legacy corrRect.hclust drawing matches every pinned linkage method", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp", "wt")])
  methods <- c(
    "complete", "ward", "ward.D", "ward.D2", "single",
    "average", "mcquitty", "median", "centroid"
  )

  draw <- function(fun, method) {
    graphics::plot.new()
    graphics::plot.window(xlim = c(0.5, nrow(M) + 0.5),
                          ylim = c(0.5, nrow(M) + 0.5))
    fun(M, k = 3, method = method)
  }

  for (method in methods) {
    expected <- capture_legacy_helper_plot(
      draw(corrplot::corrRect.hclust, method)
    )
    actual <- capture_legacy_helper_plot(
      draw(corrplotng::corrRect.hclust, method)
    )

    expect_identical(actual$result, expected$result, info = method)
    expect_identical(actual$plot, expected$plot, info = method)
  }
})
