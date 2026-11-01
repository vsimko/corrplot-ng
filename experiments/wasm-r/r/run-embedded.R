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
  wasmtime_library <- if (.Platform$OS.type == "windows") {
    file.path(lib, "wasmtime.dll")
  } else if (identical(Sys.info()[["sysname"]], "Darwin")) {
    file.path(lib, "libwasmtime.dylib")
  } else {
    file.path(lib, "libwasmtime.so")
  }
  if (!file.exists(wasmtime_library)) {
    stop("Wasmtime shared library not found: ", wasmtime_library, call. = FALSE)
  }

  for (package in c("jsonlite", "digest")) {
    if (!requireNamespace(package, quietly = TRUE)) stop("missing R package: ", package, call. = FALSE)
  }

  script_arg <- grep("^--file=", commandArgs(), value = TRUE)
  script_path <- sub("^--file=", "", script_arg[[1]])
  source_file <- file.path(dirname(normalizePath(script_path, mustWork = TRUE)), "wasmtime-host.c")
  # Keep the temporary extension in the repository working directory on Windows.
  # R CMD SHLIB / MinGW mishandles drive-letter absolute output paths (it drops
  # the colon and separators while constructing the linker -o argument).
  shared_dir <- if (.Platform$OS.type == "windows") getwd() else tempdir()
  shared <- tempfile(pattern = "wasmtime-host-", tmpdir = shared_dir, fileext = paste0(".", .Platform$dynlib.ext))
  on.exit(unlink(shared), add = TRUE)

  # Pass include and linker flags through the environment. On Windows, R CMD SHLIB
  # reads Makevars.win rather than a generic Makevars file, so a temporary
  # R_MAKEVARS_USER file is not portable across all R platforms.
  quote_flag_path <- function(path) {
    path <- normalizePath(path, winslash = "/", mustWork = TRUE)
    paste0('"', gsub('"', '\\\\"', path, fixed = TRUE), '"')
  }
  cppflags <- paste0("-I", quote_flag_path(include))
  ldflags <- paste0("-L", quote_flag_path(lib))
  linker_flags <- if (.Platform$OS.type == "windows") {
    "-lwasmtime"
  } else if (identical(Sys.info()[["sysname"]], "Darwin")) {
    c(paste0("-Wl,-rpath,", quote_flag_path(lib)), "-lwasmtime")
  } else {
    c(paste0("-Wl,-rpath,", quote_flag_path(lib)), "-lwasmtime", "-lpthread", "-ldl", "-lm")
  }
  # Set these in the current R process so the child R CMD SHLIB inherits
  # them unchanged on Unix and Windows (system2(env=...) handles spaces
  # differently across platforms).
  Sys.setenv(
    PKG_CPPFLAGS = cppflags,
    PKG_LIBS = paste(c(ldflags, linker_flags), collapse = " ")
  )
  status <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "SHLIB", "-o", shQuote(shared), shQuote(source_file))
  )
  if (status != 0L) stop("failed to compile the embedded Wasmtime R host", call. = FALSE)
  # Windows does not search the linked DLL's directory automatically when
  # loading an R extension. Add the Wasmtime C API lib directory to PATH.
  if (.Platform$OS.type == "windows") {
    Sys.setenv(PATH = paste(lib, Sys.getenv("PATH"), sep = .Platform$path.sep))
    # Preload by absolute path: Windows may not resolve a dependent DLL from
    # PATH when loading an R extension from R's temporary directory.
    wasmtime_handle <- dyn.load(wasmtime_library)
  }
  dyn.load(shared)
  on.exit(.Call("corrplot_wasm_shutdown"), add = TRUE)

  fixture_raw <- readBin(fixture, what = "raw", n = file.info(fixture)$size)
  request <- rawToChar(fixture_raw)
  invoke <- function() .Call("corrplot_wasm_validate", wasm, request)
  script_arg <- grep("^--file=", commandArgs(), value = TRUE)
  source(file.path(dirname(normalizePath(sub("^--file=", "", script_arg[[1]]), mustWork = TRUE)), "benchmark-utils.R"))
  metrics <- benchmark_repeated(invoke)
  response_json <- invoke()
  response <- jsonlite::fromJSON(response_json, simplifyVector = FALSE)

  result <- list(
    runtime = R.version.string,
    wasmtime = "C API (embedded)",
    wasmSha256 = digest::digest(wasm, algo = "sha256", file = TRUE),
    fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
    calls = metrics$calls,
  callsPerSample = metrics$callsPerSample,
  sampleCount = metrics$sampleCount,
  targetSecondsPerSample = metrics$targetSecondsPerSample,
  elapsedMs = metrics$elapsedMs,
  perCallMs = metrics$meanPerCallMs,
  meanPerCallMs = metrics$meanPerCallMs,
  sdPerCallMs = metrics$sdPerCallMs,
  ci95LowerMs = metrics$ci95LowerMs,
  ci95UpperMs = metrics$ci95UpperMs,
  samplesPerCallMs = metrics$samplesPerCallMs,
    response = response
  )
  output <- file(output_path, open = "wt")
  on.exit(close(output), add = TRUE)
  writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")), output)
  flush(output)
  invisible(result)
}

main()
