#!/usr/bin/env Rscript
main <- function() {
  args <- commandArgs(trailingOnly = TRUE)
  if (length(args) != 3L) {
    stop("usage: Rscript run-embedded.R MODULE.wasm FIXTURE.json OUTPUT.json", call. = FALSE)
  }
  wasm <- normalizePath(args[[1]], mustWork = TRUE)
  fixture <- normalizePath(args[[2]], mustWork = TRUE)
  output_path <- args[[3]]
  root <- Sys.getenv("WASMTIME_C_API_ROOT")
  if (!nzchar(root)) stop("WASMTIME_C_API_ROOT must point to an extracted Wasmtime C API release", call. = FALSE)
  root <- normalizePath(root, mustWork = TRUE)
  include <- file.path(root, "include")
  lib <- file.path(root, "lib")
  if (!file.exists(file.path(include, "wasmtime.h"))) stop("Wasmtime C API headers not found", call. = FALSE)
  if (!file.exists(file.path(lib, "libwasmtime.so"))) stop("libwasmtime.so not found", call. = FALSE)

  for (package in c("jsonlite", "digest")) {
    if (!requireNamespace(package, quietly = TRUE)) stop("missing R package: ", package, call. = FALSE)
  }

  script_arg <- grep("^--file=", commandArgs(), value = TRUE)
  script_path <- sub("^--file=", "", script_arg[[1]])
  source_file <- file.path(dirname(normalizePath(script_path, mustWork = TRUE)), "wasmtime-host.c")
  shared <- tempfile(fileext = paste0(".", .Platform$dynlib.ext))

  # Keep compiler flags in Makevars; system2() otherwise splits linker arguments.
  makevars <- tempfile()
  on.exit(unlink(c(makevars, shared)), add = TRUE)
  writeLines(c(
    paste("PKG_CPPFLAGS =", paste0("-I", shQuote(include))),
    paste("PKG_LIBS =", paste("-L", shQuote(lib), paste0("-Wl,-rpath,", shQuote(lib)), "-lwasmtime -lpthread -ldl -lm"))
  ), makevars)
  status <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "SHLIB", "-o", shQuote(shared), shQuote(source_file)),
    env = paste0("R_MAKEVARS_USER=", makevars)
  )
  if (status != 0L) stop("failed to compile the embedded Wasmtime R host", call. = FALSE)
  dyn.load(shared)
  on.exit(.Call("corrplot_wasm_shutdown"), add = TRUE)

  fixture_raw <- readBin(fixture, what = "raw", n = file.info(fixture)$size)
  request <- rawToChar(fixture_raw)
  invisible(.Call("corrplot_wasm_validate", wasm, request))
  calls <- 100L
  started <- proc.time()[["elapsed"]]
  for (i in seq_len(calls)) {
    response_json <- .Call("corrplot_wasm_validate", wasm, request)
  }
  elapsed <- (proc.time()[["elapsed"]] - started) * 1000
  response <- jsonlite::fromJSON(response_json, simplifyVector = FALSE)

  result <- list(
    runtime = R.version.string,
    wasmtime = "C API (embedded)",
    wasmSha256 = digest::digest(wasm, algo = "sha256", file = TRUE),
    fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
    calls = calls,
    elapsedMs = elapsed,
    perCallMs = elapsed / calls,
    response = response
  )
  output <- file(output_path, open = "wt")
  on.exit(close(output), add = TRUE)
  writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")), output)
  flush(output)
  invisible(result)
}

main()
