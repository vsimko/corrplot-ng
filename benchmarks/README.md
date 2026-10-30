# Benchmarks

The repository has two benchmark suites:

- **Shared-core benchmark:** measures the Python scene prototype on B01, B02, B05, B07 and B08.
- **Cross-language benchmark:** compares Python, Pure R, C++/Rcpp and Rust/rextendr on the common scene-v1 workloads. The regular suite covers B01, B02, B05, B06 and B07. The optional stress suite covers B03 (500x500) and B04 (1000x1000); Pure R is excluded from these two large workloads.

Performance benchmarks are deliberately excluded from pull-request workflows. They run only when a maintainer manually starts **Performance benchmarks** from the GitHub Actions tab.

## Run locally

From the repository root:

```sh
python -m unittest discover -s benchmarks -p 'test_timing_method.py' -v
python benchmarks/run_shared_core.py --iterations 20 --output benchmark-results/results.json
python benchmarks/run_scalability.py --iterations 5 --output benchmark-results/scalability.json
# Explicit opt-in for 500x500 and 1000x1000 stress workloads:
python benchmarks/run_scalability.py --iterations 3 --include-stress --output benchmark-results/scalability-stress.json
```

The shared-core runner stores its deterministic inputs and manifest in the result JSON. The scalability runner records workload dimensions and SHA-256 fingerprints, avoiding duplication of large input matrices in result artifacts.

## Renderer scalability profile

`run_scalability.py` measures four distinct stages:

- **Configuration loading:** JSON decoding of a prepared configuration.
- **Validation and scene construction:** the current `build_scene` operation, which performs input validation and constructs the renderer-neutral scene together.
- **SVG serialization:** serialization of a prebuilt scene, excluding scene construction.
- **End-to-end:** configuration decoding, scene construction and SVG serialization.

The regular profile includes B01 (20x20), B02 (100x100), B05 (100x100 with symmetric 10% missingness), and B06 (100x100 with long labels and an annotation). B03 (500x500) and B04 (1000x1000) are available only with `--include-stress`.

Each result includes raw timing samples, first-call latency, median, IQR, peak traced Python allocation, dimensions, cell count, output size, input and scene fingerprints, and environment metadata. The memory value is a `tracemalloc` allocation proxy, not process RSS. It must not be described as total process memory.

The current scene API combines validation and construction; the benchmark therefore reports them as one stage rather than claiming to isolate validation cost. Splitting that API should be considered only if profiling demonstrates a need and compatibility can be preserved.

## Interpreting results and budgets

Use the regular B02 workload as the reference profile. Treat the results as evidence for setting and revising target budgets on a named reference runner, not as universal guarantees. Compare runs only when Python version, operating system, architecture and runner class are recorded and reasonably equivalent.

Stress results are for observing scaling trends and identifying memory or runtime cliffs. They are informational and do not fail ordinary CI. Do not add brittle wall-clock assertions to unit tests or use output byte size as a proxy for memory use.

## Existing manual benchmark workflow

Open **Actions → Performance benchmarks → Run workflow**.

- Leave **Include stress** disabled for the regular comparison.
- Enable **Include stress** to add B03/B04 to the existing shared-core and cross-language suites.
- Enable **Package smoke** only when native package build and semantic parity checks are needed on Ubuntu, macOS and Windows.

The existing workflow uploads machine-readable JSON fixtures and timing results as artifacts. Benchmark execution remains manual, not part of routine pull-request CI.

## Cross-language methodology

The cross-language workflow exports shared fixtures before timing. Python, Pure R, C++/Rcpp and Rust/rextendr are compared on equivalent inputs. Each implementation must pass semantic comparison against the expected scene before its timing is considered. Pure R remains in the regular comparison but is intentionally omitted from B03/B04 stress measurements because its JSON serialization path is not practical at those sizes.

Regular measurements use warmups and repeated samples with calibrated batches for short workloads. Stress measurements use one warmup, five measured samples and batch size one. Results are specific to the CI runner and are not, by themselves, a production-language decision.

## Workloads

- B01: 20x20 dense symmetric matrix.
- B02: 100x100 dense symmetric matrix.
- B03: 500x500 stress matrix.
- B04: 1,000x1,000 stress matrix.
- B05: 100x100 matrix with a deterministic 10% missing-value pattern.
- B06: 100x100 matrix with long variable labels and an explanatory annotation.
- B07: 100x100 matrix with a fixed non-identity permutation of labels and corresponding rows/columns.
- B08: SVG serialization of the prebuilt B02 scene (shared-core suite).

## Existing shared-core correctness gate and metrics

Every workload runs semantic checks before timing. Checks cover dimensions, cell ordering, values and null preservation, label order, generated-matrix symmetry, and deterministic scene/SVG serialization. A failed gate marks that workload as not comparable and prevents a performance claim.

The shared-core runner reports first-call latency, steady-state median and IQR, SVG byte count, scene element count, and peak traced Python allocation as a documented memory proxy (not process RSS). Cross-language results record raw timing samples and environment metadata. Do not compare core-only and end-to-end measurements as if they were the same operation, and do not infer memory usage from output size.
