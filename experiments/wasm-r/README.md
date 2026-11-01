# WASM runtime from R experiment

This isolated experiment tests whether one Rust-compiled WebAssembly module can
be invoked from both Node.js and R. It does not change the production R or Node
backend.

## First implementation: shared WASI command module

The current `bindings/wasm` artifact is built with `wasm-bindgen` and expects
JavaScript glue/imports. It is therefore not a raw, host-neutral module that R
can instantiate directly through Wasmtime.

This experiment adds a separate `wasm32-wasip1` command adapter around the
existing `corrplot-core::validate_matrix` operation. It accepts one JSON
request on stdin and emits one JSON response on stdout. The Rust validation
logic remains in the shared core; the command adapter only implements the
host protocol.

Both Node and R invoke the exact same `.wasm` file through the Wasmtime CLI.
This intentionally tests the simplest cross-runtime distribution model first.
It also exposes the cost of process startup and JSON serialization, so it is
not a performance claim about an embedded runtime.

## Build

From the repository root:

```sh
rustup target add wasm32-wasip1
CARGO_TARGET_DIR=experiments/wasm-r/wasm-host/target cargo build \\
  --manifest-path experiments/wasm-r/wasm-host/Cargo.toml \
  --target wasm32-wasip1 --release
```

Locate the resulting module with the `find` command above; Cargo's target
folder can be configured by the environment or workspace settings.

Install Wasmtime and the two R-side experiment dependencies, then record the version:

```sh
wasmtime --version
Rscript -e 'install.packages(c("jsonlite", "digest"))'
```

## Run from Node and R

```sh
WASM=experiments/wasm-r/wasm-host/target/wasm32-wasip1/release/corrplot_wasm_r_host.wasm
FIXTURE=experiments/wasm-r/fixtures/valid-small.json

node experiments/wasm-r/node/run.mjs "$WASM" "$FIXTURE"
Rscript experiments/wasm-r/r/run.R "$WASM" "$FIXTURE"
```

Both runners report a module hash and the semantic response. Both runners calculate SHA-256 for the module and fixture. CI compares the
hashes reported by each runtime and verifies that both produced the same
semantic response.

Use `CORRPLOT_WASMTIME` to select a non-default Wasmtime executable.

## Second implementation: in-process host-neutral ABI

The `wasm-abi` crate builds the same `corrplot-core::validate_matrix` operation
for `wasm32-unknown-unknown`, without wasm-bindgen or WASI imports. Its exported
C ABI consists of:

- `corrplot_alloc(len) -> ptr` for host-owned input buffers;
- `corrplot_validate_json(ptr, len) -> u64`, packing output length in the high
  32 bits and output pointer in the low 32 bits;
- `corrplot_free(ptr, len)` for explicit buffer release;
- exported linear memory.

Node instantiates this module with the built-in `WebAssembly` API. The R runner
loads a small C shim through `.Call`; that shim embeds Wasmtime through its C
API, invokes the same exports, and returns the JSON response to R. CI pins
Wasmtime C API 50.0.1 and verifies its release archive SHA-256 before use.

Run the in-process path locally after installing the matching Wasmtime C API
release and setting `WASMTIME_C_API_ROOT` to its extracted directory:

```sh
CARGO_TARGET_DIR=experiments/wasm-r/wasm-abi/target cargo build \
  --manifest-path experiments/wasm-r/wasm-abi/Cargo.toml \
  --target wasm32-unknown-unknown --release
EMBEDDED=experiments/wasm-r/wasm-abi/target/wasm32-unknown-unknown/release/corrplot_wasm_r_abi.wasm
node experiments/wasm-r/node/run-embedded.mjs "$EMBEDDED" experiments/wasm-r/fixtures/valid-small.json
WASMTIME_C_API_ROOT=/path/to/wasmtime-c-api \
  Rscript experiments/wasm-r/r/run-embedded.R "$EMBEDDED" experiments/wasm-r/fixtures/valid-small.json
```

## What this proves — and does not prove

It proves that both process-based WASI and in-process embedding are viable
integration shapes on Linux, provided the CI checks pass. It does **not** prove
that a CRAN package can ship only the `.wasm` file: the R embedding still needs
a platform-specific native `libwasmtime` and a C compiler/toolchain to build the
small host shim. The module itself is portable, but the host runtime is not.

The next experiment stage is to validate the ABI on macOS and Windows, measure
repeated calls and transfer costs, and compare against a native R baseline. The
current wasm-bindgen artifact should not be reused for this purpose without its
JavaScript runtime glue.

## Measurements to collect

- Exact WASM SHA-256 and Wasmtime version.
- Cold process startup and repeated invocation cost.
- Input/output transfer overhead for small, medium, and large matrices.
- Peak R/Node process memory and Wasmtime linear memory.
- Installation and runtime dependency matrix for Windows, macOS, and Linux.
- Native R adapter baseline using the same fixtures.

Do not use this first JSON/stdin process path as the final high-throughput
interface; it is a feasibility baseline.
