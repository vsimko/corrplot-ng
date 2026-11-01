#!/usr/bin/env Rscript
# Load the extendr cdylib built from native-r/Cargo.toml and call its registered
# .Call entry point. This is an experiment, not an installable CRAN package.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 3L) {
  stop("usage: Rscript run-native-rust.R LIBRARY FIXTURE.json OUTPUT.json", call. = FALSE)
}
library_path <- normalizePath(args[[1]], mustWork = TRUE)
fixture <- normalizePath(args[[2]], mustWork = TRUE)
output_path <- args[[3]]
for (package in c("jsonlite", "digest")) {
  if (!requireNamespace(package, quietly = TRUE)) {
    stop("missing R package: ", package, call. = FALSE)
  }
}
dyn.load(library_path)
on.exit(dyn.unload(library_path), add = TRUE)

fixture_raw <- readBin(fixture, what = "raw", n = file.info(fixture)$size)
request <- rawToChar(fixture_raw)
invoke <- function() .Call("wrap__corrplot_validate_json", request)
invisible(replicate(5L, invoke(), simplify = FALSE))
pilot_calls <- 10L
started <- proc.time()[["elapsed"]]
for (i in seq_len(pilot_calls)) response_json <- invoke()
pilot_seconds <- proc.time()[["elapsed"]] - started
calls <- max(100L, min(10000L, as.integer(ceiling(0.5 / max(pilot_seconds / pilot_calls, 1e-6)))))
started <- proc.time()[["elapsed"]]
for (i in seq_len(calls)) response_json <- invoke()
elapsed_ms <- (proc.time()[["elapsed"]] - started) * 1000

result <- list(
  runtime = R.version.string,
  backend = "native Rust / extendr",
  libraryBytes = file.info(library_path)$size,
  librarySha256 = digest::digest(library_path, algo = "sha256", file = TRUE),
  fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
  calls = calls,
  elapsedMs = elapsed_ms,
  perCallMs = elapsed_ms / calls,
  response = jsonlite::fromJSON(response_json, simplifyVector = FALSE)
)
writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")), output_path)
