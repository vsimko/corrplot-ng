---
type: Engineering Plan
title: Shared-core benchmark plan
description: Define a fair, reproducible comparison of candidate shared-core implementations for corrplot-ng.
tags:
  - architecture
  - benchmarks
  - performance
  - maintainability
status: proposed
---

# Shared-core benchmark plan

## Purpose

Compare candidate implementations of the same corrplot-ng computation without choosing a language based on a trivial smoke test or an isolated microbenchmark.

The benchmark is an evidence-gathering exercise. It must not make Rust, C++, or Python a production commitment. The primary question is whether a shared core materially improves end-to-end work while remaining practical to package and maintain.

## Relationship to the architecture decision

The proposed architecture defines a renderer-neutral scene as the boundary between language APIs, shared visualization rules, and renderers. The benchmark therefore measures two distinct things:

1. **Core work:** validation, normalization, ordering, visual encoding, layout, and scene construction.
2. **End-to-end work:** language adapter plus core plus SVG serialization (and, where relevant, native R graphics).

Do not compare one implementation's core-only time with another implementation's complete plotting time.

## Candidates and scope

Use the following candidates only where a working implementation exists:

- Original R `corrplot`: behavioral and end-to-end reference, not a claim that it has the same architecture.
- Existing Python prototype: reference for the language-neutral scene contract.
- C++/Rcpp spike: candidate native implementation.
- Rust/rextendr spike: candidate native implementation.

Initially implement only the same small vertical slice in each candidate. If a candidate cannot express the same semantics, record the gap instead of silently changing the workload.

## Workload suite

Use deterministic, generated inputs with fixed seeds. Matrix sizes are square and dense unless a case explicitly says otherwise.

| ID | Workload | Input | What it exercises |
|---|---|---|---|
| B01 | Small interactive plot | 20 × 20 matrix | Adapter and fixed overhead; typical small use |
| B02 | Medium plot | 100 × 100 matrix | Normal user-scale validation, layout, and scene generation |
| B03 | Large plot | 500 × 500 matrix | Scaling of per-cell processing and scene allocation |
| B04 | Stress plot | 1,000 × 1,000 matrix | Memory growth and large-input behavior; report failures as results |
| B05 | Missing values | 100 × 100 with 10% NA values | Missing-value policy and filtering |
| B06 | Label-heavy plot | 100 × 100 with long variable labels and annotations | Text/annotation preparation and scene size |
| B07 | Reordering | 100 × 100 with a fixed non-identity variable order | Permutation and consistent label/value mapping |
| B08 | Output serialization | A prebuilt scene from B02 | SVG serialization isolated from core computation |

For B01–B07, use the same input matrix, labels, configuration, and expected semantic output for every candidate. Generate fixtures once and store them in a language-neutral format (JSON for small cases; a documented compact numeric format may be used for larger matrices). Record the generator and seed.

The first implementation milestone needs only B01, B02, B05, B07, and B08. Add B03/B04 after the harness is stable so stress cases do not block initial progress.

## Correctness gate

Performance results are valid only when the candidate passes the correctness gate.

For each workload, compare semantic scene fields before comparing rendered output:

- matrix dimensions and variable order;
- cell values and missing-value representation;
- selected visual method and per-cell encodings;
- labels, annotations, legend configuration, and coordinate system;
- deterministic element ordering and stable identifiers where defined.

Use exact comparison for numeric values when the contract requires exact preservation. Where normalization or floating-point calculations are expected, define a documented tolerance and test it explicitly.

For SVG, compare normalized structure and relevant attributes. Do not compare raster screenshots across different graphics engines as if pixel identity were guaranteed. Any visual differences must be reviewed and classified.

A candidate that fails correctness is reported as **not comparable** for that workload; do not publish its faster timing as a win.

## Measurement protocol

- Run in CI on the same OS image, hardware class, and toolchain versions for a given comparison.
- Record OS, CPU, RAM, compiler, R version, package versions, build mode, and commit SHA with every result.
- Measure release/optimized builds for performance. Keep debug builds for correctness and diagnostics only.
- Warm up each workload, then collect at least 20 measured iterations for short-running cases; increase repetitions or use a benchmark harness for stable timing.
- Report median and interquartile range. Include p95 only when the sample size makes it meaningful.
- Separate cold-start/first-call time from steady-state time.
- Separate build and package installation time from runtime performance.
- Measure peak memory or a clearly documented proxy; do not infer memory from output size.
- Run candidates serially to reduce resource contention. Keep raw machine-readable results as CI artifacts.
- Do not fail CI on small timing fluctuations. Initially publish results; establish regression thresholds only after stable baselines exist.

## Metrics

Report at least:

1. Core-only elapsed time.
2. End-to-end elapsed time.
3. Cold-start/first-call time.
4. Peak memory or documented memory estimate.
5. Scene element count and serialized SVG size.
6. Build time and clean-environment installation effort.
7. Toolchain/runtime prerequisites and number of platform-specific steps.

Report both absolute values and ratios against the original R reference where the comparison is semantically fair. Do not collapse all metrics into one opaque score.

## Packaging and maintenance evidence

Performance is only one decision input. For each candidate, record:

- Can a user install a prebuilt Windows R binary without installing a compiler or language toolchain?
- Are Linux, macOS, and Windows workflows supported with the intended distribution route?
- What generated code, vendored dependencies, lockfiles, and license notices must be maintained?
- How much extra CI configuration and release automation is required?
- How easy is local debugging for a maintainer who spends only a few hours per week on the project?
- Can a second language consume the same core without duplicating visualization semantics?

Use the existing Rust/rextendr and C++/Rcpp experiments as evidence, but validate packaging with the actual intended release artifact rather than only a development installation.

## Decision criteria

Do not select a language on speed alone. A candidate can be recommended only if:

- it passes the correctness gate for the agreed vertical slice;
- it can be consumed by at least two language interfaces without duplicating core rules;
- its packaging path meets the Windows binary-install and platform support requirements;
- its maintenance and release burden is acceptable for the project's limited capacity.

A native core should demonstrate a meaningful end-to-end benefit on at least one representative workload, or provide a clearly evidenced capability that the reference implementation cannot provide. If performance is effectively equivalent, prefer the simplest and most maintainable option.

If no candidate satisfies the gates, retain the scene contract and defer the language decision.

## Deliverables

1. Versioned benchmark fixtures and deterministic generator.
2. One minimal shared-core vertical slice with identical semantics in each candidate.
3. A repeatable benchmark runner and machine-readable result artifacts.
4. Correctness report for every workload.
5. Packaging and maintenance comparison, including clean Windows binary installation.
6. A short recommendation updating ADR 0001 with evidence, limitations, and a rollback path.

## Out of scope

- Benchmarking every legacy corrplot option.
- Optimizing a candidate before the common workload and correctness contract are fixed.
- Comparing unrelated rendering engines as a proxy for core-language speed.
- Selecting a production language or migrating the legacy R package as part of this benchmark-plan document.
- Adding a permanent performance service or complex benchmark infrastructure before there is a stable baseline.

## Proposed sequence

1. Agree on the scene contract and the initial vertical slice.
2. Add deterministic fixtures for B01, B02, B05, B07, and B08.
3. Implement semantic correctness checks before timing.
4. Add the same vertical slice to each candidate.
5. Run local and CI benchmarks; publish raw results.
6. Validate clean Windows binary installation and record maintainer effort.
7. Update ADR 0001 with a recommendation or an explicit decision to defer.
