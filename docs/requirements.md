---
okf_version: "0.2"
type: Requirements
title: Top-level requirements for corrplot-ng
description: Define product and engineering requirements with priorities and checks that match the current architecture.
tags:
  - requirements
  - compatibility
  - product
status: active
---

# Top-level requirements for corrplot-ng

## Purpose and scope

This document defines project-wide product and engineering requirements.
A requirement describes an intended outcome; it is not proof that a feature
is implemented or supported by every runtime.

The project uses versioned, language-neutral contracts to keep matrix and
scene semantics consistent across implementations. This does **not** require
all implementations to link to one compiled library. ADR 0001 explicitly
defers selecting or building a production shared-core language until a
concrete multi-consumer requirement justifies it.

See [the current architecture](architecture.md),
[ADR 0001](adr/0001-shared-core-architecture.md),
[the testing strategy](testing-strategy.md), and
[the phased roadmap](future-vision.md) for implementation status, evidence,
and dependency gates.

## Priority key

- **Must:** required to protect the project's core purpose and current
  supported workflows.
- **Should:** important, but can follow the current correctness and
  compatibility foundations.
- **Could:** optional work that requires user demand or additional evidence.

## Product requirements

| ID | Priority | Requirement | How to check |
|---|---|---|---|
| PR-01 | Must | Keep the original R package useful and stable. | Existing supported upstream R workflows continue to work; corrplot-ng does not silently change upstream behavior. |
| PR-02 | Must | Preserve high backward compatibility for the legacy R API exposed by corrplot-ng. | Pinned reference tests cover documented behavior, return values, and graphics-device effects. Known gaps are recorded; full compatibility is not claimed without evidence. |
| PR-03 | Must | Provide a shared, language-neutral design for matrix and plot semantics. | At least two current implementations consume or produce the documented normalized-matrix or scene-v1 contracts. This requirement is about contract interoperability, not reuse of one compiled implementation. |
| PR-04 | Must | Give shared contract fields the same meaning across implementations. | Shared fixtures verify labels, values, missingness, ordering, source indices, and scene semantics for each implemented consumer. Differences are explicit and tested. |
| PR-05 | Must | Preserve the agreed correlation-plot compatibility target in the legacy R path. | The R compatibility suite covers the selected legacy methods and options against a pinned upstream reference. Other runtimes may expose a documented subset; they do not inherit a claim of full feature parity. |
| PR-06 | Must | Document current behavior, defaults, limitations, and examples. | Each current public interface and versioned contract has usage guidance, examples, and explicit support status. |
| PR-07 | Must | Keep the project maintainable with limited development capacity. | Prefer small changes, automated checks, focused dependencies, and a release process proportionate to demonstrated user value. |
| PR-08 | Must | Do not claim compatibility or feature support without evidence. | Public claims identify the implementation, tested scope, reference/contract version, and known limitations. |
| PR-09 | Could | Provide a migration aid for common legacy R calls. | Revisit when user demand is demonstrated; a prototype must produce reviewable suggestions and warn about unsupported patterns. |
| PR-10 | Should | Support clear software citation for research use. | Citation metadata and instructions are available for each published package or release. |
| PR-11 | Should | Allow additional visualization types without redefining shared matrix and scene semantics in every implementation. | A new visualization reuses the versioned contracts and has renderer capability and conformance tests. |
| PR-12 | Could | Support interactive plots in web applications. | A focused browser prototype demonstrates useful inspection or selection without changing source data or publication configuration implicitly. |

## Engineering requirements

| ID | Priority | Requirement | How to check |
|---|---|---|---|
| ER-01 | Must | Separate language-specific APIs and input adapters from language-neutral matrix and scene contracts. | Normalized-matrix v1 and scene-v1 schemas/fixtures are independent of R, Python, Rust, TypeScript, and renderer-specific object types. Contract producers and consumers pass the applicable conformance tests. |
| ER-02 | Must | Validate matrix inputs and define missing-value behavior. | Tests cover shape, labels, duplicates, numeric values, non-finite values, and missing-versus-zero semantics at the normalized input boundary. |
| ER-03 | Must | Test legacy behavior before changing compatibility code. | Regression cases use a named, pinned upstream corrplot reference and record their scope and limitations. |
| ER-04 | Must | Keep output-backend details out of shared visualization semantics where practical. | Scene data remains renderer-neutral; renderer capabilities and unsupported-feature behavior are declared separately. |
| ER-05 | Must | Run automated correctness checks for each current supported interface. | Relevant fixture, R package, and native CLI workflows run in CI on their declared path filters and platforms. |
| ER-06 | Should | Keep installation and packaging practical for each interface that is declared supported. | A clean environment can install the documented artifact and run its example without an undeclared developer toolchain. Supported platforms and runtime prerequisites are explicit. |
| ER-07 | Should | Keep language-neutral contracts versioned, serializable, and inspectable. | Schemas, examples, and fixtures allow inputs and outputs to be examined in tests and debugging. Contract version changes have compatibility rules. |
| ER-08 | Should | Keep the architecture small and avoid unnecessary services or runtime dependencies. | Each major component and dependency has a documented requirement that justifies its maintenance and release cost. |

## What counts as language or runtime support

Do not use “supports a language” to mean only that a prototype can parse
a fixture. A supported public interface must state which of the following
it provides:

1. **Public API:** documented functions or commands, stable input/output
   behavior, defaults, and errors.
2. **Contract role:** which versioned inputs it produces or consumes, with
   schema and semantic conformance tests.
3. **Feature scope:** supported scene semantics and output formats, including
   explicit unsupported-feature behavior.
4. **Packaging:** installation method, runtime prerequisites, version policy,
   and supported OS/architecture matrix.
5. **Verification:** automated tests for the public behavior and declared
   platform matrix.
6. **Maintenance:** an owner and a release/update process appropriate to the
   support promise.

Contract interoperability is a useful capability, but it is not by itself
a complete language package or proof of shared compiled implementation.

## Current implementation status

| Runtime or component | Current status | What the status means |
|---|---|---|
| R package | Provisional implementation | Default plotting delegates to upstream corrplot. It also provides normalized matrix/file adapters, statistical and palette helpers, and an experimental scene-v1 preparation path. It is not a complete replacement renderer. |
| Python | Reference/prototype | The shared scene/SVG prototype exercises scene construction and deterministic SVG output. It is not a production package or selected shared core. |
| Rust | Standalone CLI | Rust stable, edition 2021, is selected for the native CLI. The CLI is a separate executable, not a reusable production library shared with R or other consumers. |
| Rust/rextendr and C++/Rcpp | Evaluation spikes | These experiments inform architecture and packaging decisions. They are not production R backends. |
| TypeScript / Node.js | Planned | No supported public TypeScript API or Node native binding is currently delivered. |
| Browser WebAssembly / Canvas | Planned | No production browser WASM or direct Canvas renderer is currently delivered. |

The current support and test details are maintained in
[the architecture document](architecture.md),
[the R implementation guide](../implementations/r/README.md),
[the native CLI guide](../tools/corrplot-ng/README.md), and
[the testing strategy](testing-strategy.md).

## Architecture constraints and decision boundaries

- Normalized-matrix v1 and scene-v1 define language-neutral data contracts.
- A contract consumer may be implemented in a different language without
  sharing a compiled library with the producer.
- Rust is selected for the standalone CLI only.
- The production shared-core language remains deferred by ADR 0001.
- No requirement in this document authorizes migration of the R package to
  Rust/rextendr or C++/Rcpp.
- TypeScript, Node native bindings, browser WASM, Canvas, and cross-backend
  parity remain future work with explicit dependencies.
- Linux distribution repository adoption and DEB/RPM automation remain
  deferred under [issue #99](https://github.com/vsimko/corrplot-ng/issues/99).

A production shared core may be reconsidered only when at least two concrete
consumers need the same compiled semantics and a working vertical slice
demonstrates reuse, packaging feasibility, and acceptable maintenance cost.
Any change requires an explicit ADR update. Contract interoperability alone
does not satisfy this gate.

## Open questions

- Which additional legacy R behaviors are necessary for the next compatibility milestone?
- Which real consumers, if any, will require a shared compiled library rather than the versioned interchange contracts?
- Which runtime and platform combinations justify a formal support commitment?
- Which output formats and renderer capabilities are required by actual users?
- Is there enough user demand to prioritize migration tooling or interactive exploration?
- What packaging and maintenance burden is acceptable for the project's available capacity?

## Status vocabulary

Use these terms in requirements, documentation, and release notes:

- **Current:** implemented and supported by the stated tests or release process.
- **Partial:** implemented for a limited scope; limitations are documented.
- **Planned:** accepted future work that is not yet implemented.
- **Deferred:** intentionally postponed until a stated condition is met.
- **Proposed:** an idea that still needs review or evidence.

A requirement or issue marked Must does not override an explicit architecture
decision or dependency gate. If a requirement conflicts with ADR 0001, update
the requirement or formally revisit the ADR; do not infer a language decision.
