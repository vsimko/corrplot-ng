# Cross-language scene-v1 benchmark

This is the first comparable native vertical slice for the shared-core decision.
It benchmarks scene construction **plus JSON encoding** for Python, C++/Rcpp and
Rust/rextendr using identical generated inputs and the same expected scene-v1
output.

## Scope

- B01: 20x20 dense symmetric matrix.
- B02: 100x100 dense symmetric matrix.
- B05: 100x100 with deterministic 10% symmetric missing values.
- B06: 100x100 with long labels and annotation.
- B07: 100x100 with a fixed variable permutation.

B08 (SVG serialization) and B03/B04 (large stress matrices) are intentionally
outside this native vertical slice. The current scene-v1 contract is an
experimental prototype, not a complete corrplot implementation.

## Local execution

Prerequisites: Python 3.12, R, Rcpp, jsonlite, rextendr, devtools, usethis,
Rust stable, and a platform C/C++ toolchain.

```sh
python benchmarks/export_native_fixtures.py --output benchmark-results/native-fixtures.json
python benchmarks/run_python_scene_v1_reference.py \
  --fixtures benchmark-results/native-fixtures.json \
  --output benchmark-results/python-scene-v1.json --iterations 20

Rscript -e 'install.packages(c("Rcpp", "jsonlite", "rextendr", "devtools", "usethis"))'
Rscript -e 'Rcpp::compileAttributes("prototypes/cpp-rcpp")'
R CMD INSTALL prototypes/cpp-rcpp

Rscript prototypes/rust-rextendr/prepare-r-package.R
R CMD INSTALL prototypes/rust-rextendr/.ci-r-package

Rscript benchmarks/run_native_scene_v1.R \
  benchmark-results/native-fixtures.json \
  benchmark-results/native-scene-v1.json
```

The CI workflow performs these steps on the same Ubuntu runner and uploads
fixtures plus machine-readable results as a 90-day artifact.

## Correctness and timing

Each native result is parsed as JSON and compared recursively with the
Python-generated expected scene before any timing is accepted. A mismatch
fails the run; it is not reported as a faster result. The compared semantics
include dimensions, row-major cell order, numeric values/nulls, labels,
coordinates, annotation, and style.

All three candidates measure the same operation: scene construction plus JSON
encoding. The Python reference uses `build_scene` plus `json.dumps`; the
native candidates return the same JSON contract from their R bindings. Build
and installation time are deliberately excluded from runtime measurements and
must be assessed separately as packaging/maintenance evidence.

The harness records first-call latency, 3 warmups, 20 measured samples,
median/IQR, raw samples, scene cell count, output bytes, and runtime/toolchain
metadata. R uses batched calls to reduce timer quantization for small inputs.
Do not compare these results with the original Python-only core-construction
numbers from the initial benchmark suite: the measured operation differs.

The C++ and Rust packages remain isolated experimental adapters. This benchmark
does not select a production language, establish Windows binary-install
readiness, or justify changing the legacy R package.
