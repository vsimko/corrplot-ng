# Initial shared-core benchmark suite

This first executable milestone covers B01, B02, B05, B07 and B08 from
`docs/shared-core-benchmark-plan.md`. It benchmarks the existing Python scene
prototype only; it does not imply that Python is the production choice or
claim cross-language performance parity.

## Run

From the repository root:

```sh
python -m unittest discover -s tests -v
python benchmarks/run_shared_core.py --iterations 20 --output benchmark-results/results.json
# Optional stress run (large memory/runtime footprint):
python benchmarks/run_shared_core.py --iterations 20 --include-stress --output benchmark-results/stress-results.json
```

The default runner uses deterministic generated inputs (seed 20261029), writes the
complete workload inputs and their manifest into the result JSON, and emits
one result per workload. This keeps the committed source compact while making
each run independently reproducible and preserving the exact input data in
the artifact.

## Workloads

- B01: 20x20 dense symmetric matrix.
- B02: 100x100 dense symmetric matrix.
- B05: 100x100 matrix with a deterministic 10% missing-value pattern.
- B06: 100x100 matrix with long variable labels and an explanatory annotation.
- B07: 100x100 matrix with a fixed non-identity permutation of labels and
  corresponding rows/columns.
- B08: SVG serialization of the prebuilt B02 scene (serialization only).
- B03/B04: optional 500x500 and 1,000x1,000 stress workloads. Run with `--include-stress`; CI exposes them through the `workflow_dispatch` input `include_stress`. They are kept out of every pull-request run because of their memory and runtime cost.

## Correctness gate and measurements

Every workload runs semantic checks before timing. Checks cover dimensions,
row-major cell ordering, values and null preservation, label order, symmetry
for generated correlation inputs, and deterministic scene/SVG serialization.
A failed gate marks that workload `not_comparable` and prevents timing.

The runner reports first-call latency, steady-state median and IQR from at
least 20 iterations, SVG byte count, scene element count, and peak traced
Python allocation as a documented memory proxy (not process RSS). It records
Python/platform/CPU metadata and raw per-iteration samples. CI uploads the
machine-readable JSON artifact; timing is informational and does not fail CI.

The current runner measures Python core-only scene construction for B01,
B02, B05 and B07, and isolated SVG serialization for B08. It does not yet
compare against R, C++/Rcpp or Rust/rextendr; those candidates must first
implement the same semantics and pass the same correctness gate.
