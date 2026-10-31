args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) stop("usage: test_native_scene_v1.R fixtures.json")
fixtures_path <- args[[1]]

suppressPackageStartupMessages(library(jsonlite))
suppressPackageStartupMessages(library(corrplotngcppspike))
suppressPackageStartupMessages(library(corrplotngrustspike))
source("benchmarks/scene_v1_reference.R")

same_scene <- function(value, expected) {
  comparison <- all.equal(value, expected, tolerance = 1e-12,
                          check.attributes = FALSE)
  if (isTRUE(comparison)) TRUE else comparison
}

same_json_scene <- function(value, expected) {
  if (!is.character(value) || length(value) != 1L || is.na(value)) {
    return("candidate did not return one JSON string")
  }
  parsed <- tryCatch(
    fromJSON(value, simplifyVector = FALSE),
    error = function(error) paste("invalid JSON:", conditionMessage(error))
  )
  if (is.character(parsed) && length(parsed) == 1L &&
      startsWith(parsed, "invalid JSON:")) return(parsed)
  same_scene(parsed, expected)
}

fixture <- fromJSON(fixtures_path, simplifyVector = FALSE)
candidates <- list(
  R_pure = function(labels, matrix, annotation) {
    build_scene_json_r(labels, matrix, annotation)
  },
  Cpp_Rcpp = function(labels, matrix, annotation) {
    build_scene_json_cpp(labels, matrix, annotation)
  },
  Rust_rextendr = function(labels, matrix, annotation) {
    build_scene_json_rust(labels, matrix, annotation)
  }
)

for (case in fixture$workloads) {
  labels <- unlist(case$labels, use.names = FALSE)
  matrix <- do.call(rbind, lapply(case$matrix, function(row) {
    vapply(row, function(value) {
      if (is.null(value)) NA_real_ else as.numeric(value)
    }, numeric(1))
  }))
  for (candidate_name in names(candidates)) {
    actual <- candidates[[candidate_name]](labels, matrix, case$annotation)
    comparison <- same_json_scene(actual, case$expected_scene)
    if (!isTRUE(comparison)) {
      stop(case$id, " / ", candidate_name,
           ": semantic mismatch: ", paste(comparison, collapse = "; "))
    }
    message("PASS ", case$id, " / ", candidate_name)
  }
}

# Invalid inputs must fail consistently across all R-facing implementations.
# The messages are part of this narrow prototype contract, not a general R API.
expect_rejection <- function(candidate_name, candidate, labels, matrix, pattern) {
  error_message <- tryCatch({
    candidate(labels, matrix, "")
    NA_character_
  }, error = function(error) conditionMessage(error))
  if (is.na(error_message)) {
    stop(candidate_name, ": invalid input was accepted")
  }
  if (!grepl(pattern, error_message, fixed = TRUE)) {
    stop(candidate_name, ": unexpected error message: ", error_message)
  }
  message("PASS invalid input / ", candidate_name, " / ", pattern)
}

invalid_cases <- list(
  list(
    name = "dimensions",
    labels = c("a", "b"),
    matrix = matrix(1, nrow = 1L, ncol = 2L),
    message = "labels and matrix dimensions must agree and be non-empty"
  ),
  list(
    name = "range",
    labels = c("a", "b"),
    matrix = matrix(c(1, 1.1, 1.1, 1), nrow = 2L),
    message = "matrix values must be finite, NA, or between -1 and 1"
  ),
  list(
    name = "non-finite",
    labels = c("a", "b"),
    matrix = matrix(c(1, Inf, Inf, 1), nrow = 2L),
    message = "matrix values must be finite, NA, or between -1 and 1"
  ),
  list(
    name = "nan-is-not-missing",
    labels = c("a", "b"),
    matrix = matrix(c(1, NaN, NaN, 1), nrow = 2L),
    message = "matrix values must be finite, NA, or between -1 and 1"
  )
)

for (invalid in invalid_cases) {
  for (candidate_name in names(candidates)) {
    expect_rejection(
      candidate_name, candidates[[candidate_name]],
      invalid$labels, invalid$matrix, invalid$message
    )
  }
}

message("All scene-v1 parity and invalid-input tests passed on ", R.version$platform)
