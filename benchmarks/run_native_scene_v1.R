args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) stop("usage: run_native_scene_v1.R fixtures.json results.json")
fixtures_path <- args[[1]]
output_path <- args[[2]]

suppressPackageStartupMessages(library(jsonlite))
suppressPackageStartupMessages(library(corrplotngcppspike))
suppressPackageStartupMessages(library(corrplotngrustspike))
source("benchmarks/scene_v1_reference.R")

target_sample_ms <- 100
sample_count <- 30L
warmups <- 5L

measure_one <- function(fun, validate, workload, candidate, measurement,
                        scene_cells, scene_json_bytes) {
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
    warmup_iterations = warmups, scene_cells = scene_cells,
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
  r_scene <- build_scene_v1_r(labels, matrix, annotation)
  r_json <- encode_scene_v1_r(r_scene)
  if (!isTRUE(same_json_scene(r_json, expected))) {
    stop(case$id, ": pure R scene-v1 output differs from Python oracle")
  }

  cells <- length(expected$cells)
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

  candidates <- list(
    Cpp_Rcpp = function() build_scene_json_cpp(labels, matrix, annotation),
    Rust_rextendr = function() build_scene_json_rust(labels, matrix, annotation)
  )
  for (candidate_name in names(candidates)) {
    results[[length(results) + 1L]] <- measure_one(
      candidates[[candidate_name]],
      function(value) same_json_scene(value, expected),
      case$id, candidate_name, "scene_construction_plus_json_encoding",
      cells, json_bytes
    )
  }
}

payload <- list(
  schema_version = 2L,
  suite = "corrplot-ng-cross-language-scene-v1",
  candidates = c("Python", "R_pure", "Cpp_Rcpp", "Rust_rextendr"),
  timing_method = list(
    clock = "R proc.time elapsed with calibrated repeated batches",
    target_sample_ms = target_sample_ms, samples = sample_count,
    warmups = warmups,
    batching = "double batch size until one sample reaches target duration"
  ),
  environment = list(
    R = R.version.string, platform = R.version$platform,
    os = Sys.info()[["sysname"]], machine = Sys.info()[["machine"]],
    cxx_compiler = system2("R", c("CMD", "config", "CXX"), stdout = TRUE),
    rustc = system2("rustc", "--version", stdout = TRUE),
    rcpp_version = as.character(packageVersion("Rcpp")),
    rextendr_version = as.character(packageVersion("rextendr")),
    jsonlite_version = as.character(packageVersion("jsonlite"))
  ),
  results = results
)
dir.create(dirname(output_path), recursive = TRUE, showWarnings = FALSE)
write_json(payload, output_path, pretty = TRUE, auto_unbox = TRUE,
           digits = NA, null = "null")
