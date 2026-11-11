test_that("legacy plotting uses the active device and restores temporary graphics parameters", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))

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

  graphics::par(mar = c(3, 4, 2, 1), bg = "grey90")
  before <- graphics::par(c("mar", "bg"))

  expect_no_error(corrplot_ng(
    corr,
    method = "circle",
    tl.pos = "n",
    cl.pos = "n",
    mar = c(1, 2, 3, 4)
  ))

  expect_identical(grDevices::dev.cur(), device)
  expect_identical(grDevices::dev.list(), devices)
  expect_equal(graphics::par(c("mar", "bg")), before)
})

test_that("add=TRUE composes on the existing plot without replacing its device or coordinates", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))

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
  graphics::plot.window(xlim = c(0.5, 2.5), ylim = c(2.5, 0.5), asp = 1)
  graphics::segments(0.5, 1.5, 2.5, 1.5, col = "grey80")
  before_usr <- graphics::par("usr")
  before_par <- graphics::par(c("mar", "bg"))

  expect_no_error(corrplot_ng(
    corr,
    add = TRUE,
    method = "circle",
    tl.pos = "n",
    cl.pos = "n"
  ))

  expect_identical(grDevices::dev.cur(), device)
  expect_identical(grDevices::dev.list(), devices)
  expect_equal(graphics::par("usr"), before_usr)
  expect_equal(graphics::par(c("mar", "bg")), before_par)
})

test_that("legacy graphics parameters are restored when drawing fails after setup", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))

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

  graphics::par(mar = c(2, 3, 4, 5), bg = "grey90")
  before <- graphics::par(c("mar", "bg"))

  expect_error(corrplot_ng(
    corr,
    method = "circle",
    tl.cex = NA_real_,
    tl.pos = "lt",
    cl.pos = "n"
  ))

  expect_identical(grDevices::dev.cur(), device)
  expect_identical(grDevices::dev.list(), devices)
  expect_equal(graphics::par(c("mar", "bg")), before)
})

test_that("scene preparation does not open, close, or mutate a graphics device", {
  corr <- matrix(c(1, 0.6, 0.6, 1), 2,
    dimnames = list(c("A", "B"), c("A", "B")))

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

  before <- graphics::par(c("mar", "bg", "usr"))
  scene <- corrplot_ng(corr, engine = "scene")

  expect_s3_class(scene, "corrplot_ng_scene")
  expect_identical(grDevices::dev.cur(), device)
  expect_identical(grDevices::dev.list(), devices)
  expect_equal(graphics::par(c("mar", "bg", "usr")), before)
})
