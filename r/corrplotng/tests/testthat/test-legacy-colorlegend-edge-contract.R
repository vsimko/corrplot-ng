test_that("legacy colorlegend preserves tick, segment, alignment, and sizing options", {
  palette <- grDevices::hcl.colors(11, "Blue-Red 3")
  configurations <- list(
    list(vertical = TRUE, align = "c", at = c(0, 0.25, 0.5, 0.75, 1)),
    list(vertical = TRUE, align = "l", lim.segment = "auto",
         at = c(0, 0.5, 1), ratio.colbar = 0.2),
    list(vertical = TRUE, align = "r", lim.segment = c(0.2, 0.8),
         at = c(0.2, 0.5, 0.8), ratio.colbar = 0.8),
    list(vertical = FALSE, align = "c", lim.segment = "auto",
         at = c(0, 0.5, 1), ratio.colbar = 0.25),
    list(vertical = FALSE, align = "l", lim.segment = c(0.1, 0.9),
         at = c(0.1, 0.5, 0.9), ratio.colbar = 0.6),
    list(vertical = FALSE, align = "r", addlabels = FALSE,
         at = c(0, 0.5, 1), ratio.colbar = 0.4)
  )

  for (configuration in configurations) {
    draw <- function(fun) {
      graphics::plot.new()
      graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
      do.call(fun, c(
        list(
          colbar = palette,
          labels = c("minimum", "middle", "maximum"),
          xlim = c(0.05, 0.95),
          ylim = c(0.1, 0.9)
        ),
        configuration
      ))
    }

    expected <- capture_legacy_helper_plot(draw(corrplot::colorlegend))
    actual <- capture_legacy_helper_plot(draw(corrplotng::colorlegend))
    expect_identical(actual$result$visible, expected$result$visible)
    expect_identical(actual$plot, expected$plot)
  }
})
