capture_legacy_device_workflow <- function(plotter, M) {
  paths <- c(tempfile(fileext = ".pdf"), tempfile(fileext = ".pdf"))
  devices <- integer()
  on.exit({
    open_devices <- grDevices::dev.list()
    if (!is.null(open_devices)) {
      for (device in rev(devices)) {
        if (device %in% open_devices) {
          grDevices::dev.off(which = device)
          open_devices <- grDevices::dev.list()
          if (is.null(open_devices)) break
        }
      }
    }
    unlink(paths)
  }, add = TRUE)

  for (path in paths) {
    grDevices::pdf(path, width = 7, height = 6, onefile = TRUE)
    devices <- c(devices, grDevices::dev.cur())
  }

  # Draw on the first device while the second remains open. The corrplot
  # function must neither switch devices nor close a device owned by its caller.
  grDevices::dev.set(devices[[1]])
  graphics::plot.new()
  graphics::plot.window(xlim = c(0.5, ncol(M) + 0.5),
                        ylim = c(nrow(M) + 0.5, 0.5))
  graphics::segments(0.5, 0.5, ncol(M) + 0.5, nrow(M) + 0.5,
                     col = "grey85")
  graphics::par(mar = c(3, 4, 2, 1), bg = "grey95")
  before_par <- graphics::par(c("mar", "bg"))
  before_devices <- grDevices::dev.list()
  before_device <- grDevices::dev.cur()

  result <- withVisible(plotter(M, method = "circle", tl.pos = "n",
                                cl.pos = "n", mar = c(1, 2, 3, 4)))

  expect_identical(grDevices::dev.cur(), before_device)
  expect_identical(grDevices::dev.list(), before_devices)
  expect_equal(graphics::par(c("mar", "bg")), before_par)

  # Continue composing on the same caller-owned page after corrplot returns.
  graphics::title(main = "post-corrplot composition")
  graphics::points(1, 1, pch = 4, col = "red")
  list(
    result = result,
    plot = grDevices::recordPlot(),
    device = grDevices::dev.cur(),
    devices = grDevices::dev.list(),
    par = graphics::par(c("mar", "bg"))
  )
}

test_that("legacy corrplot preserves caller-owned devices and page composition", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp")])

  upstream <- capture_legacy_device_workflow(corrplot::corrplot, M)
  package <- capture_legacy_device_workflow(corrplotng::corrplot, M)

  expect_identical(package$result$visible, upstream$result$visible)
  expect_identical(package$result$value, upstream$result$value)
  expect_identical(package$plot, upstream$plot)
  expect_identical(package$device, upstream$device)
  expect_length(package$devices, 2L)
  expect_length(upstream$devices, 2L)
  expect_equal(package$par, upstream$par)
})

test_that("legacy corrplot errors do not take ownership of the active graphics device", {
  M <- stats::cor(datasets::mtcars[, c("mpg", "cyl", "disp", "hp")])
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  device <- grDevices::dev.cur()
  devices <- grDevices::dev.list()
  on.exit({
    open_devices <- grDevices::dev.list()
    if (!is.null(open_devices) && device %in% open_devices) {
      grDevices::dev.off(which = device)
    }
    unlink(path)
  }, add = TRUE)

  graphics::plot.new()
  graphics::plot.window(xlim = c(0, 1), ylim = c(0, 1))
  before_par <- graphics::par(c("mar", "bg", "usr"))

  expect_error(
    corrplotng::corrplot(M, tl.pos = "not-a-valid-position"),
    regexp = "."
  )

  expect_identical(grDevices::dev.cur(), device)
  expect_identical(grDevices::dev.list(), devices)
  expect_equal(graphics::par(c("mar", "bg")), before_par)
})
