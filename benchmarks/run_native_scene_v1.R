args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 2L || length(args) > 3L) stop("usage: run_native_scene_v1.R fixtures.json results.json [--stress-mode]")
fixtures_path <- args[[1]]
output_path <- args[[2]]
stress_mode <- length(args) == 3L && identical(args[[3]], "--stress-mode")

suppressPackageStartupMessages(library(jsonlite))
suppressPackageStartupMessages(library(corrplotngcppspike))
suppressPackageStartupMessages(library(corrplotngrustspike))
source("benchmarks/scene_v1_reference.R")

target_sample_ms <- 100
regular_sample_count <- 30L
regular_warmups <- 5L
stress_sample_count <- 5L
stress_warmups <- 1L

measure_one <- function(fun, validate, workload, candidate, measurement,
                        scene_cells, scene_json_bytes, stress = FALSE) {
  sample_count <- if (stress) stress_sample_count else regular_sample_count
  warmups <- if (stress) stress_warmups else regular_warmups
  first_started <- proc.time()[["elapsed"]]
  first_value <- fun()
  first_time <- (proc.time()[["elapsed"]] - first_started) * 1000
  validation <- validate(first_value)
  if (!isTRUE(validation)) {
    stop(workload, " / ", candidate, " / ", measurement,
         ": correctness mismatch: ", paste(validation, collapse = "; "))
  }
  output_bytes <- if (is.character(first_value)) nchar(first_value, type = "bytes") else scene_json_bytes
  invisible(lapply(seq_len(warmups), function(i) fun()))

  batch <- 1L
  repeat {
    if (stress) break
    started <- proc.time()[["elapsed"]]
    for (j in seq_len(batch)) fun()
    elapsed <- (proc.time()[["elapsed"]] - started) * 1000
    if (elapsed >= target_sample_ms) break
    batch <- batch * 2L
  }

  samples <- numeric(sample_count)
  for (i in seq_len(sample_count)) {
    started <- proc.time()[["elapsed"]]
    for (j in seq_len(batch)) fun()
    samples[[i]] <- (proc.time()[["elapsed"]] - started) * 1000 / batch
  }
  list(
    workload = workload, candidate = candidate, status = "passed",
    measurement = measurement, first_call_ms = first_time,
    median_ms = median(samples), iqr_ms = IQR(samples),
    min_ms = min(samples), max_ms = max(samples),
    samples_ms = as.list(samples), batch_size = batch,
    sample_target_ms = target_sample_ms, sample_count = sample_count,
    warmup_iterations = warmups, stress_mode = stress, scene_cells = scene_cells,
    scene_json_bytes = output_bytes
  )
}

same_scene <- function(value, expected) {
  comparison <- all.equal(value, expected, tolerance = 1e-12,
                          check.attributes = FALSE)
  if (isTRUE(comparison)) TRUE else comparison
}
same_json_scene <- function(value, expected) {
  same_scene(fromJSON(value, simplifyVector = FALSE), expected)
}

package_artifact_sizes <- function(package) {
  package_root <- system.file(package = package)
  libs_dir <- file.path(package_root, "libs")
  files <- list.files(libs_dir, full.names = TRUE)
  if (!length(files)) return(list(total_bytes = 0, files = list()))
  sizes <- file.info(files)$size
  list(total_bytes = sum(sizes), files = as.list(setNames(as.numeric(sizes), basename(files))))
}

fixture <- fromJSON(fixtures_path, simplifyVector = FALSE)
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
  cells <- length(expected$cells)
  json_bytes <- NA_integer_
  if (!stress_mode) {
    r_scene <- build_scene_v1_r(labels, matrix, annotation)
    r_json <- encode_scene_v1_r(r_scene)
    if (!isTRUE(same_json_scene(r_json, expected))) {
      stop(case$id, ": pure R scene-v1 output differs from Python oracle")
    }
    json_bytes <- nchar(r_json, type = "bytes")
    results[[length(results) + 1L]] <- measure_one(
      function() build_scene_v1_r(labels, matrix, annotation),
      function(value) same_scene(value, expected),
      case$id, "R_pure", "scene_construction_only", cells, json_bytes
    )
    results[[length(results) + 1L]] <- measure_one(
      function() encode_scene_v1_r(r_scene),
      function(value) same_json_scene(value, expected),
      case$id, "R_pure", "json_encoding_only", cells, json_bytes
    )
    results[[length(results) + 1L]] <- measure_one(
      function() build_scene_json_r(labels, matrix, annotation),
      function(value) same_json_scene(value, expected),
      case$id, "R_pure", "scene_construction_plus_json_encoding", cells, json_bytes
    )
  }

  candidates <- list(
    Cpp_Rcpp = function() build_scene_json_cpp(labels, matrix, annotation),
    Rust_rextendr = function() build_scene_json_rust(labels, matrix, annotation)
  )
  for (candidate_name in names(candidates)) {
    results[[length(results) + 1L]] <- measure_one(
      candidates[[candidate_name]],
      function(value) same_json_scene(value, expected),
      case$id, candidate_name, "scene_construction_plus_json_encoding",
      cells, json_bytes, stress = stress_mode
    )
  }
}

payload <- list(
  schema_version = 2L,
  suite = "corrplot-ng-cross-language-scene-v1",
  candidates = if (stress_mode) c("Python", "Cpp_Rcpp", "Rust_rextendr") else c("Python", "R_pure", "Cpp_Rcpp", "Rust_rextendr"),
  timing_method = list(
    clock = "R proc.time elapsed with calibrated repeated batches",
    target_sample_ms = target_sample_ms,
    samples = if (stress_mode) stress_sample_count else regular_sample_count,
    warmups = if (stress_mode) stress_warmups else regular_warmups,
    stress_mode = stress_mode, stress_batch_size = if (stress_mode) 1L else NULL,
    batching = "calibrate batch size for regular workloads; fixed batch size 1 for B03/B04"
  ),
  environment = list(
    R = R.version.string, platform = R.version$platform,
    os = Sys.info()[["sysname"]], machine = Sys.info()[["machine"]],
    cxx_compiler = system2("R", c("CMD", "config", "CXX"), stdout = TRUE),
    rustc = system2("rustc", "--version", stdout = TRUE),
    rcpp_version = as.character(packageVersion("Rcpp")),
    rextendr_version = as.character(packageVersion("rextendr")),
    jsonlite_version = as.character(packageVersion("jsonlite")),
    native_package_sizes = list(
      cpp_rcpp = package_artifact_sizes("corrplotngcppspike"),
      rust_rextendr = package_artifact_sizes("corrplotngrustspike")
    )
  ),
  memory_measurement = list(
    status = "not measured per candidate",
    reason = "All implementations run in one R process; process RSS cannot be attributed reliably to an individual native candidate."
  ),
  results = results
)
dir.create(dirname(output_path), recursive = TRUE, showWarnings = FALSE)
write_json(payload, output_path, pretty = TRUE, auto_unbox = TRUE,
           digits = NA, null = "null")
