# Benchmarks

The repository has two benchmark suites:

- **Shared-core benchmark:** measures the Python scene prototype on B01, B02, B05, B07 and B08.
- **Cross-language benchmark:** compares Python, Pure R, C++/Rcpp and Rust/rextendr on the common scene-v1 workloads. The regular suite covers B01, B02, B05, B06 and B07. The optional stress suite covers B03 (500x500) and B04 (1000x1000); Pure R is excluded from these two large workloads.

Performance benchmarks are deliberately excluded from pull-request workflows. They run only when a maintainer manually starts **Performance benchmarks** from the GitHub Actions tab. This prevents routine PRs from consuming long-running benchmark resources.

## Run locally

From the repository root:

```sh
python -m unittest discover -s benchmarks -p 'test_timing_method.py' -v
python benchmarks/run_shared_core.py --iterations 20 --output benchmark-results/results.json
# Optional large-matrix stress run:
python benchmarks/run_shared_core.py --iterations 20 --include-stress --output benchmark-results/stress-results.json
```

The shared-core runner uses deterministic generated inputs (seed 20261029), writes the workload inputs and manifest into the result JSON, and emits one result per workload. The artifact therefore preserves the exact input data used by a run.

## Run in GitHub Actions

Open **Actions → Performance benchmarks → Run workflow**.

- Leave **Include stress** disabled for the regular comparison.
- Enable **Include stress** to add B03/B04 for the shared-core and cross-language suites.
- Enable **Package smoke** only when native package build and semantic parity checks are needed on Ubuntu, macOS and Windows.

The workflow uploads machine-readable JSON fixtures and timing results as artifacts. Results are informational; timing fluctuations do not fail CI. Benchmark comparisons should be interpreted only alongside their semantic correctness checks and recorded runner/toolchain metadata.

## Cross-language methodology

The cross-language workflow exports shared fixtures before timing. Python, Pure R, C++/Rcpp and Rust/rextendr are compared on equivalent inputs. Each implementation must pass semantic comparison against the expected scene before its timing is considered. Pure R remains in the regular comparison but is intentionally omitted from B03/B04 stress measurements because its JSON serialization path is not practical at those sizes.

Regular measurements use warmups and repeated samples with calibrated batches for short workloads. Stress measurements use one warmup, five measured samples and batch size one to avoid repeating multi-second workloads unnecessarily. The workflow records raw samples and artifacts; performance results are specific to the CI runner and are not, by themselves, a production-language decision.

## Workloads

- B01: 20x20 dense symmetric matrix.
- B02: 100x100 dense symmetric matrix.
- B03: 500x500 stress matrix.
- B04: 1,000x1,000 stress matrix.
- B05: 100x100 matrix with a deterministic 10% missing-value pattern.
- B06: 100x100 matrix with long variable labels and an explanatory annotation.
- B07: 100x100 matrix with a fixed non-identity permutation of labels and corresponding rows/columns.
- B08: SVG serialization of the prebuilt B02 scene (shared-core suite).

## Correctness gate and metrics

Every workload runs semantic checks before timing. Checks cover dimensions, cell ordering, values and null preservation, label order, generated-matrix symmetry, and deterministic scene/SVG serialization. A failed gate marks that workload as not comparable and prevents a performance claim.

The shared-core runner reports first-call latency, steady-state median and IQR, SVG byte count, scene element count, and peak traced Python allocation as a documented memory proxy (not process RSS). Cross-language results record raw timing samples and environment metadata. Do not compare core-only and end-to-end measurements as if they were the same operation, and do not infer memory usage from output size.
