# Cross-language scene-v1 benchmark

This experimental benchmark compares the same scene-v1 semantics for Python,
pure R, C++/Rcpp and Rust/rextendr. All candidates use identical deterministic
inputs and must pass a recursive comparison against the Python-generated
expected scene before their timings are accepted.

## Workloads

- B01: 20x20 dense symmetric matrix.
- B02: 100x100 dense symmetric matrix.
- B05: 100x100 with deterministic 10% symmetric missing values.
- B06: 100x100 with long labels and annotation.
- B07: 100x100 with a fixed variable permutation.

B03/B04 stress matrices and B08 SVG serialization are outside this vertical
slice. The scene-v1 contract remains an experimental prototype, not a complete
corrplot implementation.

## Timing methodology

Each measured sample is a calibrated batch. The harness doubles the batch size
until a sample takes at least 100 ms, then records 30 samples after five warmups.
Python uses time.perf_counter_ns; R uses elapsed time from proc.time over the
same calibrated-batch approach. Results include the selected batch size,
all raw samples, first-call latency, median, IQR, minimum and maximum.

The Python and pure-R runners report scene construction only, JSON encoding
only, and end-to-end construction plus encoding. C++/Rcpp and Rust/rextendr
report end-to-end construction plus JSON encoding through their R bindings.
Compare end-to-end results across all four candidates; compare isolated phases
only between Python and pure R. R's elapsed timer has coarser resolution than
Python's high-resolution clock, but calibrated batches reduce timer quantization.

The target sample duration and sample count are fixed across candidates. Batch
sizes may differ because candidate speeds differ. Runtime measurements exclude
package compilation and installation; those are separate packaging evidence.

## Run

    python benchmarks/export_native_fixtures.py --output benchmark-results/native-fixtures.json
    python benchmarks/run_python_scene_v1_reference.py --fixtures benchmark-results/native-fixtures.json --output benchmark-results/python-scene-v1.json
    Rscript -e 'install.packages(c("Rcpp", "jsonlite", "rextendr", "devtools", "usethis"))'
    Rscript -e 'Rcpp::compileAttributes("prototypes/cpp-rcpp")'
    R CMD INSTALL prototypes/cpp-rcpp
    Rscript prototypes/rust-rextendr/prepare-r-package.R
    R CMD INSTALL prototypes/rust-rextendr/.ci-r-package
    Rscript benchmarks/run_native_scene_v1.R benchmark-results/native-fixtures.json benchmark-results/native-scene-v1.json

CI runs the same workflow on a common Ubuntu runner and uploads fixtures and
machine-readable results as a 90-day artifact.

## Interpretation and limitations

- Correctness is a hard gate; a mismatch stops the run rather than reporting
  misleading performance.
- JSON serializers are language-specific. The end-to-end measurement represents
  the complete adapter path, not a pure language-runtime comparison.
- Isolated Python/R phase timings help identify where their costs occur.
- The benchmark does not select a production language, establish Windows or
  macOS binary-install readiness, or justify changing the legacy R package.
- One CI runner supports controlled relative comparison, not a universal
  performance claim. Repeat on other operating systems before deciding.

## Initial CI results (2026-10-30)

The first complete run on the final PR #37 head was
[workflow run 41045222556](https://github.com/vsimko/corrplot-ng/actions/runs/41045222556),
commit `eed349992251c58cca788750fab50086e7bb2965`. All 25 native/R measurements
and all 15 Python measurements passed their correctness gates. The run used one
Ubuntu x86_64 GitHub-hosted runner, R 4.6.2, Rcpp 1.1.2 and rextendr 0.5.0.

End-to-end median milliseconds (scene construction plus JSON encoding):

| Workload | Python | Pure R | C++/Rcpp | Rust/rextendr |
|---|---:|---:|---:|---:|
| B01, 20x20 | 0.942 | 241.0 | 0.758 | 0.715 |
| B02, 100x100 | 23.316 | 5792.0 | 18.750 | 17.750 |
| B05, 10% missing | 22.968 | 5761.0 | 18.375 | 17.625 |
| B06, long labels | 23.544 | 5745.0 | 18.875 | 17.875 |
| B07, reorder | 23.432 | 5730.0 | 18.625 | 17.750 |

For pure R, the isolated scene-construction median on B02–B07 workloads is about
36–39 ms, while JSON encoding alone is about 5.6 seconds. This points to the
current R serialization path as the dominant cost in this prototype; it should
not be interpreted as an inherent limit of the R language.

In these workloads Rust/rextendr is about 5–7% faster than C++/Rcpp, and the two
native prototypes are roughly 20–25% faster than Python end-to-end. The gap
between Rust and C++ is small relative to the scope of this experiment. These
are implementation-specific results, not a production-language decision: the
benchmark covers only scene-v1, one Linux environment and five workloads. The
Python and R JSON encoders also produce different serialized byte counts even
when the parsed scene passes the semantic correctness comparison.

The machine-readable inputs and raw measurements are retained in the workflow
artifact linked above. Re-run on Windows and macOS, and compare install/package
readiness, API ergonomics, maintenance cost and full corrplot feature coverage
before choosing a production shared core.
