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
| valid-small | 0.05760 | 0.00240 | 0.00320 | 0.10560 |
| valid-general-32 | 0.10240 | 0.03620 | 0.02520 | 1.44410 |
| valid-general-128 | 0.67390 | 0.52160 | 0.30800 | 13.27000 |
| valid-correlation-64 | 0.24100 | 0.15040 | 0.08990 | 5.81000 |
| invalid-asymmetric | 0.07320 | 0.00330 | 0.00420 | 0.07400 |

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
| valid-small | 0.13760 | 0.00680 | 0.01680 | 0.87130 |
| valid-general-32 | 0.28000 | 0.08040 | 0.08310 | 19.63000 |
| valid-general-128 | 2.22750 | 1.46760 | 0.84400 | 65.40000 |
| valid-correlation-64 | 0.77400 | 0.46900 | 0.18180 | 20.15000 |
| invalid-asymmetric | 0.23500 | 0.04730 | 0.01560 | 0.49520 |

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

CI run: https://github.com/vsimko/corrplot-ng/actions/runs/41342721347
