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
  x <- matrix(c(1, Inf, Inf, 1), nrow = 2)
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
