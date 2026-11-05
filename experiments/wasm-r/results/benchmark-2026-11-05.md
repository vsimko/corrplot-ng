# R-host WASM benchmark results — 2026-11-05

## Scope and reproducibility

The manual workflow run [#41861322600](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600) completed successfully with **9/9 runner jobs green**: three independent repetitions each on Ubuntu, Intel macOS, and Windows. The original run had one cancelled Intel macOS repetition; only that job was rerun (workflow attempt 4), and its artifact completed successfully. The measurements ran against commit [`57304bced6fd24819c3c86a758b0786e3e43fbe9`](https://github.com/vsimko/corrplot-ng/commit/57304bced6fd24819c3c86a758b0786e3e43fbe9), after the Darwin peak-RSS and validator fixes in PRs #333–#335.

Each runner:
- built the `wasm32-wasip1` process-based module and the `wasm32-unknown-unknown` host-neutral ABI module from the shared Rust core;
- ran six fixtures: small valid, general numeric 32×32, 128×128 and 512×512, 64×64 correlation, and invalid asymmetric;
- compared Node and R module/fixture SHA-256 values and semantic responses for both WASI and embedded ABI paths;
- compared the embedded Wasmtime, V8, native Rust, and native-R reference responses;
- recorded ten warm throughput samples per backend/fixture, with per-run 95% Student-t intervals;
- launched three separate Rscript processes per fixture for embedded Wasmtime initialization, first-call, memory, and transfer-phase measurements.

The nine artifacts contain the raw per-fixture JSON, ten-sample warm series, and three separate embedded-R process repetitions. The checked-in CSVs provide compact platform/fixture summaries across the three runner repetitions. Raw per-run means, 95% intervals, and per-process samples remain in the artifact ZIPs. Cross-run summaries use medians and ranges; they are **not pooled 95% confidence intervals**.

## Platform and toolchain matrix

| Platform | Runner | R | Node | Rust compiler | Wasmtime | V8 package / engine |
|---|---|---|---|---|---|---|
| Ubuntu Linux x86-64 | ubuntu-latest | 4.6.2 (2026-10-29) | v24.22.1 | 1.99.1 (58cf8f9c3, 2026-10-13) | Wasmtime 50.0.1 | V8 8.2.1 / engine 12.4.254.21 |
| macOS 15 Intel x86-64 | macos-15-intel | 4.6.2 (2026-10-29) | v24.19.0 | 1.99.1 (58cf8f9c3, 2026-10-13) | Wasmtime 50.0.1 | V8 8.2.1 / engine 11.7.439.14 |
| Windows x86-64 GNU | windows-latest | 4.6.2 (2026-10-29 ucrt) | v24.22.2 | 1.99.1 (58cf8f9c3, 2026-10-13) | Wasmtime 50.0.1 | V8 8.2.1 / engine 14.6.202.33 |

All three platforms used Wasmtime CLI and C API **50.0.1**. The C API release archives were SHA-256 verified before use:

| Platform | C API archive SHA-256 |
|---|---|
| Ubuntu Linux x86-64 | `e2c0c26e1aef52f82f0b9918ad6f34cc30f4b557aee3e261cb127a9462be8f4d` |
| macOS 15 Intel x86-64 | `b6d436cba5c0964785c6ff7c9b5a743131555a0227b4f742185c4fb520c502c5` |
| Windows x86-64 GNU | `9001a5df99305f6bc4a22d1e38f020f953b26cc88755840743dfabd44d9eaea4` |

The Windows `runtime-sizes.txt` in this run has an empty `R:` summary line because `R --version` did not emit through that shell invocation. The per-fixture JSON and job logs independently record **R 4.6.2 (2026-10-29 ucrt)**. The follow-up workflow change in this PR records `Rscript -e 'cat(R.version.string)'` and also emits Node, Rust, Cargo, Wasmtime CLI, and C API archive versions/hashes into future artifacts.

## Cross-runtime identity and parity

All nine artifacts passed the workflow's semantic parity assertions for all six fixtures:
- Node and R used matching SHA-256 values for the process-based WASI module and the fixture.
- Node and R used matching SHA-256 values for the embedded ABI module and the fixture.
- The three independent Rscript embedded-Wasmtime processes per fixture returned the same semantic response as Node's embedded ABI runner.
- V8, native Rust, and native-R reference responses matched the embedded ABI result.

The module builds are **not byte-identical across operating systems**; each platform builds its own module bytes. The guarantee demonstrated here is that Node and R use the exact same bytes **within each platform/repetition**, not that Linux, macOS, and Windows artifacts share one cross-platform hash.

| Platform | WASI module SHA-256 (Node = R) | Embedded ABI SHA-256 (Node = R) | Generated 512×512 fixture SHA-256 |
|---|---|---|---|
| Ubuntu Linux x86-64 | `baf63002f4e31b86ee7c9a1fb7cc561cbdaf4c9743995647164f696dfc0d3802` | `5eb4a54893403650cc820abb2714fc8fb333ec5af674a4140c48f9874aeb3ba4` | `f52fdcc1eb1d234e04c98456afa4484dda6baf17c8ba7b162c28867e69efb6b1` |
| macOS 15 Intel x86-64 | `6c24e7eaffb5257d5139eb56682f356dff9f785a09c48629141ff028e947f49a` | `ae4089bb00a992f498351f3d6b9d0fb8dcf6665ebf0d05d83748013dd939d1dc` | `f52fdcc1eb1d234e04c98456afa4484dda6baf17c8ba7b162c28867e69efb6b1` |
| Windows x86-64 GNU | `6737ba90bb25d2ddfebc9e9fef78b23746a978255af60767a50ee2e05c1b3b6c` | `a670cf83ea8dc569cfb28520d83ecdfa5b909845962a47730463d6d3a2ed768c` | `f52fdcc1eb1d234e04c98456afa4484dda6baf17c8ba7b162c28867e69efb6b1` |

The generated 512×512 fixture has the same byte hash on all three platforms. The five committed fixture files have different Windows hashes from Linux/macOS, consistent with CRLF checkout conversion; Node and R still hash the same fixture bytes within each platform and all semantic responses match. This PR adds a fixture-specific `.gitattributes` rule to force LF checkout for future Windows runs.

## Warm throughput

Values are median per-call milliseconds across the three independent runner repetitions. Bracketed values show the min–max of the three runner means. [`benchmark-2026-11-05.csv`](benchmark-2026-11-05.csv) contains the median, min–max range, and median within-run 95% interval across the three runner means. The raw per-run means, call counts, ten individual timing values, and their intervals remain in the corresponding artifact ZIPs.

### Ubuntu Linux x86-64

Median mean ms/call across three independent runner repetitions; bracketed values are the min–max of the three runner means. Each runner's own 95% CI is retained in the CSV.

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0897 [0.0870–0.0919] | 0.0040 [0.0039–0.0040] | 0.0020 [0.0019–0.0020] | 0.1564 [0.1496–0.1573] |
| valid-general-32 | 0.1485 [0.1435–0.1490] | 0.0571 [0.0566–0.0572] | 0.0301 [0.0299–0.0301] | 1.8577 [1.7815–1.8854] |
| valid-general-128 | 0.9304 [0.9283–0.9316] | 0.8197 [0.8187–0.8277] | 0.4245 [0.4240–0.4436] | 19.0192 [18.1904–19.4200] |
| valid-general-512 | 11.7274 [11.6791–11.7859] | 12.0952 [12.0795–12.3061] | 5.6249 [5.6000–5.6302] | 249.6500 [244.6600–254.7250] |
| valid-correlation-64 | 0.3248 [0.3177–0.3281] | 0.2370 [0.2369–0.2391] | 0.1209 [0.1205–0.1212] | 8.2210 [7.8377–8.3909] |
| invalid-asymmetric | 0.1066 [0.1038–0.1098] | 0.0046 [0.0046–0.0047] | 0.0025 [0.0024–0.0025] | 0.1278 [0.1201–0.1295] |

### macOS 15 Intel x86-64

Median mean ms/call across three independent runner repetitions; bracketed values are the min–max of the three runner means. Each runner's own 95% CI is retained in the CSV.

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.1043 [0.0878–0.1275] | 0.0066 [0.0063–0.0081] | 0.0033 [0.0032–0.0035] | 0.3575 [0.3473–0.3869] |
| valid-general-32 | 0.2441 [0.1912–0.2773] | 0.0987 [0.0694–0.1140] | 0.0459 [0.0381–0.0538] | 4.3670 [4.0974–4.9765] |
| valid-general-128 | 2.2368 [1.2005–2.4545] | 1.2721 [0.9198–1.6374] | 0.7688 [0.5004–0.8193] | 48.6286 [32.5207–49.7526] |
| valid-general-512 | 19.2296 [15.6431–20.8932] | 15.6821 [13.1300–16.8873] | 11.0011 [10.0957–11.7258] | 405.0333 [393.6333–487.6667] |
| valid-correlation-64 | 0.6452 [0.5806–0.8075] | 0.4132 [0.3527–0.5446] | 0.2259 [0.1896–0.2508] | 17.3944 [15.0950–26.4256] |
| invalid-asymmetric | 0.2258 [0.2196–0.2264] | 0.0118 [0.0103–0.0146] | 0.0077 [0.0062–0.0077] | 0.4216 [0.3262–0.4742] |

### Windows x86-64 GNU

Median mean ms/call across three independent runner repetitions; bracketed values are the min–max of the three runner means. Each runner's own 95% CI is retained in the CSV.

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0813 [0.0351–0.0833] | 0.0041 [0.0031–0.0042] | 0.0023 [0.0016–0.0024] | 0.4858 [0.2722–0.5610] |
| valid-general-32 | 0.1313 [0.0627–0.1347] | 0.0595 [0.0400–0.0596] | 0.0325 [0.0198–0.0329] | 5.6527 [3.4643–5.9740] |
| valid-general-128 | 0.8530 [0.4886–0.8600] | 0.8556 [0.5709–0.8822] | 0.4265 [0.2672–0.4302] | 40.9583 [27.3243–43.1304] |
| valid-general-512 | 10.1978 [5.7527–10.5055] | 12.3000 [8.7358–13.2625] | 5.9042 [3.3706–6.3077] | 437.0000 [316.2500–451.6667] |
| valid-correlation-64 | 0.2869 [0.1614–0.2906] | 0.2526 [0.1572–0.2552] | 0.1236 [0.0748–0.1286] | 16.9508 [11.7126–18.3898] |
| invalid-asymmetric | 0.0974 [0.0450–0.1023] | 0.0048 [0.0035–0.0052] | 0.0027 [0.0020–0.0028] | 0.3367 [0.1812–0.3536] |

## Cold start, first call, memory, and transfer

The following table uses the 512×512 fixture. The process-based values time one spawned `wasmtime run` invocation from an already-running R or Node host; they include child-process startup and execution, not Rscript/Node startup. There is one observation per independent runner (n=3). Embedded Wasmtime initialization and first-call values aggregate the three separate Rscript processes in each of the three runner repetitions (n=9). Values are median [min–max].

| Platform | R-host Wasmtime CLI invocation ms | Node-host Wasmtime CLI invocation ms | Embedded Wasmtime init ms | Embedded first-call ms | R process peak RSS after first call | WASM linear memory after first call |
|---|---:|---:|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 30.0 [24.0–34.0] | 28.8 [26.3–38.8] | 46.2 [45.4–47.7] | 18.3 [18.1–23.8] | 105.3 [104.6–105.5] MiB (n=9) | 10.75 MiB |
| macOS 15 Intel x86-64 | 41.0 [38.0–55.0] | 44.9 [34.6–49.4] | 55.2 [40.2–90.5] | 18.6 [16.8–21.4] | Unavailable on Darwin; explicit JSON null and note (n=9) | 10.75 MiB |
| Windows x86-64 GNU | 40.0 [40.0–50.0] | 36.5 [25.1–41.2] | 51.9 [32.2–74.0] | 16.3 [10.5–17.2] | 100.1 [91.1–100.5] MiB (n=9) | 10.75 MiB |

macOS does not expose process-lifetime `ru_maxrss` through the `getrusage` interface used here. The runner records current R-process RSS snapshots and emits JSON `null` plus an explicit note for peak RSS; it does not substitute a snapshot and label it as a peak. Linux and Windows peak RSS includes the R process and Wasmtime allocations, not WASM-only memory. WASM linear memory is reported separately.

The C shim times allocation, input copy, guest execution, input free, output copy, and output free. The next table shows the median of the per-process phase medians for the 512×512 fixture; allocation/copy/free values are microseconds, guest execution is milliseconds.

| Platform | Allocation µs | Input copy µs | Guest execution ms | Input free µs | Output copy µs | Output free µs |
|---|---:|---:|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 28.8 | 52.6 | 11.92 | 0.62 | 0.28 | 0.25 |
| macOS 15 Intel x86-64 | 68.5 | 91.5 | 14.65 | 4.00 | 2.00 | 1.00 |
| Windows x86-64 GNU | 28.3 | 53.4 | 11.96 | 0.70 | 0.30 | 0.30 |

The process-based WASI RSS delta is only a snapshot of the already-running parent R or Node process; it does not measure the spawned Wasmtime child and is not a peak measurement. Platform/fixture summaries across three runner repetitions and nine embedded-R processes are retained in [`benchmark-2026-11-05-runtime.csv`](benchmark-2026-11-05-runtime.csv); raw per-process snapshots and phase sample arrays remain in the linked workflow artifacts.

## Runtime and binary sizes

These are installed V8 package bytes, the Wasmtime C API shared-library bytes, the embedded ABI module bytes, and the native Rust R binding library bytes. They are **component sizes**, not complete end-user installation sizes or a CRAN dependency closure.

| Platform | V8 package | Wasmtime shared library | Embedded ABI module | Native Rust R library |
|---|---:|---:|---:|---:|
| Ubuntu Linux x86-64 | 49,824,145 B (47.5 MiB) | 98,429,356 B (93.9 MiB) | 147,742 B | 799,488 B (0.8 MiB) |
| macOS 15 Intel x86-64 | 41,039,865 B (39.1 MiB) | 68,689,112 B (65.5 MiB) | 147,730 B | 664,896 B (0.6 MiB) |
| Windows x86-64 GNU | 36,193,559 B (34.5 MiB) | 107,662,025 B (102.7 MiB) | 147,750 B | 2,328,919 B (2.2 MiB) |

## Interpretation

1. **Native Rust was fastest in the median for all six fixtures on all three platforms.** On the 512×512 fixture, the median was 5.62 ms/call on Ubuntu, 11.00 ms/call on Intel macOS, and 5.90 ms/call on Windows.
2. **There is no universal winner between V8 and Wasmtime.** Their relative throughput crosses by fixture and platform. On the 512×512 case, V8 was slightly faster than Wasmtime on Ubuntu and Windows; Wasmtime was faster on Intel macOS.
3. **The native R reference was much slower at this JSON validation boundary**, especially on the 512×512 fixture (median 249.65–437.00 ms/call). This is not a full plotting benchmark and should not be generalized to all production R work.
4. **The runtime dominates the module size.** The embedded ABI module is about 144 KiB, while the Wasmtime shared library is 65.5–102.7 MiB and the installed V8 package is 34.5–47.5 MiB. The native Rust R binding is 0.63–2.22 MiB.
5. **The process-based WASI route is a portability/feasibility baseline**, not the preferred high-throughput interface: it pays spawned-process startup and JSON/stdin transport costs. The embedded C API and V8 paths avoid process startup but still carry substantial runtime dependencies.

## Limitations and decision

- The experiment measures the shared validation boundary and synthetic JSON fixtures, not corrplot rendering or end-to-end package workflows.
- Per-run 95% intervals describe within-run sample variation. The independent runner repetitions are summarized separately with medians and ranges; Windows shows noticeable runner-to-run variation.
- Process peak RSS includes R and Wasmtime allocations and is not a WASM-only measure. macOS peak RSS is unavailable with the current `getrusage` method.
- The process-based parent RSS snapshots do not include the spawned Wasmtime child's memory.
- Runtime/library sizes are component measurements, not full package installation or dependency-closure measurements.
- The native Rust R binding remains an experiment, not a production R package. This matrix does not establish CRAN acceptance or remove the remaining vendor-metadata NOTE tracked by #268.

**Decision: retain native Rust as the R integration direction and WASM for Node/browser consumers.** This is evidence for the distribution/runtime choice at the tested validation boundary, not authorization to migrate or release the production R package. Keep the R experiment isolated until production-package installation, dependency/license inventory, offline installation, `R CMD check --as-cran`, supported-platform behavior, and authoritative CRAN guidance are separately resolved (#173 and #268).

## Workflow artifacts

Workflow run: [#41861322600](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600) — 9/9 runner jobs successful (final workflow attempt 4).

| Platform | Runner repetition | Raw artifact |
|---|---:|---|
| Ubuntu Linux x86-64 | 1 | [13341635351](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13341635351) |
| Ubuntu Linux x86-64 | 2 | [13341341094](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13341341094) |
| Ubuntu Linux x86-64 | 3 | [13339499941](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13339499941) |
| macOS 15 Intel x86-64 | 1 | [13371570926](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13371570926) |
| macOS 15 Intel x86-64 | 2 | [13371173975](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13371173975) |
| macOS 15 Intel x86-64 | 3 | [13377458818](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13377458818) |
| Windows x86-64 GNU | 1 | [13341910486](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13341910486) |
| Windows x86-64 GNU | 2 | [13340332514](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13340332514) |
| Windows x86-64 GNU | 3 | [13341315784](https://github.com/vsimko/corrplot-ng/actions/runs/41861322600/artifacts/13341315784) |

Machine-readable summaries:
- [`benchmark-2026-11-05.csv`](benchmark-2026-11-05.csv) — 72 platform/fixture/backend rows with median, min–max, and median within-run 95% intervals; raw per-run samples remain in the artifact ZIPs.
- [`benchmark-2026-11-05-runtime.csv`](benchmark-2026-11-05-runtime.csv) — 18 platform/fixture summaries aggregating three runner repetitions and nine embedded-R process runs for cold-start, memory, and transfer-phase metrics; raw phase samples remain in the artifact JSON.
