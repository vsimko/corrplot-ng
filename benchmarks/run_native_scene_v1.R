args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) stop("usage: run_native_scene_v1.R fixtures.json results.json")
fixtures_path <- args[[1]]
output_path <- args[[2]]

suppressPackageStartupMessages(library(jsonlite))
suppressPackageStartupMessages(library(corrplotngcppspike))
suppressPackageStartupMessages(library(corrplotngrustspike))

fixture <- fromJSON(fixtures_path, simplifyVector = FALSE)
candidates <- list(
  Cpp_Rcpp = build_scene_json_cpp,
  Rust_rextendr = build_scene_json_rust
)

measure_one <- function(fun, labels, matrix, annotation, expected, workload) {
  operation <- function() fun(labels, matrix, annotation)
  first_time <- system.time(first_json <- operation())[["elapsed"]] * 1000
  actual <- fromJSON(first_json, simplifyVector = FALSE)
  comparison <- all.equal(actual, expected, tolerance = 1e-12, check.attributes = FALSE)
  if (!isTRUE(comparison)) {
    stop(workload, ": correctness mismatch: ", paste(comparison, collapse = "; "))
  }
  invisible(lapply(seq_len(3L), function(i) operation()))

  batch <- if (length(labels) == 20L) 500L else 10L
  samples <- numeric(20L)
  for (i in seq_along(samples)) {
    elapsed <- system.time({
      for (j in seq_len(batch)) operation()
    })[["elapsed"]] * 1000
    samples[[i]] <- elapsed / batch
  }
  list(
    status = "passed",
    measurement = "scene_construction_plus_json_encoding",
    first_call_ms = first_time,
    median_ms = median(samples),
    iqr_ms = IQR(samples),
    min_ms = min(samples),
    max_ms = max(samples),
    samples_ms = as.list(samples),
    batch_size = batch,
    scene_cells = length(expected$cells),
    scene_json_bytes = nchar(first_json, type = "bytes")
  )
}

results <- list()
for (case in fixture$workloads) {
  labels <- unlist(case$labels, use.names = FALSE)
  matrix <- do.call(rbind, lapply(case$matrix, function(row) {
    vapply(row, function(value) {
      if (is.null(value)) NA_real_ else as.numeric(value)
    }, numeric(1))
  }))
  annotation <- case$annotation
  expected <- case$expected_scene
  for (candidate_name in names(candidates)) {
    measured <- measure_one(candidates[[candidate_name]], labels, matrix,
                            annotation, expected, case$id)
    measured$workload <- case$id
    measured$candidate <- candidate_name
    results[[length(results) + 1L]] <- measured
  }
}

payload <- list(
  schema_version = 1L,
  suite = "corrplot-ng-cross-language-scene-v1",
  candidate = c("Cpp_Rcpp", "Rust_rextendr"),
  iterations = 20L,
  warmup_iterations = 3L,
  environment = list(
    R = R.version.string,
    platform = R.version$platform,
    os = Sys.info()[["sysname"]],
    machine = Sys.info()[["machine"]],
    cxx_compiler = system2("R", c("CMD", "config", "CXX"), stdout = TRUE),
    rustc = system2("rustc", "--version", stdout = TRUE),
    rcpp_version = as.character(packageVersion("Rcpp")),
    rextendr_version = as.character(packageVersion("rextendr"))
  ),
  results = results
)
dir.create(dirname(output_path), recursive = TRUE, showWarnings = FALSE)
write_json(payload, output_path, pretty = TRUE, auto_unbox = TRUE, digits = NA, null = "null")
