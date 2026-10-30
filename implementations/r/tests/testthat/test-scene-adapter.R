test_that("the scene engine builds a renderer-neutral scene-v1 object", {
  corr <- matrix(c(1, 0.6, NA_real_, 1), nrow = 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  scene <- corrplot_ng(corr, engine = "scene", annotation = "Example")
  expect_s3_class(scene, "corrplot_ng_scene")
  expect_identical(scene$schema_version, 1L)
  expect_identical(scene$kind, "correlation_plot")
  expect_identical(vapply(scene$labels, function(item) item$text, character(1)), c("A", "B"))
  expect_length(scene$cells, 4L)
  expect_identical(lapply(scene$cells, function(cell) c(cell$row, cell$column)),
    list(c(0L, 0L), c(0L, 1L), c(1L, 0L), c(1L, 1L)))
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


test_that("coefficient labels preserve cell identity and legacy decimal formatting", {
  corr <- matrix(c(1, -0.125, -0.125, NA_real_), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  scene <- corrplot_ng(corr, engine = "scene", addCoef.col = "black")
  labels <- scene$text_annotations
  expect_identical(vapply(labels, `[[`, character(1), "text"),
    c("1.00", "-0.12", "-0.12"))
  expect_true(all(vapply(labels, `[[`, character(1), "role") == "coefficient"))
  expect_true(all(vapply(labels, `[[`, character(1), "color") == "black"))
  expect_identical(lapply(labels, function(item) c(item$row, item$column)),
    list(c(0L, 0L), c(0L, 1L), c(1L, 0L)))
})

test_that("percentage labels and number typography follow corrplot defaults", {
  corr <- matrix(0.456, 1, dimnames = list("A", "A"))
  scene <- corrplot_ng(corr, engine = "scene", addCoef.col = "navy",
    addCoefasPercent = TRUE, number.digits = 1, number.cex = 1.5,
    number.font = 4)
  label <- scene$text_annotations[[1]]
  expect_identical(label$text, "45.6")
  expect_equal(label$font_size, 19.5)
  expect_identical(label$font_weight, "bold")
  expect_identical(label$font_style, "italic")
})


test_that("percentage formatting defaults to whole percentages", {
  scene <- corrplot_ng(matrix(.125, 1), engine = "scene",
    addCoef.col = "black", addCoefasPercent = TRUE)
  expect_identical(scene$text_annotations[[1]]$text, "12")
})

test_that("number method takes precedence over addCoef.col", {
  scene <- corrplot_ng(diag(2), engine = "scene", method = "number",
    addCoef.col = "red", number.digits = 1, number.font = 3)
  expect_identical(scene$glyph_method, "number")
  expect_true(all(vapply(scene$text_annotations, `[[`, character(1), "role") == "value"))
  expect_false(any(vapply(scene$text_annotations, `[[`, character(1), "color") == "red"))
  expect_true(all(vapply(scene$text_annotations, `[[`, character(1), "font_style") == "italic"))
})

test_that("coefficient labels respect triangle and diagonal visibility", {
  corr <- matrix(c(1, .2, .3, .2, 1, .4, .3, .4, 1), 3, byrow = TRUE)
  scene <- corrplot_ng(corr, engine = "scene", addCoef.col = "black",
    type = "upper", diag = FALSE)
  expect_identical(lapply(scene$text_annotations, function(item) c(item$row, item$column)),
    list(c(0L, 1L), c(0L, 2L), c(1L, 2L)))
})

test_that("coefficient formatting arguments are validated", {
  expect_error(corrplot_ng(diag(1), engine = "scene", number.digits = -1), "number.digits")
  expect_error(corrplot_ng(diag(1), engine = "scene", number.digits = 1.5), "number.digits")
  expect_error(corrplot_ng(diag(1), engine = "scene", number.cex = 0), "number.cex")
  expect_error(corrplot_ng(diag(1), engine = "scene", number.font = 5), "number.font")
  expect_error(corrplot_ng(diag(1), engine = "scene", method = "ellipse"), "does not support: method")
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


test_that("coefficient annotation fixtures match pinned corrplot 0.95 semantics", {
  fixture_path <- test_path("fixtures", "coefficient-annotation-cases.json")
  fixture <- jsonlite::fromJSON(fixture_path, simplifyVector = FALSE)
  expect_identical(fixture$reference$version, as.character(utils::packageVersion("corrplot")))
  expect_identical(fixture$reference$commit,
    "62532f700e6c9d04b93ffbf0bf56e264b0e0368e")

  for (case in fixture$cases) {
    corr <- matrix(
      unlist(case$matrix, use.names = FALSE),
      nrow = length(case$labels), byrow = TRUE,
      dimnames = list(unlist(case$labels), unlist(case$labels))
    )
    options <- case$options
    args <- list(engine = "scene")
    if (!is.null(options$addCoef_col)) args$addCoef.col <- options$addCoef_col
    if (!is.null(options$add_coef_as_percent)) args$addCoefasPercent <- options$add_coef_as_percent
    if (!is.null(options$type)) args$type <- options$type
    if (!is.null(options$diag)) args$diag <- options$diag
    if (!is.null(options$method)) args$method <- options$method
    if (!is.null(options$number_digits)) args$number.digits <- options$number_digits

    scene <- do.call(corrplot_ng, c(list(corr = corr), args))
    actual <- lapply(scene$text_annotations, function(item) {
      list(row = item$row, column = item$column, role = item$role, text = item$text)
    })
    expect_equal(actual, case$expected, info = case$id)
  }
})
