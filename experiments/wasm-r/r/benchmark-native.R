#!/usr/bin/env Rscript
# Native-R reference implementation for the WASM invocation benchmark.
# This deliberately benchmarks the same JSON request contract and validation
# rules, including JSON decoding on every call. It is not a production API.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) {
  stop("usage: Rscript benchmark-native.R FIXTURE.json OUTPUT.json", call. = FALSE)
}
fixture <- normalizePath(args[[1]], mustWork = TRUE)
output_path <- args[[2]]
if (!requireNamespace("jsonlite", quietly = TRUE) ||
    !requireNamespace("digest", quietly = TRUE)) {
  stop("jsonlite and digest are required", call. = FALSE)
}

validate_request <- function(request) {
  fail <- function(code, message, row = NULL, column = NULL) {
    diagnostic <- list(schema_version = 1L, code = code, severity = "error",
      stage = "input", message = message, path = "matrix")
    if (!is.null(row)) diagnostic$row <- row
    if (!is.null(column)) diagnostic$column <- column
    list(valid = FALSE, diagnostic = diagnostic)
  }

  labels <- unlist(request$labels, use.names = FALSE)
  rows <- as.integer(request$rows)
  columns <- as.integer(request$columns)
  values <- unlist(request$values, recursive = TRUE, use.names = FALSE)
  if (!length(labels) || rows == 0L || columns == 0L) {
    return(fail("input.matrix.invalid_shape", "matrix and labels must be non-empty"))
  }
  if (rows != columns || rows != length(labels)) {
    return(fail("input.matrix.invalid_shape",
      sprintf("matrix must be square and match labels: %d labels, %d rows, %d columns",
        length(labels), rows, columns)))
  }
  if (length(values) != rows * columns) {
    return(fail("input.matrix.invalid_shape",
      sprintf("matrix has %d values; expected %d", length(values), rows * columns)))
  }
  for (i in seq_along(labels)) {
    if (!nzchar(trimws(labels[[i]]))) {
      return(fail("input.matrix.invalid_label",
        sprintf("label at index %d must not be empty", i - 1L)))
    }
    if (i > 1L && labels[[i]] %in% labels[seq_len(i - 1L)]) {
      return(fail("input.matrix.invalid_label",
        sprintf("duplicate label '%s'", labels[[i]])))
    }
  }
  matrix_values <- matrix(as.numeric(values), nrow = rows, byrow = TRUE)
  bad <- which(!is.na(matrix_values) & !is.finite(matrix_values), arr.ind = TRUE)
  if (nrow(bad)) {
    r <- bad[1, 1] - 1L
    c <- bad[1, 2] - 1L
    return(fail("input.matrix.non_finite_value",
      sprintf("matrix value at row %d, column %d must be finite or missing", r, c), r, c))
  }

  kind <- as.character(request$kind)
  if (!kind %in% c("correlation", "general_numeric")) {
    return(list(valid = FALSE, diagnostic = list(schema_version = 1L,
      code = "configuration.invalid_field", severity = "error",
      stage = "configuration", message = sprintf("unsupported matrix kind '%s'", kind),
      path = "matrix")))
  }
  if (kind == "correlation") {
    outside <- which(!is.na(matrix_values) &
      (matrix_values < -1 | matrix_values > 1), arr.ind = TRUE)
    if (nrow(outside)) {
      r <- outside[1, 1] - 1L
      c <- outside[1, 2] - 1L
      return(fail("input.matrix.out_of_range",
        sprintf("correlation at row %d, column %d is outside [-1, 1]", r, c), r, c))
    }
    for (r in seq_len(rows)) for (c in seq_len(columns)) {
      a <- matrix_values[r, c]
      b <- matrix_values[c, r]
      symmetric <- if (is.na(a) && is.na(b)) TRUE else if (is.na(a) || is.na(b)) FALSE else abs(a - b) <= 1e-8
      if (!symmetric) {
        return(fail("input.matrix.not_symmetric",
          sprintf("correlation matrix is not symmetric at row %d, column %d", r - 1L, c - 1L), r - 1L, c - 1L))
      }
    }
    for (i in seq_len(rows)) {
      if (is.na(matrix_values[i, i]) || abs(matrix_values[i, i] - 1) > 1e-8) {
        return(fail("input.matrix.invalid_diagonal",
          sprintf("correlation diagonal at index %d must equal 1", i - 1L), i - 1L, i - 1L))
      }
    }
  }
  list(valid = TRUE)
}

request_text <- paste(readLines(fixture, warn = FALSE), collapse = "\n")
calls <- 100L
started <- proc.time()[["elapsed"]]
for (i in seq_len(calls)) {
  request <- jsonlite::fromJSON(request_text, simplifyVector = FALSE)
  response <- validate_request(request)
}
elapsed_ms <- (proc.time()[["elapsed"]] - started) * 1000
result <- list(
  runtime = R.version.string,
  implementation = "native R reference (JSON decode + validation)",
  fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
  calls = calls,
  elapsedMs = elapsed_ms,
  perCallMs = elapsed_ms / calls,
  response = response
)
writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")),
  output_path)
