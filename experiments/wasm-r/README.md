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

The Node embedded runner also reports WASM compilation/instantiation time,
first-call latency, input-copy/validation/output-copy phase timings, process
RSS snapshots, and WASM linear-memory size before and after execution. The
process-based Node runner reports end-to-end cold process time and request /
response byte counts. Its RSS delta is for the parent Node process only; it is
not a measurement of the spawned Wasmtime process. These are host-side
measurements and should not be conflated with the corresponding R-side metrics.

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

The native-R reference benchmark now runs the same JSON request decoding and
validation contract. Both R benchmarks warm up first, estimate per-call cost,
and adapt the measured iteration count to target roughly 0.5 seconds (bounded
to 100–10,000 calls), avoiding zero-millisecond readings for tiny fixtures. In addition to the small
correlation fixtures, the suite includes a 64×64 symmetric correlation
matrix and 32×32/128×128 general-numeric matrices with explicit missing values. CI compares semantic responses with the
embedded Wasmtime result and reports per-call timings. The larger fixtures help
expose input parsing, allocation, and linear-memory transfer costs; they are
still synthetic microbenchmarks. This is a
microbenchmark of the validation boundary, not a full plotting benchmark; the
reference implementation is intentionally isolated from the production package.

The embedded ABI CI matrix now includes Ubuntu, Intel macOS (`macos-15-intel`),
and Windows (`windows-latest`). Windows uses the Wasmtime MinGW C API archive
and the GNU Rust target so the native `extendr` DLL matches the Rtools toolchain.
The R host selects the platform shared library (`.so`, `.dylib`, or `.dll`) and
platform-specific linker flags. Windows support is considered experimental
until the Windows CI job passes; do not treat the platform as validated merely
because it is present in the matrix. Further work should measure larger matrix
sizes and memory/transfer costs. The current
wasm-bindgen artifact should not be reused for this purpose without its
JavaScript runtime glue.

## Measurements to collect

- Exact WASM SHA-256 and Wasmtime version.
- Cold process startup and repeated invocation cost.
- Input/output transfer overhead for small, medium, and large matrices.
- Peak R/Node process memory and Wasmtime linear memory.
- Installation and runtime dependency matrix for Windows, macOS, and Linux.
- Native R adapter baseline using the same fixtures.

The Node runners report first-call and memory/transfer snapshots. The embedded R/Wasmtime runner now separates module initialization from the first validation call and records R-process RSS snapshots plus Wasmtime linear-memory size after initialization and first call. These RSS values are snapshots, not peak-memory measurements. The R runner reports request/response byte counts, but phase-level input-copy, guest-execution, and output-copy timings inside the C shim remain open acceptance work; independent-run repeatability and peak-memory sampling also remain outstanding. Node metrics do not substitute for R-host measurements.

Do not use this first JSON/stdin process path as the final high-throughput
interface; it is a feasibility baseline.

## Three R backend comparison

The experiment now compares three ways for R to call the same Rust validation
core. Node.js and browser consumers remain WASM consumers; this comparison is
only about the R host integration.

1. **V8:** R package `V8` instantiates the existing
   `wasm32-unknown-unknown` module in-process. The runner keeps one V8 context
   and WASM instance alive across calls. It uses the WebAssembly JS API directly
   because the module exports linear memory as well as functions.
2. **Wasmtime:** the existing R `.Call` C shim embeds the Wasmtime C API and
   invokes the same host-neutral module in-process.
3. **Native Rust:** an `extendr` cdylib links to `corrplot-core` and exposes
   the same JSON request/response boundary directly to R, without a WASM
   runtime. This is an experimental binding, not yet a complete CRAN package.

Run V8 after building the embedded ABI module and installing `V8`,
`jsonlite`, and `digest`:

```sh
Rscript experiments/wasm-r/r/run-v8.R MODULE.wasm FIXTURE.json OUTPUT.json
```

Build and run the native Rust binding with an R installation available to
Cargo (the build uses `R_HOME`):

```sh
R_HOME="$(R RHOME)" cargo build \
  --manifest-path experiments/wasm-r/native-r/Cargo.toml --release
Rscript experiments/wasm-r/r/run-native-rust.R \
  experiments/wasm-r/native-r/target/release/libcorrplot_r_native.so \
  experiments/wasm-r/fixtures/valid-small.json /tmp/native-rust.json
```

The CI workflow runs all three R backends against the same fixtures and checks
their semantic responses against the Wasmtime/Node results. Each backend now runs
10 separately timed batches per fixture, targeting one second per batch after
warm-up. The reported mean per-call time and 95% Student-t confidence interval
are calculated across those batch-level measurements. This captures
within-process batch variability, not independent machine-to-machine variation.
Set `CORRPLOT_BENCH_SAMPLES` and `CORRPLOT_BENCH_TARGET_SECONDS` to adjust
the defaults.

The suite also generates a 512×512 general-numeric matrix (16× the element count
of the previous 128×128 largest fixture) to make JSON parsing, allocation, and
validation work more substantial. Existing smaller fixtures remain useful for
exposing fixed call-boundary overhead. These are still synthetic microbenchmarks,
not a full plotting benchmark or stable cross-machine performance claims.

The comparison targets Ubuntu, Intel macOS, and Windows in CI. Windows timings
should only be added to the cross-platform results after its job passes and its
benchmark artifact has been checked. Complete package installation size
(including V8 and Rust runtime dependencies), cold-start cost, and peak memory
remain explicit follow-up measurements.


## Benchmark results

The initial R backend measurements (Linux x86-64 and macOS Intel) are recorded in [results/benchmark-2026-11-01.md](results/benchmark-2026-11-01.md), with machine-readable values in [results/benchmark-2026-11-01.csv](results/benchmark-2026-11-01.csv). They are exploratory warm-throughput measurements; cold-start, RSS and Windows remain open.


## Provisional decision record — 2026-11-03

**Recommendation: (B) native Rust integration for R, with WebAssembly retained for
Node.js/browser consumers.** This is a distribution and runtime recommendation
for the tested validation boundary, not approval to migrate the production R
package.

### Evidence

- The same host-neutral `.wasm` module bytes were invoked from Node and R via
  both process-based WASI and embedded Wasmtime. The embedded module hash and
  semantic responses matched across Ubuntu, Intel macOS, and Windows.
- The embedded module is approximately 144 KiB, but the R host runtime is much
  larger: the measured Wasmtime shared library ranged from 65.5 MiB to
  102.7 MiB, while the installed V8 package ranged from 34.5 MiB to 47.5 MiB.
- In the single recorded CI run, the native Rust R binding was fastest for all
  six fixtures on all three platforms. On the 512×512 fixture it took about
  5.78 ms/call on Linux, 13.65 ms/call on Intel macOS, and 5.89 ms/call on
  Windows; the Wasmtime and V8 paths were slower in that run.
- The process-based WASI route proves portability and semantic parity, but its
  process startup and JSON/stdin boundary make it a feasibility baseline, not
  a suitable high-throughput matrix API.

### Why not (A) WASM-only for R

A portable module does not remove the host runtime requirement. Embedding
Wasmtime or V8 adds a large platform-specific native dependency, and the
measured warm validation path did not outperform the native Rust binding.
CRAN suitability and the complete end-user installation footprint have not
been established.

### Remaining gates before any production decision

The current measurements are insufficient for a release decision:

1. Measure cold process startup and first-call initialization separately from
   warm repeated-call throughput.
2. Record peak process RSS and Wasmtime linear-memory high-water marks on each
   supported platform.
3. Instrument input copy, guest execution, and output copy separately rather
   than reporting only end-to-end JSON-call time.
4. Repeat the measurements across independent CI runs and verify the complete
   installed dependency closure and CRAN implications.
5. Record a final tested/untested platform matrix and the exact R/runtime
   versions used.

Until those gates are met, keep this as an isolated experiment. Do not replace
the production R backend, claim CRAN compatibility, or make the WASM experiment
a blocker for native Node/browser engineering.
