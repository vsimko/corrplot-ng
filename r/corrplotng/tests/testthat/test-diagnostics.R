test_that("normalization errors expose stable diagnostic-v1 fields", {
  x <- matrix(
    c(1, 0.5, 0.5, 1),
    nrow = 2,
    dimnames = list(c("alpha", "beta"), c("alpha", "gamma"))
  )
  error <- tryCatch(
    corrplot_ng_normalize_matrix(x),
    corrplot_ng_error = function(condition) condition
  )

  expect_s3_class(error, "corrplot_ng_error")
  diagnostic <- corrplot_ng_diagnostic(error)
  expect_equal(diagnostic$schema_version, 1L)
  expect_equal(diagnostic$code, "input.matrix.invalid_label")
  expect_equal(diagnostic$severity, "error")
  expect_equal(diagnostic$stage, "input")
  expect_equal(diagnostic$path, "column_labels")
})

test_that("non-finite matrix diagnostics use zero-based cell coordinates", {
  x <- matrix(c(1, 1, Inf, 1), nrow = 2, byrow = TRUE)
  error <- tryCatch(
    corrplot_ng_normalize_matrix(x),
    corrplot_ng_error = function(condition) condition
  )
  diagnostic <- corrplot_ng_diagnostic(error)
  expect_equal(diagnostic$code, "input.matrix.non_finite_value")
  expect_equal(diagnostic$row, 1L)
  expect_equal(diagnostic$column, 0L)
})

test_that("diagnostic reader ignores unrelated conditions", {
  expect_null(corrplot_ng_diagnostic(simpleError("ordinary error")))
})

test_that("warning conditions expose the same diagnostic-v1 contract", {
  captured <- NULL
  withCallingHandlers(
    .corrplot_ng_warn(
      "configuration.option.deprecated",
      "The legacy option is deprecated; use the replacement option.",
      "configuration",
      path = "legacy_option"
    ),
    warning = function(condition) {
      captured <<- condition
      invokeRestart("muffleWarning")
    }
  )

  expect_s3_class(captured, "corrplot_ng_warning")
  diagnostic <- corrplot_ng_diagnostic(captured)
  expect_equal(diagnostic$schema_version, 1L)
  expect_equal(diagnostic$code, "configuration.option.deprecated")
  expect_equal(diagnostic$severity, "warning")
  expect_equal(diagnostic$stage, "configuration")
  expect_equal(diagnostic$path, "legacy_option")
})

test_that("non-square matrices share the diagnostic-v1 validation contract", {
  error <- tryCatch(
    corrplot_ng_normalize_matrix(matrix(c(1, 0.5), nrow = 2, ncol = 1)),
    corrplot_ng_error = function(condition) condition
  )
  diagnostic <- corrplot_ng_diagnostic(error)
  expect_equal(diagnostic$schema_version, 1L)
  expect_equal(diagnostic$code, "input.matrix.invalid_shape")
  expect_equal(diagnostic$severity, "error")
  expect_equal(diagnostic$stage, "input")
  expect_equal(diagnostic$path, "matrix")
})

test_that("non-numeric matrix diagnostics use the shared diagnostic-v1 contract", {
  error <- tryCatch(
    corrplot_ng_normalize_matrix(matrix(c("1", "not-a-number", "0.5", "1"), nrow = 2)),
    corrplot_ng_error = function(condition) condition
  )
  diagnostic <- corrplot_ng_diagnostic(error)
  expect_equal(diagnostic$code, "input.matrix.non_numeric")
  expect_equal(diagnostic$severity, "error")
  expect_equal(diagnostic$stage, "input")
  expect_equal(diagnostic$path, "matrix")
})

test_that("unsupported scene cell methods expose renderer diagnostic-v1 fields", {
  x <- matrix(c(1, 0.5, 0.5, 1), nrow = 2)
  error <- tryCatch(
    corrplot_ng(x, engine = "scene", method = "pie"),
    corrplot_ng_error = function(condition) condition
  )

  expect_s3_class(error, "corrplot_ng_error")
  diagnostic <- corrplot_ng_diagnostic(error)
  expect_equal(diagnostic$schema_version, 1L)
  expect_equal(diagnostic$code, "renderer.unsupported_feature")
  expect_equal(diagnostic$severity, "error")
  expect_equal(diagnostic$stage, "renderer")
  expect_equal(diagnostic$path, "method")
})
