#!/usr/bin/env Rscript
# In-process R -> V8 -> WebAssembly runner for the host-neutral ABI module.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 3L) {
  stop("usage: Rscript run-v8.R MODULE.wasm FIXTURE.json OUTPUT.json", call. = FALSE)
}
wasm <- normalizePath(args[[1]], mustWork = TRUE)
fixture <- normalizePath(args[[2]], mustWork = TRUE)
output_path <- args[[3]]

for (package in c("V8", "jsonlite", "digest")) {
  if (!requireNamespace(package, quietly = TRUE)) {
    stop("missing R package: ", package, call. = FALSE)
  }
}

wasm_bytes <- readBin(wasm, what = "raw", n = file.info(wasm)$size)
fixture_bytes <- readBin(fixture, what = "raw", n = file.info(fixture)$size)

ctx <- V8::v8()
ctx$assign("corrplotWasmBytes", wasm_bytes)
ctx$assign("corrplotInputBytes", fixture_bytes)
ctx$eval("if (typeof WebAssembly !== 'object' || typeof WebAssembly.instantiate !== 'function') throw new Error('WebAssembly is not available in this V8 engine');")

# Keep one V8 context and one compiled instance alive for all calls.
invisible(ctx$eval("
  globalThis.corrplotWasmInstance = new WebAssembly.Instance(
    new WebAssembly.Module(corrplotWasmBytes), {}
  );
  globalThis.corrplotWasmRun = function () {
    const input = corrplotInputBytes;
    const e = globalThis.corrplotWasmInstance.exports;
    const inputPtr = e.corrplot_alloc(input.length);
    let outputPtr = 0;
    let outputLen = 0;
    try {
      new Uint8Array(e.memory.buffer, inputPtr, input.length).set(input);
      const packed = e.corrplot_validate_json(inputPtr, input.length);
      outputPtr = Number(packed & 0xffffffffn);
      outputLen = Number(packed >> 32n);
      if (!outputPtr || !outputLen) throw new Error('WASM validation returned an empty result');
      return Array.from(new Uint8Array(e.memory.buffer, outputPtr, outputLen), byte => String.fromCharCode(byte)).join('');
    } finally {
      if (outputPtr && outputLen) e.corrplot_free(outputPtr, outputLen);
      if (inputPtr) e.corrplot_free(inputPtr, input.length);
    }
  };
"))

invoke <- function() ctx$call("corrplotWasmRun")
script_arg <- grep("^--file=", commandArgs(), value = TRUE)
source(file.path(dirname(normalizePath(sub("^--file=", "", script_arg[[1]]), mustWork = TRUE)), "benchmark-utils.R"))
metrics <- benchmark_repeated(invoke)
response_json <- invoke()

result <- list(
  runtime = R.version.string,
  backend = "V8 WebAssembly (in-process)",
  v8 = list(version = V8::engine_info()$version, numeric_version = as.character(V8::engine_info()$numeric_version)),
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
  response = jsonlite::fromJSON(response_json, simplifyVector = FALSE)
)
writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")), output_path)
