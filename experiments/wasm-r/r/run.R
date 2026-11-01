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

started <- proc.time()[["elapsed"]]
status <- system2(
  command = wasmtime,
  args = c("run", module),
  stdin = input_file,
  stdout = output_file,
  stderr = error_file
)
elapsed <- proc.time()[["elapsed"]] - started

stderr <- if (file.exists(error_file)) paste(readLines(error_file, warn = FALSE), collapse = "\n") else ""
if (!identical(status, 0L)) {
  stop(sprintf("Wasmtime exited with status %s: %s", status, stderr), call. = FALSE)
}

output <- paste(readLines(output_file, warn = FALSE), collapse = "\n")
response <- jsonlite::fromJSON(output, simplifyVector = FALSE)
wasm_hash <- digest::digest(module, algo = "sha256", file = TRUE)
fixture_hash <- digest::digest(fixture, algo = "sha256", file = TRUE)

cat(jsonlite::toJSON(list(
  runtime = if (nzchar(stderr)) stderr else "external Wasmtime",
  wasmSha256 = wasm_hash,
  fixtureSha256 = fixture_hash,
  elapsedMs = round(elapsed * 1000, 3),
  response = response
), auto_unbox = TRUE, pretty = TRUE), "\n")
