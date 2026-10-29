---
type: Requirements
title: Top-level requirements for corrplot-ng
description: The main product and engineering requirements that guide architecture and planning.
tags:
  - requirements
  - compatibility
  - product
status: draft
---

# Top-level requirements for corrplot-ng

## Purpose

This document lists the requirements that guide product decisions. These requirements apply across the project unless a requirement states otherwise.

The list is a starting point. It must be refined with tests and user feedback. A requirement is not proof that the feature is already implemented.

## Priority key

- **Must:** required for the project to succeed.
- **Should:** important, but may follow the first useful release.
- **Could:** useful when time and evidence support it.

## Product requirements

| ID | Priority | Requirement | How to check |
|---|---|---|---|
| PR-01 | Must | Keep the original R package useful and stable. | Existing supported R workflows continue to work in the original package. |
| PR-02 | Must | Provide high backward compatibility for the legacy R API in the new project. | A compatibility test suite covers documented legacy behavior and reports known gaps. |
| PR-03 | Must | Support more than one programming language through a shared design. | At least two language interfaces use the same documented plot model. |
| PR-04 | Must | Give shared options the same meaning across supported languages. | Cross-language tests use the same input and configuration. |
| PR-05 | Must | Make the common correlation-plot methods available. | Tests and examples cover the agreed compatibility target list. |
| PR-06 | Must | Document behavior, defaults, limitations, and examples. | Each supported public feature has documentation and an example. |
| PR-07 | Must | Keep the project maintainable with only a few hours of development time per week, with AI assistance. | Prefer small changes, automation, focused scope, and low maintenance cost. |
| PR-08 | Must | Do not claim compatibility or feature support without evidence. | Public claims match tests and documented implementation status. |
| PR-09 | Should | Provide a migration aid for common legacy R calls. | The tool produces reviewable suggestions and warns about unsupported patterns. |
| PR-10 | Should | Support clear citation for use in research. | Citation metadata and instructions are available for releases. |
| PR-11 | Should | Allow new visualization types without separate implementations of all shared rules in every language. | A prototype demonstrates reuse of the core model and visual rules. |
| PR-12 | Could | Support interactive plots in web applications. | A prototype demonstrates inspection or selection in a browser. |

## Engineering requirements

| ID | Priority | Requirement | How to check |
|---|---|---|---|
| ER-01 | Must | Separate the shared plot model from language-specific APIs. | Architecture and tests show a clear boundary. |
| ER-02 | Must | Validate inputs and define missing-value behavior. | Tests cover valid input, invalid input, and missing values. |
| ER-03 | Must | Test legacy behavior before changing compatibility code. | Reference cases and regression tests exist before changes. |
| ER-04 | Must | Keep platform-specific rendering details out of shared visual rules where practical. | Shared rules have one documented definition and backend tests. |
| ER-05 | Must | Use automated tests for core behavior and supported interfaces. | Tests run in the project’s continuous integration workflow. |
| ER-06 | Should | Keep builds and installation steps practical for each supported language. | A clean environment can install and run documented examples. |
| ER-07 | Should | Use a language-neutral model that can be serialized or inspected during debugging. | Core inputs and outputs can be examined in tests or diagnostics. |
| ER-08 | Should | Keep the initial architecture small and avoid unnecessary services. | Each major component has a clear requirement that justifies it. |

## Constraints and principles

- Backward compatibility has high priority.
- The original R package remains the reference for legacy behavior.
- The shared core language is not yet decided. Rust is a candidate.
- The project has limited development time. Prefer a small, testable first release.
- Keep repository documentation in English.
- Mark each feature as current, planned, proposed, or an open question.

## Open questions

- Which legacy functions and edge cases are required for the first compatibility milestone?
- Which two language interfaces should be used for the first prototype?
- Which rendering outputs are required for the first useful release?
- What is the minimum test suite that gives confidence in compatibility?
- Which requirements should be moved from Should or Could to Must after user feedback?
