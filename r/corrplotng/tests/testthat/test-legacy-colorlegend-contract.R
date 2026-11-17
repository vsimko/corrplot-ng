capture_colorlegend_contract <- function(draw) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path, width = 7, height = 5)
  device <- grDevices::dev.cur()
  on.exit({
    devices <- grDevices::dev.list()
    if (!is.null(devices) && device %in% devices) {
      grDevices::dev.off(which = device)
    }
    unlink(path)
  }, add = TRUE)

  graphics::plot.new()
  graphics::plot.window(xlim = c(-1, 2), ylim = c(-2, 3))
  visible <- withVisible(draw())
  list(visible = visible$visible, value = visible$value,
       plot = grDevices::recordPlot())
}

test_that("colorlegend matches pinned graphics output for vertical alignment modes", {
  configurations <- list(
    list(align = "c"),
    list(align = "l"),
    list(align = "r")
  )

  for (configuration in configurations) {
    args <- c(list(
      colbar = corrplot::COL1("Blues", 7),
      labels = c("low", "middle", "high"),
      at = c(0, 0.45, 1),
      xlim = c(-0.8, 1.7),
      ylim = c(-1.5, 2.5),
      vertical = TRUE,
      ratio.colbar = 0.32,
      lim.segment = c(0.3, 0.48),
      cex = 0.8,
      col = "navy"
    ), configuration)

    expected <- capture_colorlegend_contract(function() {
      do.call(corrplot::colorlegend, args)
    })
    actual <- capture_colorlegend_contract(function() {
      do.call(corrplotng::colorlegend, args)
    })

    expect_identical(actual$visible, expected$visible)
    expect_identical(actual$value, expected$value)
    expect_identical(actual$plot, expected$plot,
                     info = paste("align:", configuration$align))
  }
})

test_that("colorlegend matches pinned horizontal and no-label graphics output", {
  configurations <- list(
    list(
      vertical = FALSE,
      at = c(0, 0.25, 1),
      labels = c("min", "mid", "max"),
      lim.segment = c(0.35, 0.55),
      ratio.colbar = 0.28
    ),
    list(
      vertical = FALSE,
      at = c(0.2, 0.8),
      labels = c("unused", "labels"),
      addlabels = FALSE,
      lim.segment = NULL,
      ratio.colbar = 0.4
    ),
    list(
      vertical = TRUE,
      at = c(0, 1 / 3, 2 / 3, 1),
      labels = c("one", "two", "three", "four"),
      addlabels = FALSE,
      lim.segment = "auto"
    )
  )

  for (configuration in configurations) {
    args <- c(list(
      colbar = corrplot::COL2("RdBu", 9),
      xlim = c(0.2, 1.8),
      ylim = c(-1, 2),
      lwd = 1.5,
      lend = "butt"
    ), configuration)

    expected <- capture_colorlegend_contract(function() {
      do.call(corrplot::colorlegend, args)
    })
    actual <- capture_colorlegend_contract(function() {
      do.call(corrplotng::colorlegend, args)
    })

    expect_identical(actual$visible, expected$visible)
    expect_identical(actual$value, expected$value)
    expect_identical(actual$plot, expected$plot)
  }
})
