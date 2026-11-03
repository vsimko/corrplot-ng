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
  shared_dir <- tempdir()
  shared <- tempfile(
    pattern = "wasmtime-host-",
    tmpdir = shared_dir,
    fileext = .Platform$dynlib.ext
  )
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
    c("-lwasmtime", "-lpsapi")
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
  old_wd <- getwd()
  on.exit(setwd(old_wd), add = TRUE)
  setwd(shared_dir)
  status <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "SHLIB", "-o", shQuote(basename(shared)), shQuote(source_file))
  )
  setwd(old_wd)
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

  # Capture the R process footprint with platform-native tools. Values are
  # snapshots (not peak RSS) and are kept distinct from WASM linear memory.
  process_rss_bytes <- function() {
    sysname <- Sys.info()[["sysname"]]
    if (identical(sysname, "Linux") && file.exists("/proc/self/statm")) {
      fields <- scan("/proc/self/statm", what = integer(), quiet = TRUE)
      return(as.numeric(fields[[2]]) * as.numeric(system("getconf PAGESIZE", intern = TRUE)))
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

  fixture_raw <- readBin(fixture, what = "raw", n = file.info(fixture)$size)
  request <- rawToChar(fixture_raw)
  invoke <- function() .Call("corrplot_wasm_validate", wasm, request)
  rss_before_init <- process_rss_bytes()
  peak_rss_before_init <- .Call("corrplot_wasm_peak_rss_bytes")
  init_started <- proc.time()[["elapsed"]]
  .Call("corrplot_wasm_initialize", wasm)
  initialization_ms <- (proc.time()[["elapsed"]] - init_started) * 1000
  rss_after_init <- process_rss_bytes()
  peak_rss_after_init <- .Call("corrplot_wasm_peak_rss_bytes")
  wasm_memory_after_init <- .Call("corrplot_wasm_memory_bytes")

  first_started <- proc.time()[["elapsed"]]
  first_response_json <- invoke()
  first_call_ms <- (proc.time()[["elapsed"]] - first_started) * 1000
  rss_after_first_call <- process_rss_bytes()
  peak_rss_after_first_call <- .Call("corrplot_wasm_peak_rss_bytes")
  wasm_memory_after_first_call <- .Call("corrplot_wasm_memory_bytes")
  first_timed <- .Call("corrplot_wasm_validate_timed", wasm, request)
  if (!identical(first_timed$response, first_response_json)) stop("timed and ordinary cold responses differ", call. = FALSE)

  script_arg <- grep("^--file=", commandArgs(), value = TRUE)
  source(file.path(dirname(normalizePath(sub("^--file=", "", script_arg[[1]]), mustWork = TRUE)), "benchmark-utils.R"))
  metrics <- benchmark_repeated(invoke)
  response_json <- invoke()
  phase_keys <- c("allocNs", "inputCopyNs", "guestExecutionNs", "inputFreeNs", "outputCopyNs", "outputFreeNs")
  phase_samples <- lapply(phase_keys, function(key) {
    samples <- vapply(seq_len(10L), function(i) {
      measured <- .Call("corrplot_wasm_validate_timed", wasm, request)
      if (!identical(measured$response, response_json)) stop("timed and ordinary Wasmtime responses differ", call. = FALSE)
      measured[[key]]
    }, numeric(1))
    list(samplesNs = as.list(samples), medianNs = stats::median(samples), iqrNs = stats::IQR(samples))
  })
  names(phase_samples) <- phase_keys
  response <- jsonlite::fromJSON(response_json, simplifyVector = FALSE)
  first_response <- jsonlite::fromJSON(first_response_json, simplifyVector = FALSE)
  if (!identical(first_response, response)) stop("cold and repeated Wasmtime responses differ", call. = FALSE)

  result <- list(
    runtime = R.version.string,
    wasmtime = "C API (embedded)",
    wasmSha256 = digest::digest(wasm, algo = "sha256", file = TRUE),
    fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
    coldStart = list(
      initializationMs = initialization_ms,
      firstCallMs = first_call_ms
    ),
    memory = list(
      processRssBytesBeforeInitialization = rss_before_init,
      processRssBytesAfterInitialization = rss_after_init,
      processRssBytesAfterFirstCall = rss_after_first_call,
      processRssDeltaAfterInitializationBytes = rss_after_init - rss_before_init,
      processRssDeltaAfterFirstCallBytes = rss_after_first_call - rss_before_init,
      processPeakRssBytesBeforeInitialization = peak_rss_before_init,
      processPeakRssBytesAfterInitialization = peak_rss_after_init,
      processPeakRssBytesAfterFirstCall = peak_rss_after_first_call,
      peakRssNote = "Process-lifetime high-water mark; includes R and Wasmtime allocations and is not a WASM-only memory measure.",
      wasmLinearMemoryBytesAfterInitialization = wasm_memory_after_init,
      wasmLinearMemoryBytesAfterFirstCall = wasm_memory_after_first_call
    ),
    transfer = list(
      requestBytes = length(fixture_raw),
      responseBytes = nchar(response_json, type = "bytes"),
      phaseTimings = phase_samples,
      phaseTimingNote = "Nanosecond timings from the embedded C shim; each phase is sampled separately after warm-up. They are instrumentation measurements, not additive substitutes for end-to-end call latency."
    ),
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
