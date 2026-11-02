# R backend benchmark results — 2026-11-01 CI run

## Scope and method

Exploratory microbenchmarks compare three R host integrations for the same Rust validation core:

- **V8:** R package `V8` instantiating the host-neutral `wasm32-unknown-unknown` module.
- **Wasmtime:** R `.Call` shim embedding Wasmtime C API 50.0.1 and invoking that module.
- **Native Rust:** `extendr` cdylib calling the same Rust core directly.
- **Native R reference:** equivalent JSON decode/validation boundary, included as a baseline.

Each platform ran six identical JSON fixtures: small valid input, general numeric matrices of 32×32, 128×128 and 512×512, a 64×64 correlation matrix, and an invalid asymmetric matrix. The R runners warm up, pilot, then execute ten timed samples with adaptive call counts. Values below are mean milliseconds per call with the runner's 95% confidence interval. These are single-run CI microbenchmarks, not independent repeated machine-level trials. Measurements include host/ABI and JSON boundary costs.

The three CI jobs on the same commit completed successfully. All six fixture response payloads matched across the seven collected implementations/runners on each platform (Node WASI, Node embedded WASM, R WASI CLI, R Wasmtime C API, R V8, R native Rust and R native reference).

## Results — milliseconds per call (95% CI)

### Ubuntu latest — Linux x86-64

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0936 (0.0908–0.0964) | 0.0036 (0.0036–0.0036) | 0.0021 (0.0020–0.0021) | 0.1618 (0.1598–0.1637) |
| valid-general-32 | 0.1545 (0.1516–0.1575) | 0.0568 (0.0563–0.0572) | 0.0300 (0.0299–0.0301) | 1.9063 (1.8852–1.9273) |
| valid-general-128 | 0.9586 (0.9437–0.9735) | 0.8225 (0.8209–0.8241) | 0.4213 (0.4207–0.4219) | 19.8918 (19.7205–20.0632) |
| valid-general-512 | 12.5278 (12.5113–12.5444) | 12.2390 (12.2272–12.2508) | 5.7832 (5.7745–5.7920) | 253.9750 (249.9747–257.9753) |
| valid-correlation-64 | 0.3217 (0.3171–0.3262) | 0.2428 (0.2424–0.2433) | 0.1190 (0.1189–0.1190) | 8.2410 (8.1417–8.3402) |
| invalid-asymmetric | 0.1071 (0.1041–0.1100) | 0.0043 (0.0042–0.0044) | 0.0025 (0.0025–0.0025) | 0.1246 (0.1232–0.1259) |

Runtime/library sizes: V8 installed package 49,824,152 B (47.5 MiB); Wasmtime shared library 98,429,356 B (93.9 MiB); embedded WASM module 147,076 B (143.6 KiB); native Rust library 787,392 B (769 KiB).

### macOS 15 Intel — Darwin x86-64

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.1438 (0.1349–0.1526) | 0.0073 (0.0072–0.0074) | 0.0059 (0.0052–0.0066) | 0.5173 (0.4988–0.5357) |
| valid-general-32 | 0.2819 (0.2597–0.3041) | 0.1083 (0.0971–0.1195) | 0.0563 (0.0536–0.0590) | 6.1806 (5.8952–6.4660) |
| valid-general-128 | 1.9763 (1.8307–2.1219) | 1.3083 (1.2242–1.3923) | 0.9621 (0.7513–1.1730) | 64.3533 (54.1848–74.5218) |
| valid-general-512 | 25.7725 (24.8278–26.7172) | 21.0949 (19.0205–23.1693) | 13.6487 (13.1728–14.1246) | 545.6500 (523.4579–567.8421) |
| valid-correlation-64 | 0.7210 (0.6826–0.7593) | 0.3956 (0.3699–0.4213) | 0.2284 (0.2097–0.2471) | 19.7462 (19.0815–20.4108) |
| invalid-asymmetric | 0.1936 (0.1786–0.2086) | 0.0101 (0.0090–0.0112) | 0.0061 (0.0057–0.0066) | 0.3870 (0.3672–0.4069) |

Runtime/library sizes: V8 installed package 41,039,865 B (39.1 MiB); Wasmtime shared library 68,689,112 B (65.5 MiB); embedded WASM module 147,079 B (143.6 KiB); native Rust library 664,392 B (649 KiB).

### Windows latest — x86-64 / GNU toolchain

| Fixture | V8 | Wasmtime | Native Rust | Native R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.0804 (0.0802–0.0806) | 0.0037 (0.0037–0.0037) | 0.0024 (0.0023–0.0024) | 0.4700 (0.4617–0.4781) |
| valid-general-32 | 0.1355 (0.1300–0.1410) | 0.0586 (0.0586–0.0587) | 0.0324 (0.0324–0.0325) | 5.7033 (5.6345–5.7721) |
| valid-general-128 | 0.8700 (0.8593–0.8807) | 0.8611 (0.8555–0.8668) | 0.4360 (0.4283–0.4437) | 41.2083 (40.6742–41.7425) |
| valid-general-512 | 10.1386 (10.0555–10.2218) | 12.4000 (12.3178–12.4822) | 5.8856 (5.8250–5.9462) | 437.0000 (428.4062–445.5938) |
| valid-correlation-64 | 0.2810 (0.2769–0.2851) | 0.2525 (0.2512–0.2538) | 0.1235 (0.1232–0.1238) | 17.2931 (17.0998–17.4864) |
| invalid-asymmetric | 0.0986 (0.0974–0.0998) | 0.0043 (0.0043–0.0044) | 0.0028 (0.0028–0.0028) | 0.3360 (0.3263–0.3457) |

Runtime/library sizes: V8 installed package 36,193,559 B (34.5 MiB); Wasmtime shared library 107,662,025 B (102.7 MiB); embedded WASM module 147,086 B (143.6 KiB); native Rust DLL 2,010,425 B (1.9 MiB). The Windows artifact did not report an R version string in `runtime-sizes.txt`; this does not affect the successful CI status.

## Interpretation

1. **Native Rust was fastest in all six fixture cases on all three CI platforms.** The gap is largest for larger matrices; on the 512×512 fixture it is roughly 2× faster than either WASM host.
2. **Wasmtime generally outperformed V8**, especially on small inputs where the V8/JavaScript boundary is expensive. Windows 512×512 is an exception: Wasmtime was slower than V8 in this run.
3. **Both WASM-in-R approaches have a significant runtime footprint.** Wasmtime's shared library alone ranged from about 65.5 MiB on Intel macOS to 102.7 MiB on Windows. The WASM module itself is about 144 KiB.
4. **The native R reference is substantially slower** under this equivalent JSON boundary, especially as matrix size grows. This is a microbenchmark of the validation boundary, not a full plotting benchmark.

## Limitations and remaining acceptance work

- These are one CI run per platform. The per-run 95% intervals describe sample variation within that run, not between-run or between-machine reproducibility.
- The measured R backend numbers are warm repeated-call throughput. Cold startup / first-call latency is not isolated.
- Peak RSS, guest linear-memory high-water mark, and a complete installed dependency closure are not measured.
- Input/output transfer is included in the end-to-end JSON call timing but is not separately instrumented.
- Package/runtime sizes are individual installed package/library/module measurements, not full end-user installation sizes.
- The Windows R version string was not emitted by the size-reporting script and should be captured explicitly in a follow-up.
- The benchmark is limited to the shared validation operation and synthetic fixtures; it does not measure corrplot rendering or end-to-end package workflows.

## Artifacts and workflow

The latest successful workflow is [WASM from R experiment #41353730560](https://github.com/vsimko/corrplot-ng/actions/runs/41353730560). It produced per-platform artifacts:

- [Ubuntu benchmark artifact](https://github.com/vsimko/corrplot-ng/actions/runs/41353730560/artifacts/13102721463)
- [macOS Intel benchmark artifact](https://github.com/vsimko/corrplot-ng/actions/runs/41353730560/artifacts/13102408900)
- [Windows benchmark artifact](https://github.com/vsimko/corrplot-ng/actions/runs/41353730560/artifacts/13106325899)

The separate R package checks and foundation test workflows on the same commit also passed. This remains an isolated feasibility experiment and does not change the production backend strategy.
