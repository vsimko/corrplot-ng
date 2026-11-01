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

## What this proves — and does not prove

It proves that R and Node can drive the same WASI module without embedding Rust
or duplicating matrix validation. It does **not** prove that a CRAN package can
ship only the `.wasm` file: users still need a native Wasmtime executable,
which is platform-specific, and each invocation starts a process.

The next experiment stage should compare a persistent embedded Wasmtime C API
host for R and an in-process Node host. That requires a stable pointer/length
ABI and explicit memory ownership. The current wasm-bindgen artifact should
not be reused for that purpose without its JavaScript runtime glue.

## Measurements to collect

- Exact WASM SHA-256 and Wasmtime version.
- Cold process startup and repeated invocation cost.
- Input/output transfer overhead for small, medium, and large matrices.
- Peak R/Node process memory and Wasmtime linear memory.
- Installation and runtime dependency matrix for Windows, macOS, and Linux.
- Native R adapter baseline using the same fixtures.

Do not use this first JSON/stdin process path as the final high-throughput
interface; it is a feasibility baseline.
