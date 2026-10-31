---
okf_version: "0.2"
type: Documentation Index
title: corrplot-ng documentation
description: Index and editing guidelines for the OKF-based project documentation.
tags:
  - documentation
  - okf
  - writing-guidelines
status: active
---

# corrplot-ng documentation

This directory contains project documentation in the Open Knowledge Format (OKF).

Use this file as the starting point. Keep all repository documentation in English.

## Project at a glance

The repository currently provides:

- A provisional R package with a legacy-compatible delegation path, normalized-matrix adapters, and an experimental scene-v1 preparation path.
- Versioned normalized-matrix v1 and scene-v1 contracts.
- A Python scene/SVG reference prototype.
- A standalone Rust CLI with its own native build and test workflow.

Rust is the accepted strategic shared-core implementation in [ADR 0001](adr/0001-shared-core-architecture.md). Merged PR #180 proves that the CLI and disposable R/rextendr adapter use the same `crates/corrplot-core` matrix-validation operation. This does not yet constitute a complete shared visualization engine or production R migration. The remaining rollout gates are CRAN guidance and a controlled maintainer-time comparison. C++/Rcpp is only a possible R-specific fallback; The cross-runtime TypeScript architecture is specified by [ADR 0002](adr/0002-cross-runtime-typescript-architecture.md); Node bindings, browser WASM, Canvas, and other renderers remain planned.

See [the current architecture](architecture.md) for implementation boundaries. See [top-level requirements](requirements.md) for product priorities; requirements are goals and are not proof that a feature is implemented.

## Documents

- [Top-level requirements](requirements.md): product and engineering requirements, priorities, and acceptance checks.
- [Current architecture](architecture.md): implemented components, the selected Rust shared-core direction, versioned contracts, and remaining rollout gates.
- [Repository layout](repository-layout.md): top-level directory ownership, contract/reference/experiment categories, and shared fixture sources of truth.
- [ADR 0001: shared-core architecture](adr/0001-shared-core-architecture.md): Rust shared-core decision and the CLI/rextendr reuse proof.
- [ADR 0002: cross-runtime TypeScript architecture](adr/0002-cross-runtime-typescript-architecture.md): Node-API and browser-WASM targets, backend selection, API boundaries, compatibility matrix, and memory/error contracts.
- [corrplot-core](../crates/corrplot-core/README.md): the dependency-free shared matrix validation library.
- [Native CLI installation](cli-installation.md): installed paths, embedded runtime assets, checksums, upgrades, and clean-environment verification.
- [Renderer capability negotiation](renderer-capabilities.md): machine-readable renderer declarations, strict/permissive behavior, host targets, and current backend limits.
- [Renderer output v1](renderer-output-v1.md): shared Rust output types, SVG/PNG/RGBA dimensions and ownership, direct Canvas host integration, and output diagnostics.
- [Legacy R adapter design](legacy-r-adapter.md): explicit legacy/scene routing, supported R input, `corrRect()` / `corrRect.hclust()` cluster-region compatibility, and graphics-device ownership.
- [Legacy R return values](legacy-r-return-values.md): invisible corrplot/corrplot.mixed results, reordered matrix names, plotted cell coordinates, and mixed diagonal behavior.
- [Diagnostic-v1 error model](diagnostics.md): stable codes, structured R/Python/CLI errors, and bug-report guidance.
- [Future vision](future-vision.md): draft goals and possible development direction; not a delivery schedule.
- [Catalog of diagrams and visualizations](diagram-catalog.md): compatibility targets, shared display options, and ideas for future visualizations.
- [Testing strategy](testing-strategy.md): current test layers, evidence limits, and CI workflows.
- [Regression test guide](../tests/README.md): pinned legacy reference, raster baselines, normalized input tests, and SVG validation.
- [Shared-core benchmark plan](shared-core-benchmark-plan.md): benchmark scope, correctness gates, measurement protocol, and decision criteria.
- [Benchmark operations](../benchmarks/README.md): how to run the shared-core, cross-language, stress, and package-smoke workflows.

The R package lives in [implementations/r](../implementations/r/README.md). It delegates legacy plotting to upstream `corrplot` by default, exposes a mixed legacy wrapper, and also provides an experimental scene path, normalized-matrix and file adapters, and statistical/palette helpers. Its scene path prepares renderer-neutral data; it is not a complete replacement renderer.

The shared Rust library lives in [crates/corrplot-core](../crates/corrplot-core/README.md), and the standalone CLI lives in [tools/corrplot-ng](../tools/corrplot-ng/README.md). The `prototypes/` tree currently groups versioned contracts, a Python reference implementation, and disposable language spikes; their different ownership and stability levels are documented in the [repository layout](repository-layout.md). Experimental Rust/rextendr and C++/Rcpp work must not be described as production R backends.

## Editing guidelines

### 1. Keep the text simple

Follow the principles of ASD-STE100 where practical.

- Use short sentences.
- Prefer common words.
- Use one main idea in each sentence.
- Use active voice when possible.
- Use direct instructions.
- Explain a technical term when readers may not know it.
- Avoid idioms, jokes, and vague phrases.
- Use the same term for the same concept throughout the documentation.

### 2. Keep all content in English

Write headings, body text, metadata, examples, comments, and file names in English. Keep code identifiers unchanged when they are part of an API or example.

### 3. Use OKF-style Markdown

Each knowledge document should use Markdown and YAML frontmatter.

- Add a clear title.
- Add a short description.
- Add useful tags.
- Add a type when the document type is clear.
- Add a status when the document is a draft or proposal.
- Keep metadata values short and factual.
- Do not add metadata fields that have no clear use.

Follow the format used by the existing documents in this directory. If the project adopts a stricter OKF schema later, update the documents to match it.

### 4. Separate facts from plans

Use clear wording to show the status of each statement.

- **Current:** verified in the repository or tests.
- **Planned:** an agreed goal that is not implemented yet.
- **Proposed:** an idea that needs review.
- **Open question:** a decision that the project has not made.

Do not describe a planned or proposed feature as implemented. Do not promise a release date unless the project has agreed to it.

### 5. Keep documents focused

- Give each file one main purpose.
- Put project-wide goals in the future vision document.
- Put plot types and visualization ideas in the diagram catalog.
- Keep general writing rules in this index.
- Link to related documents instead of copying large sections.
- Use relative links for files in this directory.

### 6. Make changes easy to review

- Use clear headings and short lists.
- Use tables only when they make comparisons easier.
- Keep each change focused on one topic.
- Check all links after moving or renaming a file.
- Update this index when you add, remove, or rename a document.
- Prefer small changes that reviewers can understand quickly.

### 7. Record decisions and uncertainty

When a design is not final, say so. Record the question and the reason it matters. When the team makes a decision, update the relevant document and remove outdated statements.

### 8. Review before commit

Before you commit a documentation change, check that:

- The text is in English.
- The sentences are short and clear.
- The frontmatter is valid YAML.
- The status of each feature is clear.
- The links work.
- The document does not conflict with the current code or tests.
- This index reflects the current document set.

## Suggested document template

Use this template for a new planning or knowledge document:

```yaml
---
type: Document Type
title: Clear document title
description: One sentence that explains the purpose.
tags:
  - relevant-topic
status: draft
---
```

After the frontmatter, add a short purpose section. Use headings that describe the content. Mark proposals and open questions clearly.

## About OKF

The project uses OKF-style Markdown and YAML frontmatter to make knowledge clear and structured. These are project editing rules. They do not claim that every field or rule is part of a formal OKF specification.
