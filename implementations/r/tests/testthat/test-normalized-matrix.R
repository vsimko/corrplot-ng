test_that("numeric matrices normalize to the versioned contract", {
  x <- matrix(c(1, NA_real_, 0, 1), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  normalized <- corrplot_ng_normalize_matrix(x)

  expect_s3_class(normalized, "corrplot_ng_normalized_matrix_v1")
  expect_identical(normalized$schema_version, 1L)
  expect_identical(normalized$kind, "normalized_matrix")
  expect_identical(normalized$labels, c("A", "B"))
  expect_identical(normalized$values, list(list(1, NULL), list(0, 1)))
  expect_identical(normalized$source, list(
    format = "r_matrix",
    row_labels_present = TRUE,
    column_labels_present = TRUE
  ))
  expect_identical(normalized$source_indices, list(0L, 1L))
  expect_null(normalized$metadata)
})

test_that("data frames use column names when default row names are absent", {
  x <- data.frame(A = c(1, 0), B = c(0, 1), check.names = FALSE)
  normalized <- corrplot_ng_normalize_matrix(x)

  expect_identical(normalized$labels, c("A", "B"))
  expect_identical(normalized$source$format, "r_data_frame")
  expect_false(normalized$source$row_labels_present)
  expect_true(normalized$source$column_labels_present)
})

test_that("one supplied name axis is used for both normalized axes", {
  row_named <- diag(2)
  rownames(row_named) <- c("x", "y")
  row_result <- corrplot_ng_normalize_matrix(row_named)
  expect_identical(row_result$labels, c("x", "y"))
  expect_true(row_result$source$row_labels_present)
  expect_false(row_result$source$column_labels_present)

  column_named <- diag(2)
  colnames(column_named) <- c("p", "q")
  column_result <- corrplot_ng_normalize_matrix(column_named)
  expect_identical(column_result$labels, c("p", "q"))
  expect_false(column_result$source$row_labels_present)
  expect_true(column_result$source$column_labels_present)
})

test_that("unnamed matrices receive deterministic generated labels", {
  normalized <- corrplot_ng_normalize_matrix(diag(3))
  expect_identical(normalized$labels, c("V1", "V2", "V3"))
  expect_false(normalized$source$row_labels_present)
  expect_false(normalized$source$column_labels_present)
})

test_that("row and column names must match and labels must be unique and non-empty", {
  mismatch <- diag(2)
  dimnames(mismatch) <- list(c("A", "B"), c("B", "A"))
  expect_error(corrplot_ng_normalize_matrix(mismatch), "row and column labels must match exactly")

  duplicate <- diag(2)
  dimnames(duplicate) <- list(c("A", "A"), c("A", "A"))
  expect_error(corrplot_ng_normalize_matrix(duplicate), "row labels must be unique")

  empty <- diag(2)
  dimnames(empty) <- list(c("A", ""), c("A", ""))
  expect_error(corrplot_ng_normalize_matrix(empty), "row label at index 2 must not be empty")
})

test_that("Unicode labels are preserved exactly", {
  x <- diag(2)
  dimnames(x) <- list(c("Δ temperature", "湿度"), c("Δ temperature", "湿度"))
  normalized <- corrplot_ng_normalize_matrix(x)
  expect_identical(normalized$labels, c("Δ temperature", "湿度"))
})

test_that("non-square and empty inputs are rejected", {
  expect_error(corrplot_ng_normalize_matrix(matrix(1:6, nrow = 2, ncol = 3)),
    "non-empty square matrix")
  expect_error(corrplot_ng_normalize_matrix(matrix(numeric(), nrow = 0, ncol = 0)),
    "non-empty square matrix")
})

test_that("data frames reject non-numeric columns without coercion", {
  x <- data.frame(A = c(1, 2), B = c("3", "4"), check.names = FALSE)
  expect_error(corrplot_ng_normalize_matrix(x), "column 'B'.*must contain integer or double")
  factor_input <- data.frame(A = factor(c("1", "2")), B = c(0, 1))
  expect_error(corrplot_ng_normalize_matrix(factor_input), "column 'A'.*must contain integer or double")
})

test_that("NaN and infinities are rejected with source coordinates", {
  nan <- matrix(c(1, NaN, 0, 1), 2, byrow = TRUE)
  expect_error(corrplot_ng_normalize_matrix(nan), "row 1, column 2 is NaN")

  infinite <- matrix(c(1, Inf, 0, 1), 2, byrow = TRUE)
  expect_error(corrplot_ng_normalize_matrix(infinite), "row 1, column 2 must be finite")
})

test_that("all-missing and constant matrices remain valid normalized inputs", {
  all_missing <- corrplot_ng_normalize_matrix(matrix(NA_real_, 2, 2))
  expect_identical(all_missing$values, list(list(NULL, NULL), list(NULL, NULL)))

  all_zero <- corrplot_ng_normalize_matrix(matrix(0, 2, 2))
  expect_identical(all_zero$values, list(list(0, 0), list(0, 0)))
})

test_that("optional source metadata is preserved and validated", {
  normalized <- corrplot_ng_normalize_matrix(diag(2),
    metadata = list(source_note = "prepared matrix Δ"))
  expect_identical(normalized$metadata, list(source_note = "prepared matrix Δ"))
  expect_error(corrplot_ng_normalize_matrix(diag(2), metadata = list("unnamed")),
    "named list with unique non-empty names")
})

test_that("R adapter output serializes to normalized-matrix v1 JSON", {
  skip_if_not_installed("jsonlite")
  x <- matrix(c(1, NA_real_, 0, 1), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  normalized <- corrplot_ng_normalize_matrix(x)
  encoded <- jsonlite::toJSON(normalized, auto_unbox = FALSE,
    null = "null", na = "null", digits = NA)
  decoded <- jsonlite::fromJSON(encoded, simplifyVector = FALSE)

  expect_identical(decoded$schema_version, 1L)
  expect_identical(decoded$kind, "normalized_matrix")
  expect_identical(decoded$labels, list("A", "B"))
  expect_null(decoded$values[[1]][[2]])
  expect_identical(decoded$values[[2]][[1]], 0)
  expect_identical(decoded$source_indices, list(0L, 1L))
})

test_that("R adapter output matches shared normalized-matrix v1 fixtures", {
  skip_if_not_installed("jsonlite")
  fixture_path <- system.file("extdata", "normalized-matrix-v1",
    "valid-cases.json", package = "corrplotng")
  expect_true(nzchar(fixture_path))
  fixtures <- jsonlite::fromJSON(fixture_path, simplifyVector = FALSE)

  for (case in fixtures$cases) {
    input <- case$input
    values <- do.call(rbind, lapply(input$values, function(row) {
      vapply(row, function(value) if (is.null(value)) NA_real_ else as.numeric(value), numeric(1))
    }))
    dimnames(values) <- list(unlist(input$labels, use.names = FALSE), unlist(input$labels, use.names = FALSE))
    normalized <- corrplot_ng_normalize_matrix(values)
    expect_identical(normalized$labels, unlist(input$labels), info = case$id)
    expect_equal(normalized$values, input$values, info = case$id)
    expect_identical(normalized$source$format, "r_matrix", info = case$id)
    expect_identical(normalized$source_indices, as.list(as.integer(seq_along(input$labels) - 1L)),
      info = case$id)
  }
})
