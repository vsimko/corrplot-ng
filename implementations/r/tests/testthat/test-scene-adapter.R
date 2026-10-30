test_that("the scene engine builds a renderer-neutral scene-v1 object", {
  corr <- matrix(c(1, 0.6, NA_real_, 1), nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B")))
  scene <- corrplot_ng(corr, engine = "scene", annotation = "Example")
  expect_s3_class(scene, "corrplot_ng_scene")
  expect_identical(scene$schema_version, 1L)
  expect_identical(scene$kind, "correlation_plot")
  expect_identical(vapply(scene$labels, function(item) item$text, character(1)), c("A", "B"))
  expect_length(scene$cells, 4L)
  expect_identical(lapply(scene$cells, function(cell) c(cell$row, cell$column)),
    list(c(0, 0), c(0, 1), c(1, 0), c(1, 1)))
  expect_equal(vapply(scene$cells, function(cell) {
    if (is.null(cell$value)) NA_real_ else cell$value
  }, numeric(1)), c(1, 0.6, NA_real_, 1))
  expect_identical(scene$annotation, "Example")
  expect_identical(scene$style$positive_color, "#2166ac")
})

test_that("the scene engine derives labels when dimnames are absent or partial", {
  scene <- corrplot_ng(diag(2), engine = "scene")
  expect_identical(vapply(scene$labels, function(item) item$text, character(1)), c("1", "2"))
  column_named <- diag(2)
  colnames(column_named) <- c("x", "y")
  scene <- corrplot_ng(column_named, engine = "scene")
  expect_identical(vapply(scene$labels, function(item) item$text, character(1)), c("x", "y"))
})

test_that("the scene engine rejects options it cannot represent", {
  corr <- diag(2)
  expect_error(corrplot_ng(corr, engine = "scene", method = "ellipse"), "does not support: method")
  expect_error(corrplot_ng(corr, engine = "scene", order = "AOE"), "does not support: order")
})

test_that("the scene engine rejects invalid matrix semantics", {
  expect_error(corrplot_ng(matrix(1:4, 2), engine = "scene"), "values in")
  expect_error(corrplot_ng(matrix(1:6, 2, 3), engine = "scene"), "square")
  expect_error(corrplot_ng(matrix(Inf, 1), engine = "scene"), "finite")
  expect_error(corrplot_ng(matrix(c(1, 0, 0, 1), 2,
    dimnames = list(c("A", "B"), c("B", "A"))), engine = "scene"),
    "row and column names must match")
  expect_error(corrplot_ng(diag(2), engine = "scene", annotation = NA_character_), "annotation")
})

test_that("the default engine remains the legacy corrplot implementation", {
  corr <- matrix(c(1, 0.6, 0.6, 1), nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B")))
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  tryCatch(expect_no_error(corrplot_ng(corr, method = "color", order = "original")),
    finally = {
      grDevices::dev.off()
      unlink(path)
    })
})
