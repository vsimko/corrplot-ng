# R-host WASM benchmark results — 2026-11-05

## Scope and reproducibility

The manual workflow run [#41981917849](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849) completed on 2026-11-05 with **9/9 runner jobs successful** (workflow attempt 1): three independent runner repetitions each on Ubuntu Linux, Intel macOS, and Windows. Measurements ran against commit [`3b4427f88eb26c56ff887be5758ffd251f59f2f3`](https://github.com/vsimko/corrplot-ng/commit/3b4427f88eb26c56ff887be5758ffd251f59f2f3).

Each runner built the wasm32-wasip1 process-based module and the wasm32-unknown-unknown host-neutral ABI module from the shared Rust core; ran six fixtures (small valid, general numeric 32×32, 128×128 and 512×512, 64×64 correlation, and invalid asymmetric); compared module and fixture SHA-256 values and semantic responses across Node, R/Wasmtime, R/V8, native Rust, and the native-R reference; recorded ten warm throughput batches per backend/fixture with within-run 95% Student-t intervals; and launched three separate Rscript processes per fixture for embedded Wasmtime initialization, first-call, memory, and transfer-phase measurements.

All nine artifact ZIPs were downloaded and inspected. Each contains 68 files, including 24 warm-benchmark rows, six fixture results per backend, three embedded-R process results per fixture, six repeatability summaries, and runtime/toolchain metadata. The machine-readable summaries aggregate the three runner repetitions; raw samples remain in the artifact ZIPs. Cross-run medians and ranges are not pooled 95% confidence intervals.

## Platform and toolchain matrix

| Platform | Runner | R | Node | Rust compiler | Wasmtime | V8 package / engine |
|---|---|---|---|---|---|---|
| Ubuntu Linux x86-64 | ubuntu-latest | 4.6.2 (2026-10-29) | v24.22.1 | 1.99.1 (58cf8f9c3, 2026-10-13) | 50.0.1 | 8.2.1 / 12.4.254.21 |
| macOS 15 Intel x86-64 | macos-15-intel | 4.6.2 (2026-10-29) | v24.19.0 | 1.99.1 (58cf8f9c3, 2026-10-13) | 50.0.1 | 8.2.1 / 11.7.439.14 |
| Windows x86-64 GNU | windows-latest | 4.6.2 (2026-10-29 ucrt) | v24.22.2 | 1.99.1 (58cf8f9c3, 2026-10-13) | 50.0.1 | 8.2.1 / 14.6.202.33 |

All platforms used Wasmtime CLI and C API 50.0.1. The workflow verified the C API release archive SHA-256 before use:

| Platform | C API archive | SHA-256 |
|---|---|---|
| Ubuntu Linux x86-64 | wasmtime-v50.0.1-x86_64-linux-c-api.tar.xz | e2c0c26e1aef52f82f0b9918ad6f34cc30f4b557aee3e261cb127a9462be8f4d |
| macOS 15 Intel x86-64 | wasmtime-v50.0.1-x86_64-macos-c-api.tar.xz | b6d436cba5c0964785c6ff7c9b5a743131555a0227b4f742185c4fb520c502c5 |
| Windows x86-64 GNU | wasmtime-v50.0.1-x86_64-mingw-c-api.zip | 9001a5df99305f6bc4a22d1e38f020f953b26cc88755840743dfabd44d9eaea4 |

R 4.6.2 was recorded in all nine runtime metadata files, including Windows. Node, Rust, Cargo, Wasmtime CLI, and the C API archive version/hash were also recorded for every runner.

## Cross-runtime identity and parity

All nine artifacts passed the workflow parity assertions, and independent review of the downloaded JSON confirmed:
- Node and R used matching WASI module and fixture SHA-256 values for every fixture within each platform/repetition.
- Node and R/Wasmtime used matching embedded ABI module and fixture SHA-256 values.
- All three independent Rscript embedded-Wasmtime processes per fixture returned the same semantic response as the Node embedded WebAssembly host.
- R/V8, native Rust, and native-R reference responses matched the embedded result for all six fixtures.

The module builds are **not byte-identical across operating systems**: each platform builds its own module bytes. The demonstrated guarantee is that Node and R use the exact same module bytes within each platform/repetition. All six fixture SHA-256 values were identical across Linux, macOS, and Windows; the fixture-specific .gitattributes rule enforces LF checkout on Windows.

| Platform | WASI module SHA-256 (Node = R) | Embedded ABI SHA-256 (Node = R) | Generated 512×512 fixture SHA-256 |
|---|---|---|---|
| Ubuntu Linux x86-64 | baf63002f4e31b86ee7c9a1fb7cc561cbdaf4c9743995647164f696dfc0d3802 | 5eb4a54893403650cc820abb2714fc8fb333ec5af674a4140c48f9874aeb3ba4 | 589eb4ff04076fb22640d11b597acdf6a9516c4c699d435bc24858eb3a8c56f7 |
| macOS 15 Intel x86-64 | 6c24e7eaffb5257d5139eb56682f356dff9f785a09c48629141ff028e947f49a | ae4089bb00a992f498351f3d6b9d0fb8dcf6665ebf0d05d83748013dd939d1dc | 589eb4ff04076fb22640d11b597acdf6a9516c4c699d435bc24858eb3a8c56f7 |
| Windows x86-64 GNU | 6737ba90bb25d2ddfebc9e9fef78b23746a978255af60767a50ee2e05c1b3b6c | a670cf83ea8dc569cfb28520d83ecdfa5b909845962a47730463d6d3a2ed768c | 589eb4ff04076fb22640d11b597acdf6a9516c4c699d435bc24858eb3a8c56f7 |

## Warm throughput

Values are median per-call milliseconds across three independent runner repetitions; bracketed values show the min–max of the three runner means. Each runner contributes ten timed batches. The CSV retains the median of the three within-run 95% interval endpoints; these are not between-run confidence intervals.

### Ubuntu Linux x86-64

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0669 [0.0477–0.0686] | 0.0040 [0.0034–0.0041] | 0.0019 [0.0016–0.0020] | 0.1361 [0.0967–0.1379] |
| valid-general-32 | 0.1226 [0.0961–0.1253] | 0.0497 [0.0467–0.0503] | 0.0278 [0.0223–0.0278] | 1.9043 [1.3029–1.9140] |
| valid-general-128 | 0.8842 [0.7602–0.9032] | 0.6997 [0.6678–0.7010] | 0.3846 [0.3136–0.3861] | 19.3200 [13.3458–19.8327] |
| valid-general-512 | 11.8721 [9.3748–12.0193] | 11.3943 [10.1330–11.5186] | 5.1887 [4.0455–5.4144] | 245.8400 [184.8333–249.1000] |
| valid-correlation-64 | 0.2857 [0.2379–0.2875] | 0.2008 [0.1874–0.2009] | 0.1103 [0.0869–0.1110] | 8.1827 [5.8740–8.3008] |
| invalid-asymmetric | 0.0839 [0.0621–0.0844] | 0.0047 [0.0039–0.0047] | 0.0024 [0.0020–0.0025] | 0.1068 [0.0769–0.1080] |

### macOS 15 Intel x86-64

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.1056 [0.0935–0.2628] | 0.0068 [0.0067–0.0127] | 0.0035 [0.0035–0.0068] | 0.3799 [0.3651–0.6804] |
| valid-general-32 | 0.2696 [0.1914–0.4184] | 0.0812 [0.0775–0.1305] | 0.0430 [0.0411–0.0842] | 4.4018 [4.3302–7.5105] |
| valid-general-128 | 1.7318 [1.4960–3.6902] | 1.1514 [1.1351–2.0365] | 0.6829 [0.5244–1.6984] | 37.7333 [36.3750–59.5889] |
| valid-general-512 | 18.7250 [16.1800–36.8484] | 15.3365 [14.5870–25.7079] | 9.4160 [8.2992–18.1509] | 401.6000 [401.2333–758.2500] |
| valid-correlation-64 | 0.6304 [0.6067–0.9339] | 0.3991 [0.3490–0.5796] | 0.1865 [0.1827–0.2911] | 19.4421 [17.5421–29.0684] |
| invalid-asymmetric | 0.2422 [0.1563–0.2447] | 0.0151 [0.0103–0.0155] | 0.0071 [0.0050–0.0080] | 0.5297 [0.3081–0.5341] |

### Windows x86-64 GNU

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0697 [0.0363–0.0858] | 0.0042 [0.0028–0.0048] | 0.0023 [0.0014–0.0024] | 0.4021 [0.2472–0.4654] |
| valid-general-32 | 0.1129 [0.0635–0.1317] | 0.0518 [0.0391–0.0586] | 0.0304 [0.0184–0.0313] | 5.2857 [3.3831–5.6538] |
| valid-general-128 | 0.7742 [0.4370–0.8303] | 0.7400 [0.5674–1.0175] | 0.4204 [0.2550–0.4475] | 40.2400 [27.3333–40.5600] |
| valid-general-512 | 9.8774 [5.6126–10.2917] | 12.4250 [8.5804–12.6750] | 5.9026 [3.3054–6.4221] | 437.6667 [314.2500–457.3333] |
| valid-correlation-64 | 0.2465 [0.1466–0.2881] | 0.2175 [0.1530–0.2422] | 0.1105 [0.0693–0.1281] | 16.5246 [11.2396–18.7288] |
| invalid-asymmetric | 0.0822 [0.0449–0.1007] | 0.0050 [0.0035–0.0051] | 0.0028 [0.0018–0.0029] | 0.2861 [0.1713–0.3381] |

## Cold start, first call, memory, and transfer

The table uses the 512×512 fixture. Process-based values time one spawned wasmtime run invocation from an already-running R or Node host; they include child-process startup and execution, not Rscript/Node startup (n=3 runner observations). Embedded Wasmtime initialization and first-call values aggregate three separate Rscript processes in each of three runner repetitions (n=9). Values are median [min–max].

| Platform | R-host Wasmtime CLI invocation ms | Node-host Wasmtime CLI invocation ms | Embedded Wasmtime init ms | Embedded first-call ms | R process peak RSS after first call | WASM linear memory after first call |
|---|---:|---:|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 23.0 [22.0–24.0] | 23.4 [22.1–25.5] | 45.9 [34.7–62.8] | 16.6 [14.5–25.8] | 104.85 [104.09–105.55] MiB (n=9) | 10.75 MiB |
| macOS 15 Intel x86-64 | 53.0 [45.0–73.0] | 39.2 [38.4–62.9] | 59.0 [43.1–74.2] | 20.8 [18.2–35.6] | Unavailable on Darwin; JSON null + note (n=9) | 10.75 MiB |
| Windows x86-64 GNU | 40.0 [30.0–50.0] | 37.3 [24.0–40.8] | 53.5 [32.2–65.2] | 15.1 [10.2–18.5] | 99.91 [99.38–100.70] MiB (n=9) | 10.75 MiB |

macOS does not expose process-lifetime ru_maxrss through the getrusage interface used here. The R host records current RSS snapshots but emits JSON null plus an explicit unavailability note for process peak RSS; it does not substitute an RSS snapshot and label it as a peak. Linux and Windows process peak RSS includes the R process and Wasmtime allocations, not WASM-only memory. WASM linear memory is reported separately.

The C shim times allocation, input copy, guest execution, input free, output copy, and output free. The table shows the median of the per-process phase medians for the 512×512 fixture; allocation/copy/free values are microseconds and guest execution is milliseconds.

| Platform | Allocation µs | Input copy µs | Guest execution ms | Input free µs | Output copy µs | Output free µs |
|---|---:|---:|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 61.5 | 114.3 | 10.86 | 1.58 | 0.85 | 0.31 |
| macOS 15 Intel x86-64 | 62.0 | 83.0 | 14.45 | 4.00 | 2.00 | 1.00 |
| Windows x86-64 GNU | 55.7 | 70.1 | 11.04 | 1.85 | 0.85 | 0.30 |

The process-based WASI RSS delta is only a snapshot of the already-running parent R or Node process; it does not measure the spawned Wasmtime child and is not a peak-memory measurement. The runtime CSV retains all 18 platform/fixture summaries, including Node embedded-host cold-start and memory metrics; raw per-process snapshots and phase sample arrays remain in the workflow artifacts.

## Runtime and binary sizes

These are component sizes: installed V8 package bytes, Wasmtime C API shared-library bytes, embedded ABI module bytes, and native Rust R binding library bytes. They are not complete end-user installation sizes or a CRAN dependency closure.

| Platform | V8 package | Wasmtime shared library | Embedded ABI module | Native Rust R library |
|---|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 49,824,147 B (47.5 MiB) | 98,429,356 B (93.9 MiB) | 147,742 B | 799,488 B (0.8 MiB) |
| macOS 15 Intel x86-64 | 41,039,865 B (39.1 MiB) | 68,689,112 B (65.5 MiB) | 147,730 B | 664,896 B (0.6 MiB) |
| Windows x86-64 GNU | 36,193,559 B (34.5 MiB) | 107,662,025 B (102.7 MiB) | 147,750 B | 2,328,919 B (2.2 MiB) |

## Interpretation

1. **Native Rust was fastest by median on all six fixtures and all three platforms.** On 512×512 it measured 5.19 ms/call on Ubuntu, 9.42 ms/call on Intel macOS, and 5.90 ms/call on Windows.
2. **There is no universal winner between V8 and Wasmtime.** Their relative throughput crosses by fixture and platform. On 512×512, V8 was slightly faster than Wasmtime on Ubuntu and Windows; Wasmtime was faster on Intel macOS.
3. **The native-R reference was much slower at this JSON validation boundary**, especially on 512×512 (median 245.84–437.67 ms/call). This is not a full plotting benchmark and must not be generalized to all production R workloads.
4. **The runtime dominates the module size.** The embedded ABI module is about 144 KiB, while the Wasmtime shared library is 65.5–102.7 MiB and the installed V8 package is 34.5–47.5 MiB. The native Rust R binding library is 0.63–2.22 MiB.
5. **Process-based WASI is a portability/feasibility baseline**, not the preferred high-throughput interface: each invocation pays spawned-process startup and JSON/stdin transport costs. Embedded Wasmtime and V8 avoid process startup but still carry substantial runtime dependencies.

## Limitations and decision

- The experiment measures the shared validation boundary and synthetic JSON fixtures, not corrplot rendering or end-to-end package workflows.
- Per-run 95% intervals describe within-run batch variation. Independent runner repetitions are summarized separately with medians and ranges; they are not pooled confidence intervals. Windows shows noticeable runner-to-run variation.
- Process peak RSS includes R and Wasmtime allocations and is not WASM-only memory. Darwin peak RSS is unavailable with the current getrusage method.
- Process-based parent RSS snapshots do not include the spawned Wasmtime child.
- Runtime/library sizes are component measurements, not complete package installation or dependency-closure measurements.
- The native Rust R binding remains an isolated experiment. This matrix does not establish CRAN acceptance or remove the remaining vendor-metadata NOTE tracked by #268.

**Decision: retain native Rust as the R integration direction and WASM for Node/browser consumers.** This is evidence for the runtime/distribution choice at the tested validation boundary, not authorization to migrate or release the production R package. Keep the R experiment isolated until production-package installation, dependency/license inventory, offline installation, R CMD check --as-cran, supported-platform behavior, and authoritative CRAN guidance are separately resolved (#173 and #268).

## Workflow artifacts

Workflow run: [#41981917849](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849) — 9/9 runner jobs successful (attempt 1).

| Platform | Runner repetition | Raw artifact |
|---|---:|---|
| Ubuntu Linux x86-64 | 1 | [artifact 13393979840](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13393979840) |
| Ubuntu Linux x86-64 | 2 | [artifact 13394368603](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394368603) |
| Ubuntu Linux x86-64 | 3 | [artifact 13394164138](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394164138) |
| macOS 15 Intel x86-64 | 1 | [artifact 13394289222](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394289222) |
| macOS 15 Intel x86-64 | 2 | [artifact 13394378735](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394378735) |
| macOS 15 Intel x86-64 | 3 | [artifact 13394975895](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394975895) |
| Windows x86-64 GNU | 1 | [artifact 13394537918](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394537918) |
| Windows x86-64 GNU | 2 | [artifact 13394642505](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394642505) |
| Windows x86-64 GNU | 3 | [artifact 13394438472](https://github.com/vsimko/corrplot-ng/actions/runs/41981917849/artifacts/13394438472) |

Machine-readable summaries:
- [benchmark-2026-11-05.csv](benchmark-2026-11-05.csv) — 72 platform/fixture/backend rows with median, min–max, and median within-run 95% interval endpoints; raw per-run samples remain in the artifact ZIPs.
- [benchmark-2026-11-05-runtime.csv](benchmark-2026-11-05-runtime.csv) — 18 platform/fixture summaries aggregating three runner repetitions and nine embedded-R process runs for cold-start, memory, and transfer-phase metrics; raw phase samples remain in the artifact JSON.
