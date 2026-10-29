---
okf_version: "0.2"
type: Engineering Strategy
title: Testing strategy
description: Define the test layers and evidence needed to protect compatibility and shared behavior.
tags:
  - testing
  - compatibility
  - quality
status: proposed
---

# Testing strategy

## Purpose

Build a test system before the project adds a large amount of implementation code. Protect the original R package and check that future language interfaces use the same rules.

The repository has a provisional R package scaffold under `implementations/r/`. Its `corrplot_ng()` adapter delegates to the original `corrplot` package. The Python fixture checks validate test data only; neither proves that a new implementation is compatible.

## Test layers

### 1. Legacy R compatibility

Use the original `corrplot` package as the reference for behavior. Until a new engine exists, the provisional R adapter calls that package directly. Keep this status explicit in test reports.

The initial visual regression harness is described by `tests/legacy-visual/cases.json`. Its representative cases are based on examples from the original repository:

- `tests/testthat/test-corrplot.R`
- `vignettes/example-corrplot.R`
- `vignettes/example-corrplot.mixed.R`
- `vignettes/corrplot-intro.Rmd`

Each case is rendered twice in the same CI environment: once through `corrplot::corrplot()` and once through `corrplotng::corrplot_ng()`. The resulting PNG pixel arrays are compared exactly. CI installs a pinned commit of the original package so that changes upstream do not silently redefine the reference.

This comparison is intentionally dynamic rather than a collection of checked-in screenshots: the original implementation is the reference renderer, and the adapter is the candidate. When a new renderer replaces the adapter, differences should fail the test and be reviewed. Any accepted visual differences must be documented rather than hidden by automatically regenerating snapshots. Exact pixel comparison is appropriate for the current wrapper because both paths use the same renderer and device; a future cross-renderer comparison may need fixed environments and reviewed tolerances.

The current battery is representative, not exhaustive. Expand it from the original examples over time, especially for public arguments and interactions between options.

Record tests for:

- Public functions and arguments.
- Default values and argument combinations.
- Return values and side effects.
- Errors and warnings.
- Plot methods: `circle`, `square`, `ellipse`, `number`, `shade`, `color`, and `pie`.
- Important labels, legends, ordering, and graphical parameters.
- User drawing after plotting, including common native R graphics operations.

Each compatibility case must record the reference package version or commit, input, call, expected result, and verification method. Do not mark a case as verified until an automated test passes.

### 2. Shared data and mathematical rules

Test input validation and documented rules for:

- Matrix dimensions and labels.
- Symmetry and diagonal values where required.
- Missing and non-finite values.
- Values outside the supported range.
- Variable ordering and subsetting.
- Empty, one-variable, and large inputs.

Do not assume that every input must be accepted. Define whether each case is accepted, normalized, or rejected.

### 3. Shared plot model and language interfaces

When the shared model exists, test it independently from rendering. Use the same versioned fixture cases in each language.

Compare semantic fields such as variable order, values, visual method, labels, legend settings, and missing-value handling. Define documented exceptions instead of silently allowing language-specific differences.

### 4. Rendering regression

Test plot structure and geometry before relying on image comparison. Add visual snapshots only for stable, meaningful outputs.

Avoid comparing raw image bytes across different operating systems or graphics devices unless the environment is fixed. Prefer scene or normalized vector-output comparisons where available, and use reviewed image tolerances when renderers differ.

### 5. Integration and packaging

Check package installation, supported runtime versions, language bindings, and supported rendering backends. Add operating systems and version combinations based on the project's declared support policy.

### 6. Property-based tests

Add generated inputs after the main rules are documented. Check invariants such as determinism, symmetric handling, valid ranges, and consistent reordering. Use fixed seeds for reproducible failures.

## Test data

Keep shared cases in `tests/fixtures/`. Each case should have:

- A stable, unique ID.
- A small input.
- Explicit expected properties or an expected result.
- A short description of the behavior under test.
- A source reference when it captures legacy behavior.

Use JSON for language-neutral numeric fixtures when practical. Represent missing values as JSON `null`; do not use non-standard JSON values such as `NaN` or `Infinity`.

The initial fixture file is a seed, not a complete compatibility corpus. Expand it with observed behavior from the original R package.

## Continuous integration

Use three levels as the implementation grows:

1. **Pull request:** fixture integrity, fast unit tests, linting, and core compatibility cases.
2. **Main branch:** the full supported test suite and package installation checks.
3. **Scheduled run:** broader runtime matrices, generated inputs, visual regression, and performance checks.

A required check must fail when its tests fail. Do not report a compatibility status from a workflow that only validates fixture structure.

## Compatibility status

Track each important behavior in a compatibility ledger:

- `verified`: an automated test passes against a named reference.
- `partial`: only part of the behavior is covered.
- `changed`: behavior changed intentionally and the change is documented.
- `untested`: no adequate automated evidence exists.

The ledger should link to the test case and reference version. Test coverage percentage alone is not proof of compatibility.

## Initial implementation plan

1. Keep the fixture-integrity test and CI workflow working.
2. Run the legacy visual regression battery against a pinned original package.
3. Expand the battery with representative examples from the original tests and vignettes.
4. Capture additional semantic and behavioral regression tests.
5. Add shared-model tests when that model has a defined schema.
6. Add Python and TypeScript runners only when those interfaces exist.

## Acceptance criteria

- Test fixtures have stable IDs and documented expected properties.
- CI validates fixture integrity on each pull request.
- Visual regression cases identify their original source and pinned reference version.
- Compatibility tests identify the reference version.
- Claims about supported behavior link to passing tests.
- Failures are reproducible and explain which rule was broken.
