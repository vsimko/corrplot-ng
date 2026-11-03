args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) {
  stop("usage: Rscript run.R <module.wasm> <fixture.json>", call. = FALSE)
}

module <- normalizePath(args[[1L]], mustWork = TRUE)
fixture <- normalizePath(args[[2L]], mustWork = TRUE)
wasmtime <- Sys.getenv("CORRPLOT_WASMTIME", unset = Sys.which("wasmtime"))
if (!nzchar(wasmtime)) {
  stop("Wasmtime CLI not found; install Wasmtime or set CORRPLOT_WASMTIME", call. = FALSE)
}

if (!requireNamespace("jsonlite", quietly = TRUE) || !requireNamespace("digest", quietly = TRUE)) {
  stop("the jsonlite and digest packages are required for this experiment", call. = FALSE)
}

input <- readBin(fixture, what = "raw", n = file.info(fixture)$size)
input_file <- tempfile("corrplot-wasm-input-")
output_file <- tempfile("corrplot-wasm-output-")
error_file <- tempfile("corrplot-wasm-stderr-")
on.exit(unlink(c(input_file, output_file, error_file)), add = TRUE)
writeBin(input, input_file)

version <- system2(wasmtime, "--version", stdout = TRUE, stderr = TRUE)
if (!length(version)) version <- "external Wasmtime"

process_rss_bytes <- function() {
  sysname <- Sys.info()[["sysname"]]
  if (identical(sysname, "Linux") && file.exists("/proc/self/statm")) {
    fields <- scan("/proc/self/statm", what = integer(), quiet = TRUE)
    page_size <- suppressWarnings(as.numeric(system("getconf PAGESIZE", intern = TRUE)))
    if (length(fields) >= 2L && is.finite(page_size)) return(as.numeric(fields[[2]]) * page_size)
  }
  if (identical(sysname, "Darwin")) {
    value <- suppressWarnings(system2("ps", c("-o", "rss=", "-p", Sys.getpid()), stdout = TRUE))
    if (length(value) && is.finite(as.numeric(trimws(value[[1]])))) return(as.numeric(trimws(value[[1]])) * 1024)
  }
  if (.Platform$OS.type == "windows") {
    value <- suppressWarnings(system2("powershell", c("-NoProfile", "-Command",
      paste0("(Get-Process -Id ", Sys.getpid(), ").WorkingSet64")), stdout = TRUE))
    if (length(value) && is.finite(as.numeric(value[[1]]))) return(as.numeric(value[[1]]))
  }
  NA_real_
}

rss_before <- process_rss_bytes()
started <- proc.time()[["elapsed"]]
status <- system2(
  command = wasmtime,
  args = c("run", module),
  stdin = input_file,
  stdout = output_file,
  stderr = error_file
)
elapsed <- proc.time()[["elapsed"]] - started
rss_after <- process_rss_bytes()

stderr <- if (file.exists(error_file)) paste(readLines(error_file, warn = FALSE), collapse = "\n") else ""
if (!identical(status, 0L)) {
  stop(sprintf("Wasmtime exited with status %s: %s", status, stderr), call. = FALSE)
}

output <- paste(readLines(output_file, warn = FALSE), collapse = "\n")
response <- jsonlite::fromJSON(output, simplifyVector = FALSE)
wasm_hash <- digest::digest(module, algo = "sha256", file = TRUE)
fixture_hash <- digest::digest(fixture, algo = "sha256", file = TRUE)

cat(jsonlite::toJSON(list(
  runtime = paste(version, collapse = " "),
  wasmSha256 = wasm_hash,
  fixtureSha256 = fixture_hash,
  elapsedMs = round(elapsed * 1000, 3),
  coldProcessMs = round(elapsed * 1000, 3),
  inputBytes = length(input),
  outputBytes = file.info(output_file)$size,
  parentRssBeforeBytes = rss_before,
  parentRssAfterBytes = rss_after,
  parentRssDeltaBytes = rss_after - rss_before,
  memoryMeasurementNote = "RSS snapshots describe the parent R process, not the spawned Wasmtime child; they are not peak-memory measurements.",
  response = response
), auto_unbox = TRUE, pretty = TRUE), "\n")
