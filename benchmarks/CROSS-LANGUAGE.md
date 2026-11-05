> **Historical report — 2026-10-30 prototype comparison.** This document preserves the original Python, pure-R, C++/Rcpp and Rust/rextendr benchmark evidence. The C++/Rcpp prototype was later rejected and retired by PR #332; the proposed maintainer-effort comparison was discontinued by #269. Historical measurements below are not current benchmark results, do not represent a Rust-only rerun, and must not be used to reopen the architecture decision.

# Cross-language scene-v1 benchmark

At the reviewed snapshot, this experimental benchmark compared the same scene-v1 semantics for Python, pure R, C++/Rcpp and Rust/rextendr. All candidates used identical deterministic inputs and passed a recursive comparison against the Python-generated expected scene before their timings were accepted.

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

## Historical run procedure

The original 2026-10-30 run used the following sequence. The C++/Rcpp setup and install commands have been retired and are intentionally omitted; they are not valid against current `main`. The historical workflow run links below preserve the original artifacts and measurements.

For the current Python, pure-R and Rust/rextendr benchmark workflow, follow [benchmarks/README.md](README.md) and `.github/workflows/cross-language-benchmarks.yml`. The current workflow no longer installs Rcpp or references the removed C++ prototype.

## Input validation parity

The R-facing implementations reject non-square/dimension-mismatched matrices,
correlation values outside [-1, 1], and infinite values. They use the same
error text for these invalid inputs. Missing R values (`NA`) remain valid and
serialize as JSON `null`; ordinary `NaN` values are rejected rather than being
silently treated as missing. Ordinary finite values preserve row-major cell
ordering. The Rust binding returns an error through extendr's default
`result_panic` behavior instead of returning an empty string, matching the
Rcpp and pure-R error contract. The Python reference rejects the corresponding
invalid configuration categories with `ValueError`; its wording remains
Python-specific.

Run the Python negative tests with:

    python -m unittest discover -s benchmarks -p 'test_scene_v1_*.py' -v

The R smoke harness exercises the same negative cases after both native
packages are installed.

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

## Follow-up CI results and packaging matrix (2026-10-30)

A follow-up run on the PR #38 head,
[workflow run 41051285476](https://github.com/vsimko/corrplot-ng/actions/runs/41051285476),
repeated all benchmark measurements and added source-package smoke tests on
Ubuntu, macOS and Windows. All 40 benchmark measurements passed their
correctness gates, and all three platform package-smoke jobs passed. The run
used Ubuntu x86_64, R 4.6.2, Rcpp 1.1.2, rextendr 0.5.0 and jsonlite 2.0.0
for performance measurements.

Updated end-to-end median milliseconds:

| Workload | Python | Pure R | C++/Rcpp | Rust/rextendr |
|---|---:|---:|---:|---:|
| B01, 20x20 | 0.954 | 260 | 0.883 | 0.738 |
| B02, 100x100 | 22.987 | 6473 | 21.438 | 18.250 |
| B05, 10% missing | 22.575 | 6448 | 21.125 | 18.125 |
| B06, long labels | 22.983 | 6452 | 21.500 | 18.375 |
| B07, reorder | 23.100 | 6431 | 21.375 | 18.250 |

The follow-up confirmed the ordering at that snapshot: Rust/rextendr was about
14–16% faster than C++/Rcpp and around 20% faster than Python for these
workloads. The pure-R end-to-end path remained dominated by JSON encoding:
on B02 its isolated construction median was about 34.5 ms, while JSON encoding
alone was about 6.33 seconds. These are historical prototype results, not
current workflow measurements or a general statement about R performance.

The updated measurements are still implementation-specific and limited to
scene-v1 on one Linux runner. The Windows/macOS jobs validate source builds and
semantic behavior only; they do not measure cross-platform performance or
binary distribution readiness. The result was historical evidence from the two then-existing prototypes.
C++/Rcpp was subsequently rejected and retired by PR #332; Rust is the sole
selected native implementation.

The complete raw inputs and measurements are retained in the
[workflow artifact](https://github.com/vsimko/corrplot-ng/actions/runs/41051285476).

## Cross-platform package smoke tests

At the time of the original report, the `package-smoke` job built and installed
the C++/Rcpp and Rust/rextendr prototype packages on Ubuntu, macOS and Windows,
then checked semantic parity for pure R and both native bindings. PR #332 later
retired the C++/Rcpp implementation. The current package-smoke job builds and
tests only the Rust/rextendr adapter plus the pure-R implementation; see
[benchmarks/README.md](README.md) for current workflow behavior.

## Python large-matrix stress run (2026-10-30)

A separate stress run was executed with
[workflow run 41061409661](https://github.com/vsimko/corrplot-ng/actions/runs/41061409661)
from commit `8978c159d145e44a538666ac92ad8e699fe28a3e`. The workflow completed
successfully and uploaded the machine-readable artifact
`shared-core-stress-benchmark-results` (artifact ID `12980710758`).

The run used CPython 3.12.15 on an x86_64 Linux GitHub-hosted runner with four
reported CPUs. It used seed `20261029`, three warmups and 20 measured iterations.
Semantic assertions ran before timing. The `--include-stress` option executes
the full shared-core suite (B01, B02, B03, B04, B05, B06, B07 and B08), not
only B03/B04. The measurements below are the per-call scene-construction
medians for B03/B04; SVG serialization is a separate B08 measurement.

| Workload | Matrix | Scene cells | Median scene construction | IQR | Peak traced allocation |
|---|---:|---:|---:|---:|---:|
| B03 | 500x500 | 250,000 | 2,457.39 ms | 35.76 ms | 101.5 MB |
| B04 | 1,000x1,000 | 1,000,000 | 10,236.91 ms | 129.11 ms | 413.4 MB |

Both stress workloads passed their semantic correctness gates for all 20
measured iterations. B04's median is about 4.17 times B03's while containing
four times as many cells, which is consistent with approximately linear
scaling in this tested range. This is an observation about this Python
prototype and workload only, not a complexity guarantee.

The peak allocation figures are from Python `tracemalloc`; they are an
allocation proxy, not process resident-set size (RSS), and do not represent
whole-process memory consumption. The first-call latencies were 147.72 ms for
B03 and 616.20 ms for B04; these are reported separately from the steady-state
medians. The B08 SVG-serialization-only median was 156.33 ms for its 100x100
fixture.

This stress run is Python-only. It does not compare Python with pure R,
C++/Rcpp or Rust/rextendr, and must not be used to select a production language.
The benchmark runner's extended suite also re-ran the baseline workloads and
B08. Raw inputs and all samples are available in the workflow artifact linked
above.
