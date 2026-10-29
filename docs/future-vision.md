---
type: Project Vision
title: Future vision for corrplot-ng
description: A working sketch of the goals and possible direction of corrplot-ng.
tags:
  - vision
  - roadmap
  - architecture
status: draft
---

# Future vision for corrplot-ng

## Purpose

The project aims to make correlation plots easier to use across programming languages. It also aims to keep existing R workflows working.

This document is a working sketch. It records ideas for discussion. It does not define a final design or a delivery promise.

## Main goals

1. Keep the original R package useful and stable.
2. Build a shared foundation for more than one programming language.
3. Provide a clear API with sensible defaults.
4. Keep a compatibility layer for the existing R API.
5. Help users move old code to the new API.
6. Support clear software citation for research.

## Possible design

The project may use these parts:

- **Shared core:** defines the data model, validation, visual rules, and layout.
- **R interface:** offers a new API and supports legacy calls.
- **Python interface:** makes the shared features available to Python users.
- **TypeScript interface:** supports web and JavaScript applications.
- **Migration tool:** reads R code and suggests equivalent calls in the new API.
- **Rendering backends:** turn the shared visual model into output for each target environment.

Rust is one candidate for the shared core. A small prototype must test the language, build process, rendering options, and support for each target language before the project makes this decision.

## Compatibility principles

The original R package is the reference for legacy behavior.

The project should record the current behavior in tests before it changes the implementation. These tests should cover default values, argument order, labels, ordering, annotations, return values, and graphics behavior.

The team must report any known differences. The project must not claim full compatibility until the tests support that claim.

## Possible development steps

### Step 1: Learn and record

- List the public functions and arguments in the original R package.
- Record the supported plot methods and options.
- Build example inputs and reference outputs.
- Identify behavior that needs special care.

### Step 2: Define the model

- Define a language-neutral data model.
- Group related settings into clear configuration sections.
- Separate data selection, visual encoding, layout, and rendering.
- Document defaults and validation rules.

### Step 3: Build a small prototype

- Select a small set of common plot types.
- Render the same examples through the proposed core.
- Test one R interface and one other language interface.
- Compare output quality, build cost, and maintenance effort.

### Step 4: Add compatibility

- Add an R layer that maps legacy calls to the shared model.
- Test old examples against the original package.
- Record gaps and unsupported cases.
- Keep the original package available as a reference.

### Step 5: Expand with care

- Add the remaining plot methods.
- Add Python and TypeScript support.
- Build the migration tool.
- Add new visualization types only when the core design supports them well.

## Open questions

- Which rendering model gives the best balance of consistent output and native integration?
- Which features must match the original output exactly?
- How should the core handle missing values and invalid input?
- Which features belong in the core, and which belong in a language interface?
- What is the smallest useful first release?

## Success measures

- Existing R examples pass a growing set of compatibility tests.
- The same data and configuration have clear, documented meaning across languages.
- Users can find examples and citation details for each supported interface.
- New features do not require a separate implementation for every language.
- The project remains maintainable with limited development time.
