test_that("the provisional adapter delegates to the legacy corrplot package", {
  corr <- matrix(
    c(1, 0.6, 0.6, 1),
    nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B"))
  )

  for (method in c("circle", "square", "ellipse", "number", "shade", "color", "pie")) {
    path <- tempfile(fileext = ".pdf")
    grDevices::pdf(path)
    tryCatch(
      expect_no_error(
        corrplot_ng(
          corr,
          method = method,
          type = "full",
          tl.pos = "n",
          cl.pos = "n"
        )
      ),
      finally = {
        grDevices::dev.off()
        unlink(path)
      }
    )
  }
})

test_that("the adapter forwards common plotting arguments", {
  corr <- matrix(
    c(1, -0.4, -0.4, 1),
    nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B"))
  )
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  tryCatch(
    expect_no_error(
      corrplot_ng(
        corr,
        method = "color",
        order = "original",
        addCoef.col = "black",
        tl.col = "black"
      )
    ),
    finally = {
      grDevices::dev.off()
      unlink(path)
    }
  )
})

test_that("legacy missing-cell labels remain compatible across all seven glyph methods", {
  corr <- matrix(c(1, NA_real_, 0.6, 1), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  for (method in c("circle", "square", "ellipse", "number", "shade", "color", "pie")) {
    path <- tempfile(fileext = ".pdf")
    grDevices::pdf(path)
    tryCatch(
      expect_no_error(corrplot_ng(corr, method = method, na.label = "NA",
        na.label.col = "purple", tl.pos = "n", cl.pos = "n")),
      finally = {
        grDevices::dev.off()
        unlink(path)
      }
    )
  }
})


test_that("legacy add=TRUE draws on the active device and restores graphics parameters", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({
    grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  graphics::plot(1:3, 1:3, type = "n")
  graphics::par(mar = c(4, 5, 2, 1), oma = c(1, 2, 3, 4), xpd = TRUE)
  device_before <- grDevices::dev.cur()
  devices_before <- grDevices::dev.list()
  parameters_before <- graphics::par(c("mar", "oma", "xpd", "mfrow", "pty"))

  expect_no_error(corrplot_ng(corr, add = TRUE, method = "circle",
    tl.pos = "n", cl.pos = "n"))

  expect_identical(grDevices::dev.cur(), device_before)
  expect_identical(grDevices::dev.list(), devices_before)
  expect_equal(graphics::par(c("mar", "oma", "xpd", "mfrow", "pty")),
    parameters_before)
})

test_that("legacy graphics parameters and device selection are restored after errors", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({
    grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  graphics::plot(1:3, 1:3, type = "n")
  graphics::par(mar = c(3, 4, 5, 2), xpd = FALSE)
  device_before <- grDevices::dev.cur()
  devices_before <- grDevices::dev.list()
  parameters_before <- graphics::par(c("mar", "oma", "xpd", "mfrow", "pty"))

  expect_error(corrplot_ng(corr, add = TRUE, method = "not-a-method"),
    "should be one of")
  expect_identical(grDevices::dev.cur(), device_before)
  expect_identical(grDevices::dev.list(), devices_before)
  expect_equal(graphics::par(c("mar", "oma", "xpd", "mfrow", "pty")),
    parameters_before)
})

test_that("nested legacy overlays leave the caller's device and parameters intact", {
  corr <- matrix(c(1, -0.4, -0.4, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({
    grDevices::dev.off()
    unlink(path)
  }, add = TRUE)

  graphics::plot(1:3, 1:3, type = "n")
  device_before <- grDevices::dev.cur()
  parameters_before <- graphics::par(c("mar", "oma", "xpd"))

  corrplot_ng(corr, add = TRUE, method = "circle", tl.pos = "n", cl.pos = "n")
  corrplot_ng(corr, add = TRUE, method = "number", tl.pos = "n", cl.pos = "n")

  expect_identical(grDevices::dev.cur(), device_before)
  expect_equal(graphics::par(c("mar", "oma", "xpd")), parameters_before)
})
