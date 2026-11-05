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

It demonstrates that both process-based WASI and in-process embedding are viable
integration shapes on Ubuntu Linux, Intel macOS, and Windows: the manual matrix
in workflow run [#41861322600](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600)
completed all nine runner repetitions. It does **not** prove that a CRAN package
can ship only the `.wasm` file: the R embedding still needs a platform-specific
native `libwasmtime` and a C compiler/toolchain to build the small host shim.
The module itself is portable, but the host runtime is not.

The native-R reference benchmark runs the same JSON request decoding and
validation contract as the other backends. The manual matrix warms up first and
uses ten timed batches per backend/fixture with an adaptive call-count target
of roughly one second per batch (bounded to 100–10,000 calls); the pull-request
smoke uses three batches targeting 0.1 seconds. This avoids zero-millisecond
readings for tiny fixtures. The suite includes a 64×64 symmetric correlation
matrix and 32×32, 128×128, and 512×512 general-numeric matrices with explicit
missing values. CI compares semantic responses with the embedded Wasmtime
result and reports per-call timings. These are synthetic microbenchmarks of the
validation boundary, not a full plotting benchmark; the reference
implementation is intentionally isolated from the production package.

The embedded ABI CI matrix has now passed three independent runner
repetitions on Ubuntu, Intel macOS (`macos-15-intel`), and Windows
(`windows-latest`). Windows uses the Wasmtime MinGW C API archive and the GNU
Rust target so the native `extendr` DLL matches the Rtools toolchain. The R host
selects the platform shared library (`.so`, `.dylib`, or `.dll`) and
platform-specific linker flags. Darwin does not expose process-lifetime
`ru_maxrss` through the `getrusage` interface used here; the runner records
that peak as JSON `null` with an explicit note rather than substituting a
snapshot. The current wasm-bindgen artifact should not be reused for this
purpose without its JavaScript runtime glue.

## Measurements collected — manual matrix 2026-11-04/05

The completed nine-job matrix records:

- SHA-256 for the process-based WASI module, the embedded ABI module, and each fixture;
- Wasmtime CLI invocation time from the already-running R and Node hosts, embedded-runtime initialization, first-call latency, and warm repeated-call throughput;
- current R/Node RSS snapshots, process-lifetime peak RSS where the OS exposes it, and Wasmtime linear-memory size;
- request/response byte counts and separate allocation, input-copy, guest-execution, input-free, output-copy, and output-free timings;
- six fixtures including the 512×512 general-numeric case;
- three independent CI runner repetitions per OS, with three separate Rscript processes per fixture and runner;
- exact R, Node, Rust, Wasmtime CLI/C API, and V8 runtime versions, plus runtime/module sizes.

The C shim uses a monotonic high-resolution clock. Process-lifetime peak RSS includes R and Wasmtime allocations; it is not a WASM-only memory measurement. macOS peak RSS is unavailable through this `getrusage` path and is explicitly reported as unavailable. The process-based WASI RSS deltas describe only the parent R or Node process, not the spawned Wasmtime child.

The raw artifacts and machine-readable summaries are linked from [the 2026-11-05 results report](results/benchmark-2026-11-05.md).

Committed fixture JSON is pinned to LF in `.gitattributes`, and the workflow
rejects CRLF fixture bytes before hashing. This removes the Windows checkout
hash drift observed in the completed matrix for the five committed fixtures.
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
their semantic responses against the Wasmtime/Node results. In the manual
matrix, each backend runs ten separately timed batches per fixture, targeting
one second per batch after warm-up; pull-request smoke runs three batches at
0.1 seconds. The reported mean per-call time and 95% Student-t confidence
interval are calculated across those batch-level measurements. This captures
within-process batch variability, not independent machine-to-machine variation.
Set `CORRPLOT_BENCH_SAMPLES` and `CORRPLOT_BENCH_TARGET_SECONDS` to adjust
the defaults.

The suite also generates a 512×512 general-numeric matrix (16× the element count
of the previous 128×128 largest fixture) to make JSON parsing, allocation, and
validation work more substantial. Existing smaller fixtures remain useful for
exposing fixed call-boundary overhead. These are still synthetic microbenchmarks,
not a full plotting benchmark or stable cross-machine performance claims.

The manual matrix completed and its nine artifacts were reviewed. The current
run records R 4.6.2 on all platforms, Node 24.x, Rust 1.99.1, Wasmtime 50.0.1,
and V8 8.2.1 (with platform-specific embedded V8 engine versions). The full
dependency closure and production-package installation size remain follow-up
work; the measured library/package sizes below are component sizes only.

The workflow now writes Node, Rust, Cargo, Wasmtime CLI, Wasmtime C API
archive/hash, and Rscript version metadata directly into runtime-size artifacts.


## Benchmark results

The completed three-platform, three-repetition R-host matrix is recorded in
[results/benchmark-2026-11-05.md](results/benchmark-2026-11-05.md). The
machine-readable warm-throughput data is in
[results/benchmark-2026-11-05.csv](results/benchmark-2026-11-05.csv), and
cold-start, memory, and transfer-phase process data is in
[results/benchmark-2026-11-05-runtime.csv](results/benchmark-2026-11-05-runtime.csv).

The [2026-11-01 report](results/benchmark-2026-11-01.md) and its CSV are
retained as the historical baseline; they are not the current cross-platform
R-host results.


## Decision record — 2026-11-05

**Recommendation: (B) native Rust integration for R, with WebAssembly retained
for Node.js/browser consumers.** This updates the provisional 2026-11-03
recommendation using the completed R-host matrix. It is a distribution/runtime
recommendation for the tested validation boundary, not approval to migrate or
release the production R package.

### Evidence

- The manual workflow completed 3× Ubuntu, 3× Intel macOS, and 3× Windows
  runner repetitions. For both process-based WASI and the embedded ABI, Node
  and R reported matching module and fixture hashes within each platform and
  returned equivalent semantic responses across all six fixtures.
- Native Rust was fastest by median for all six fixtures on all three
  platforms. On the 512×512 fixture it measured 5.62 ms/call on Ubuntu,
  11.00 ms/call on Intel macOS, and 5.90 ms/call on Windows.
- V8 and Wasmtime trade places depending on fixture and platform; neither is
  consistently faster. Both are slower than the native Rust binding in these
  measurements.
- The embedded ABI module is about 144 KiB, but the Wasmtime shared library
  ranges from 65.5 MiB to 102.7 MiB and the installed V8 package from 34.5 MiB
  to 47.5 MiB. The native Rust R binding library ranges from 0.63 MiB to
  2.22 MiB.
- Embedded Wasmtime initialization is roughly 46–55 ms median and first-call
  latency roughly 16–19 ms for the 512×512 fixture. Process-lifetime peak RSS
  is about 105 MiB on Ubuntu and 100 MiB on Windows; Darwin peak RSS is
  unavailable through the current `getrusage` interface. Wasmtime linear
  memory is 10.75 MiB after the 512×512 call.
- The process-based WASI route proves cross-runtime portability and semantic
  parity, but each invocation pays spawned-process startup and JSON/stdin
  transport costs. It remains a feasibility baseline, not the high-throughput
  interface.

### Why not (A) WASM-only for R

A portable module does not remove the host runtime requirement. Embedding
Wasmtime or V8 adds a large platform-specific native dependency, and neither
WASM host outperformed the native Rust binding consistently. The measured
component sizes are not a complete package dependency closure, and CRAN
suitability has not been established.

### Remaining gates before any production decision

1. Build and measure the actual production R package candidate, not only the
   disposable `extendr` adapter.
2. Record complete source-tarball and installed dependency/license inventories.
3. Verify offline source installation, `R CMD check --as-cran`, supported
   platform behavior, and Windows binary installation without Rust on the
   end user's PATH.
4. Obtain authoritative CRAN/maintainer guidance on the retained Cargo
   checksum/provenance NOTE tracked by #268.

Until those gates and an explicit maintainer decision are complete, keep this
as an isolated experiment. Do not replace the production R backend, claim CRAN
compatibility, or treat the WASM experiment as a production migration approval.
