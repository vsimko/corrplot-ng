# ADR 0001: Shared-core architecture for corrplot-ng

- **Status:** Proposed; implementation-language decision deferred
- **Date:** 2026-10-29
- **Decision owners:** corrplot-ng maintainers
- **Related:** [Issue #12](https://github.com/vsimko/corrplot-ng/issues/12), [Rust/rextendr spike #13](https://github.com/vsimko/corrplot-ng/pull/13), [C++/Rcpp spike #14](https://github.com/vsimko/corrplot-ng/pull/14)

## Context

corrplot-ng should preserve the established R `corrplot` interface while enabling a modern API and wrappers for R, Python, and TypeScript. The project has only a few hours of maintainer time per week, so the architecture must minimize duplicated behavior and ongoing release/CI work.

The repository already has a Python scene prototype and two minimal R integration experiments: Rust via `rextendr` and C++ via `Rcpp`. Both experiments have passed their current cross-platform CI checks on Windows, macOS, and Linux. These results establish that the basic integrations can work; they do **not** yet establish which option is easier to distribute or maintain. In particular, installation of a built Windows binary without a compiler toolchain and the relevant CRAN distribution requirements remain to be verified.

## Decision

**Adopt a renderer-neutral, language-neutral scene model as the architectural boundary. Do not select a production implementation language yet.**

The scene model should describe the intended visualization, not how a particular language or graphics library draws it. Keep the existing Python prototype as a reference implementation while the contract is clarified. Treat Rust/rextendr and C++/Rcpp as disposable evidence-gathering spikes, not production foundations.

The intended logical layers are:

1. **Compatibility and language APIs** — the legacy R API, the new R API, and future Python/TypeScript APIs. The legacy R layer translates existing arguments into the shared model and preserves historical defaults and behavior.
2. **Visualization model** — validated inputs, normalized configuration, matrix layout, visual encodings, labels, annotations, and a renderer-neutral scene representation.
3. **Renderers/exporters** — convert the scene to supported output formats or native graphics systems. Rendering-specific details must not leak into the public configuration model.
4. **Compatibility evidence** — fixtures and regression tests based on the original R package, covering both semantic outputs and rendered results where stable comparison is practical.

These are logical boundaries, not an instruction to create separate repositories or publish separate packages immediately. Keep the implementation in this repository until there is demonstrated value in splitting it.

## Why defer the implementation-language choice?

The current spikes demonstrate basic R integration, but the most consequential constraints are not settled:

- Can users install Windows binaries without installing a compiler or language toolchain?
- What are the current CRAN requirements and practical maintenance costs for each route?
- How much release automation, generated code, and platform-specific debugging does each approach require?
- Can the chosen core be reused by Python and TypeScript without maintaining separate implementations of visualization semantics?

Choosing based only on a successful smoke test would answer the easiest question rather than the decisive ones.

## Options considered

### Rust + rextendr

**Potential advantages:** a strong type system, explicit data modeling, and a plausible route to a shared native core.

**Risks to validate:** Rust toolchain and build integration, generated bindings, Windows binary distribution, CRAN policy and release workflow, and reuse from Python/TypeScript.

### C++ + Rcpp

**Potential advantages:** mature R ecosystem integration and familiar R package workflows.

**Risks to validate:** compiler/toolchain setup, generated registration and binding maintenance, Windows binary distribution, CRAN policy and release workflow, and reuse from Python/TypeScript.

### Keep Python as the production core

**Potential advantages:** builds on the existing prototype and avoids choosing a native language prematurely.

**Risks:** R and TypeScript integration and distribution may become awkward; Python could become a required runtime even for users who only need the R package. This remains a useful reference/prototype, not a production commitment.

### Separate implementations per language

Rejected for now. It would duplicate visualization semantics, increase regression risk, and exceed the likely maintenance capacity.

## Consequences

- The scene contract becomes the first stable technical artifact; it is not yet a promised public API.
- No production core should be implemented in Rust or C++ until the distribution and maintenance checks below are complete.
- Existing R behavior remains the compatibility reference. New API design must not force a breaking change in the legacy API.
- A single repository and shared fixtures are preferred initially.
- CI should test the contract and compatibility fixtures independently from each language binding.

## Decision gate: evidence required before choosing Rust or C++

Close the following items before making a production-language decision:

- [ ] Verify installation of a built Windows R binary on a clean environment without Rust, Rtools, or another compiler toolchain.
- [ ] Check current CRAN requirements and document the release implications for both approaches.
- [ ] Record local setup, CI complexity, generated files, debugging steps, and maintainer effort for each spike.
- [ ] Demonstrate that the scene contract can be consumed independently of the R binding.
- [ ] Compare both approaches against the same minimal end-to-end example and compatibility expectations.
- [ ] Write a short recommendation with evidence, trade-offs, and a rollback path.

If neither native approach meets the distribution and maintenance constraints, defer the choice rather than expanding either spike.

## Next implementation step

Define a small, versioned scene-model contract using the existing Python prototype and one representative correlation-matrix example. It should cover the minimum information needed to express matrix dimensions/order, cell values, visual encoding, labels, and annotations. Add fixtures and tests before implementing additional rendering features.

Then run the same example through the R-facing prototype and one non-R consumer. Keep this vertical slice deliberately small; do not implement all corrplot features or migrate the legacy package as part of the experiment.

## Revisit when

Revisit this ADR after the Windows binary-install test, CRAN review, and matched end-to-end comparison are complete, or if a concrete requirement makes one implementation path clearly unsuitable.
