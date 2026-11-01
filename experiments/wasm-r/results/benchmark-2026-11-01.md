# R backend benchmark results — 2026-11-01

## Scope

Exploratory microbenchmarks of the three R host integrations for the shared Rust validation core:

- V8 package + host-neutral WebAssembly module
- Wasmtime C API + host-neutral WebAssembly module
- Native Rust cdylib + extendr

The pure-R implementation is included only as a reference baseline. Node.js and browser execution remain WASM consumers and are not part of this R-host comparison.

Both CI jobs used R 4.6.2 and the same five JSON fixtures. Each runner warmed up, ran a pilot, then selected a bounded number of repeated calls targeting approximately 0.5 seconds. Reported values are milliseconds per call. They include host/ABI and JSON-boundary overhead, not just the inner Rust function. These are single-run CI measurements, not statistically characterized benchmark results.

## Linux x86-64

Environment: Ubuntu 24.04, R 4.6.2, V8 8.2.1, Wasmtime 50.0.1.

| Fixture | V8 | Wasmtime | Native Rust | Pure R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.11460 | 0.00440 | 0.00500 | 0.14840 |
| valid-general-32 | 0.19700 | 0.06660 | 0.03240 | 1.81820 |
| valid-general-128 | 0.99580 | 0.82720 | 0.42000 | 17.87000 |
| valid-correlation-64 | 0.37770 | 0.24240 | 0.12160 | 8.29000 |
| invalid-asymmetric | 0.12900 | 0.00570 | 0.00480 | 0.12720 |

### Linux binary/package sizes

| Component | Bytes | Approximate size |
|---|---:|---:|
| Installed V8 R package (including its native engine) | 49,824,145 | 47.5 MiB |
| Wasmtime shared library | 98,429,356 | 93.9 MiB |
| Embedded WASM module | 147,076 | 143.6 KiB |
| Native Rust R library | 787,392 | 769 KiB |

## macOS Intel x86-64

Environment: macOS 15 Intel, R 4.6.2, V8 8.2.1, Wasmtime 50.0.1.

| Fixture | V8 | Wasmtime | Native Rust | Pure R reference |
|---|---:|---:|---:|---:|
| valid-small | 0.15520 | 0.00530 | 0.00700 | 0.37530 |
| valid-general-32 | 0.22800 | 0.06520 | 0.05050 | 4.09000 |
| valid-general-128 | 1.42170 | 0.93710 | 0.48600 | 32.75000 |
| valid-correlation-64 | 0.57190 | 0.26560 | 0.22060 | 16.54000 |
| invalid-asymmetric | 0.13200 | 0.00550 | 0.00900 | 0.25600 |

### macOS binary/package sizes

| Component | Bytes | Approximate size |
|---|---:|---:|
| Installed V8 R package (including its native engine) | 41,039,865 | 39.1 MiB |
| Wasmtime shared library | 68,689,112 | 65.5 MiB |
| Embedded WASM module | 147,079 | 143.6 KiB |
| Native Rust R library | 664,392 | 649 KiB |

## Interpretation

1. **Native Rust** is the fastest backend on the larger valid matrix fixtures on both platforms. Its advantage is less pronounced on tiny inputs, where R/native call overhead is a larger fraction of total time.
2. **Wasmtime** is consistently faster than V8 in these measurements. It wins some tiny-input cases, while native Rust wins the larger matrix cases.
3. **V8** has the largest per-call overhead in this prototype, particularly for small inputs. The R↔JavaScript conversion and JS↔WASM buffer boundary are included in the measurement.
4. **Runtime footprint** is substantial for both WASM-in-R choices. The installed V8 package is about 39–48 MiB; the Wasmtime shared library is about 66–94 MiB on these CI targets. The WASM module itself is only about 144 KiB.
5. **Native Rust** produces a much smaller R-side shared library in this prototype (about 649–769 KiB), but requires a platform-specific native build.

## Limitations and next measurements

- One CI run per platform; no confidence intervals or repeated independent runner samples.
- Warm repeated-call throughput is measured; cold-start / first-call latency is not yet measured.
- Memory peak/RSS and full installed dependency closure are not yet measured.
- Windows has not been tested.
- The native binding currently uses a JSON string boundary for parity. A typed R/native API could reduce overhead, but would no longer be an identical ABI comparison.
- Package sizes are installed V8 package directory, Wasmtime shared library file, Rust cdylib file, and WASM file; they are not complete end-user download sizes.

CI run: https://github.com/vsimko/corrplot-ng/actions/runs/41343130014
