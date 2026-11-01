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
if (!isTRUE(ctx$eval("typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function'"))) {
  stop("the selected V8 engine does not support WebAssembly", call. = FALSE)
}

# Keep one V8 context and one compiled instance alive for all calls.
ctx$eval("
  globalThis.corrplotWasmInstance = new WebAssembly.Instance(
    new WebAssembly.Module(corrplotWasmBytes), {}
  );
  globalThis.corrplotWasmRun = function (input) {
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
")

invoke <- function() ctx$call("corrplotWasmRun", fixture_bytes)
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
  backend = "V8 WebAssembly (in-process)",
  v8 = as.character(ctx$eval("JSON.stringify({version: (typeof process !== 'undefined' && process.versions && process.versions.v8) || 'embedded'})")),
  wasmSha256 = digest::digest(wasm, algo = "sha256", file = TRUE),
  fixtureSha256 = digest::digest(fixture, algo = "sha256", file = TRUE),
  calls = calls,
  elapsedMs = elapsed_ms,
  perCallMs = elapsed_ms / calls,
  response = jsonlite::fromJSON(response_json, simplifyVector = FALSE)
)
writeLines(as.character(jsonlite::toJSON(result, auto_unbox = TRUE, null = "null")), output_path)
