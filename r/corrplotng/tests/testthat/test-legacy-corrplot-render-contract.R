capture_legacy_corrplot_render <- function(fun, args) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 7)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  result <- withVisible(do.call(fun, args))
  list(result = result, plot = grDevices::recordPlot())
}

test_that("legacy corrplot glyphs, triangle types, and diagonal visibility match upstream", {
  M <- matrix(
    c(
      1,  0.3, -0.4,  0.2,
      0.3, 1,   0.6, -0.1,
     -0.4, 0.6,  1,    0.5,
      0.2, -0.1, 0.5,  1
    ),
    nrow = 4,
    dimnames = list(
      c("alpha", "beta", "gamma", "delta"),
      c("alpha", "beta", "gamma", "delta")
    )
  )
  methods <- c("circle", "square", "ellipse", "number", "shade", "color", "pie")
  display_types <- c("full", "upper", "lower")

  for (method in methods) {
    for (display_type in display_types) {
      for (show_diagonal in c(TRUE, FALSE)) {
        args <- list(
          corr = M,
          method = method,
          type = display_type,
          diag = show_diagonal,
          tl.pos = "n",
          cl.pos = "n"
        )
        context <- paste(
          "method:", method,
          "type:", display_type,
          "diag:", show_diagonal
        )

        expected <- capture_legacy_corrplot_render(corrplot::corrplot, args)
        actual <- capture_legacy_corrplot_render(corrplotng::corrplot, args)

        expect_identical(actual$result, expected$result, info = context)
        expect_identical(actual$plot, expected$plot, info = context)
      }
    }
  }
})
