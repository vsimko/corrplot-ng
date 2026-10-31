args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) stop("usage: test_native_scene_v1.R fixtures.json")
fixtures_path <- args[[1]]

suppressPackageStartupMessages(library(jsonlite))
suppressPackageStartupMessages(library(corrplotngcppspike))
suppressPackageStartupMessages(library(corrplotngrustspike))
source("benchmarks/scene_v1_reference.R")
for (package in c("corrplotngcppspike", "corrplotngrustspike")) {
  package_root <- system.file(package = package)
  library_files <- list.files(file.path(package_root, "libs"), full.names = TRUE)
  package_bytes <- if (length(library_files)) sum(file.info(library_files)$size) else 0
  message("PACKAGE_SIZE ", package, " ", package_bytes, " bytes")
}

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


for (case in fixture$invalid_cases) {
  labels <- unlist(case$labels, use.names = FALSE)
  matrix <- do.call(rbind, lapply(case$matrix, function(row) {
    vapply(row, function(value) {
      if (is.null(value)) NA_real_
      else if (is.character(value) && identical(value, "NaN")) NaN
      else as.numeric(value)
    }, numeric(1))
  }))
  for (candidate_name in names(candidates)) {
    error_message <- tryCatch({
      candidates[[candidate_name]](labels, matrix, case$annotation)
      NULL
    }, error = function(error) conditionMessage(error))
    if (is.null(error_message)) {
      stop(case$id, " / ", candidate_name, ": invalid input was accepted")
    }
    if (!grepl(case$expected_error_pattern, error_message, ignore.case = TRUE)) {
      stop(case$id, " / ", candidate_name,
           ": unexpected diagnostic: ", error_message)
    }
    message("PASS ", case$id, " / ", candidate_name, " (rejected)")
  }
}

message("All scene-v1 packaging smoke tests passed on ", R.version$platform)
