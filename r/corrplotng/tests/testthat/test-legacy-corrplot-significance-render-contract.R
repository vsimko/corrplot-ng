capture_legacy_corrplot_significance <- function(fun, args) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(do.call(fun, args))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy corrplot significance annotations match upstream", {
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
  P <- matrix(
    c(
      0,     0.001, 0.01,  0.025,
      0.001, 0,     0.05,  0.075,
      0.01,  0.05,  0,     0.1,
      0.025, 0.075, 0.1,   0
    ),
    nrow = 4,
    dimnames = list(labels, labels)
  )

  configurations <- list(
    list(insig = "pch", sig.level = 0.05, pch = 4,
         pch.col = "navy", pch.cex = 1.4),
    list(insig = "p-value", sig.level = 0.05, number.digits = 3,
         pch.col = "darkgreen"),
    list(insig = "blank", sig.level = 0.05,
         addgrid.col = "grey40"),
    list(insig = "n", sig.level = 0.05),
    list(insig = "label_sig", sig.level = c(0.01, 0.05, 0.1),
         pch = "*", pch.col = "purple", pch.cex = 1.2)
  )

  for (i in seq_along(configurations)) {
    configuration <- configurations[[i]]
    args <- c(
      list(
        corr = M,
        p.mat = P,
        method = if (i %% 2L) "circle" else "square",
        type = if (i %% 2L) "lower" else "upper",
        diag = FALSE,
        tl.pos = "n",
        cl.pos = "n"
      ),
      configuration
    )
    context <- paste(names(configuration), unlist(configuration),
                     collapse = ", ")

    expected <- capture_legacy_corrplot_significance(corrplot::corrplot, args)
    actual <- capture_legacy_corrplot_significance(corrplotng::corrplot, args)
    expect_identical(actual$result, expected$result, info = context)
    expect_identical(actual$plot, expected$plot, info = context)
  }
})
