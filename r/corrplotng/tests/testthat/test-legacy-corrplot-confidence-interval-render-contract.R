capture_legacy_corrplot_ci <- function(fun, args) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(do.call(fun, args))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy corrplot confidence interval overlays match upstream", {
  labels <- c("alpha", "beta", "gamma", "delta")
  M <- matrix(
    c(
      1,  0.3, -0.4,  0.2,
      0.3, 1,   0.6, -0.1,
     -0.4, 0.6,  1,    0.5,
      0.2, -0.1, 0.5,  1
    ),
    nrow = 4,
    dimnames = list(labels, labels)
  )
  low <- matrix(
    c(
      1, -0.2, -0.8, -0.3,
      -0.2, 1,  0.2, -0.6,
      -0.8, 0.2, 1,  0.1,
      -0.3, -0.6, 0.1, 1
    ),
    nrow = 4,
    dimnames = list(labels, labels)
  )
  upp <- matrix(
    c(
      1,  0.7, -0.1,  0.8,
      0.7, 1,   0.9,  0.3,
     -0.1, 0.9, 1,    0.8,
      0.8, 0.3, 0.8,  1
    ),
    nrow = 4,
    dimnames = list(labels, labels)
  )

  configurations <- list(
    list(method = "circle", type = "lower", plotCI = "circle"),
    list(method = "square", type = "upper", plotCI = "square"),
    list(method = "ellipse", type = "full", plotCI = "rect")
  )

  for (configuration in configurations) {
    args <- c(
      list(
        corr = M,
        lowCI.mat = low,
        uppCI.mat = upp,
        diag = FALSE,
        tl.pos = "n",
        cl.pos = "n",
        addgrid.col = "grey50"
      ),
      configuration
    )
    context <- paste(names(configuration), unlist(configuration),
                     collapse = ", ")

    expected <- capture_legacy_corrplot_ci(corrplot::corrplot, args)
    actual <- capture_legacy_corrplot_ci(corrplotng::corrplot, args)
    expect_identical(actual$result, expected$result, info = context)
    expect_identical(actual$plot, expected$plot, info = context)
  }
})

test_that("legacy corrplot requires both confidence interval matrices", {
  M <- diag(3)
  args <- list(corr = M, plotCI = "circle", tl.pos = "n", cl.pos = "n")

  expect_error(corrplot::corrplot, "Need lowCI.mat and uppCI.mat!")
  expect_error(
    do.call(corrplot::corrplot, args),
    "Need lowCI.mat and uppCI.mat!"
  )
  expect_error(
    do.call(corrplotng::corrplot, args),
    "Need lowCI.mat and uppCI.mat!"
  )
})
