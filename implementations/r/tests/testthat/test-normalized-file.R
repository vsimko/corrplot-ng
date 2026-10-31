test_that("CSV file inputs preserve labels, missing values, zero, and provenance", {
  path <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,1,", "B,0,1"), path, useBytes = TRUE)
  normalized <- corrplot_ng_normalize_file(path)

  expect_identical(normalized$labels, c("A", "B"))
  expect_identical(normalized$values, list(list(1, NULL), list(0, 1)))
  expect_identical(normalized$source$format, "csv")
  expect_true(normalized$source$row_labels_present)
  expect_true(normalized$source$column_labels_present)
  expect_identical(normalized$source$path, normalizePath(path, winslash = "/", mustWork = TRUE))
})

test_that("TSV inputs support generated labels when axes are unnamed", {
  path <- tempfile(fileext = ".tsv")
  writeLines(c("1\t0", "0\t1"), path)
  normalized <- corrplot_ng_normalize_file(path, format = "tsv",
    header = FALSE, row_labels = FALSE)

  expect_identical(normalized$labels, c("V1", "V2"))
  expect_identical(normalized$values, list(list(1, 0), list(0, 1)))
  expect_false(normalized$source$row_labels_present)
  expect_false(normalized$source$column_labels_present)
})

test_that("a single supplied CSV label axis is reused for both axes", {
  path <- tempfile(fileext = ".csv")
  writeLines(c("A,B", "1,0", "0,1"), path)
  normalized <- corrplot_ng_normalize_file(path, header = TRUE, row_labels = FALSE)
  expect_identical(normalized$labels, c("A", "B"))
  expect_false(normalized$source$row_labels_present)
  expect_true(normalized$source$column_labels_present)
})

test_that("CSV file labels must match exactly when both axes are present", {
  path <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "B,1,0", "A,0,1"), path)
  expect_error(corrplot_ng_normalize_file(path),
    "row and column labels must match exactly")
})

test_that("file adapters reject non-square, non-numeric, NaN, and infinite values", {
  rectangular <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,1,0", "B,0,1", "C,0,0"), rectangular)
  expect_error(corrplot_ng_normalize_file(rectangular), "square matrix")

  non_numeric <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,1,nope", "B,0,1"), non_numeric)
  expect_error(corrplot_ng_normalize_file(non_numeric), "non-numeric value")

  nan <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,1,NaN", "B,0,1"), nan)
  expect_error(corrplot_ng_normalize_file(nan), "is NaN")

  infinite <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,1,Inf", "B,0,1"), infinite)
  expect_error(corrplot_ng_normalize_file(infinite), "must be finite")
})

test_that("all-missing CSV matrices are valid normalized inputs", {
  path <- tempfile(fileext = ".csv")
  writeLines(c(",A,B", "A,NA,", "B,null,NA"), path)
  normalized <- corrplot_ng_normalize_file(path)
  expect_true(all(vapply(unlist(normalized$values, recursive = FALSE),
    function(row) all(vapply(row, is.null, logical(1))), logical(1))))
})

test_that("normalized-matrix JSON input round-trips with null and zero distinct", {
  skip_if_not_installed("jsonlite")
  path <- tempfile(fileext = ".json")
  json <- paste0(
    '{"schema_version":1,"kind":"normalized_matrix",',
    '"labels":["A","B"],"values":[[1,null],[0,1]],',
    '"source":{"format":"json","row_labels_present":true,',
    '"column_labels_present":true},"source_indices":[0,1]}'
  )
  writeLines(json, path, useBytes = TRUE)
  normalized <- corrplot_ng_normalize_file(path)
  expect_identical(normalized$labels, c("A", "B"))
  expect_identical(normalized$values, list(list(1, NULL), list(0, 1)))
  expect_identical(normalized$source$format, "json")
  expect_equal(normalized$source_indices, list(0, 1))
})
