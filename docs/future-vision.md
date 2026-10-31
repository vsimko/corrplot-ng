---
okf_version: "0.2"
type: Project Roadmap
title: Future vision and phased roadmap for corrplot-ng
description: Record the implemented baseline, evidence-based next phases, and gates for future architecture work.
tags:
  - vision
  - roadmap
  - architecture
status: active
---

# Future vision and phased roadmap for corrplot-ng

## Purpose and planning rules

This roadmap describes a possible sequence of work based on the repository's
current implementation. It is not a release schedule or a promise to deliver
every proposed capability.

The project should preserve the original R workflow, keep visualization
semantics in versioned language-neutral contracts, and add interfaces only
when their maintenance and user value are clear. The roadmap therefore
separates completed foundations, active quality work, renderer expansion,
and conditional cross-runtime work.

No dates or release milestones are assigned. A phase should advance only
when its dependencies and measurable outcomes are satisfied.

## Current baseline

The repository has moved beyond the original prototype plan. The following
foundations are implemented:

- **Normalized-matrix v1:** a versioned input contract for already-computed
  square matrices, with explicit labels, missing-value representation,
  provenance, and zero-based source-index mapping. R matrix and CSV/TSV/JSON
  adapters produce this representation. Cross-adapter parity is covered by
  the merged input work, including [PR #136](https://github.com/vsimko/corrplot-ng/pull/136).
- **Scene-v1:** a renderer-neutral, versioned representation that separates
  plot semantics and geometry from language-specific APIs and output
  backends. The Python and R consumers exercise this contract.
- **Python SVG prototype:** a deterministic scene-to-SVG reference path with
  layout, annotations, accessible metadata, structural validation, and
  seven prototype glyph methods. It remains a reference implementation,
  not the selected production shared core.
- **R package:** a default legacy-compatible delegation path plus an
  experimental scene-v1 preparation path, normalized matrix/file adapters,
  pairwise correlation testing, and palette/color-legend helpers. The scene
  path prepares data; it is not a complete replacement renderer.
- **Standalone Rust CLI:** a compiled executable with CSV/TSV and scene-v1
  inputs, publication-config v1 support, validation, and SVG output. It
  does not require Python or R at runtime and has native build/test coverage
  on Linux, macOS, and Windows.
- **Native Node.js binding:** an experimental N-API addon in `bindings/node/`
  exposes shared matrix validation and typed-scene SVG/RGBA rendering. Its
  clean npm tarball install is tested without Cargo at runtime on Node.js 22,
  24, and 26 across Linux, macOS, and Windows. Prebuilt package publishing,
  scene construction, and the unified TypeScript facade remain open.
- **Correctness and compatibility checks:** shared fixture validation,
  normalized-input contract tests, R compatibility tests against a pinned
  upstream reference, scene/SVG tests, native CLI tests, and SVG
  interoperability checks. Performance benchmarks are scheduled or manually dispatched and remain
  separate from routine pull-request CI.

The implementation evidence is reflected in the merged work on scene-v1,
publication configuration and SVG export, R compatibility, normalized
inputs, and the standalone CLI, including PRs #9, #34, #53–#58, #61,
#73–#76, #86–#94, #100–#109, #117–#129, and #133–#138. The current
component boundaries are described in [the architecture document](architecture.md)
and [ADR 0001](adr/0001-shared-core-architecture.md).

These capabilities are not equivalent to full legacy API compatibility,
a complete shared visualization engine, or identical rendering across every
backend. PR #180 proves one shared compiled operation (matrix validation)
for the CLI and disposable R/rextendr adapter. Remaining core operations and
runtime bindings require separate evidence.

## Roadmap phases

### Phase 1 — Complete semantic and compatibility foundations

**Status:** Active quality work.

Keep the current R compatibility path reliable and make shared scene
semantics explicit before adding more renderers or runtimes.

Relevant work:

- [#69 — Figure titles, subtitles, captions, and footnotes](https://github.com/vsimko/corrplot-ng/issues/69).
- [#71 — Renderer capability negotiation](https://github.com/vsimko/corrplot-ng/issues/71).
- [#72 — Structured diagnostics and stable errors](https://github.com/vsimko/corrplot-ng/issues/72).
- [#80 — Cluster rectangle semantics and legacy `corrRect()` compatibility](https://github.com/vsimko/corrplot-ng/issues/80).
- [#82 — Legacy return values and graphics-device composition](https://github.com/vsimko/corrplot-ng/issues/82).

**Dependency gate:** scene and normalized-matrix contract changes must remain
versioned and must not put renderer-specific behavior into the shared
contracts. R-specific return values and graphics-device behavior remain in
the R adapter.

**Measurable outcomes:**

- Required scene semantics are represented independently of SVG markup.
- Unsupported renderer features fail explicitly or are reported as allowed
  omissions; they are never silently dropped.
- Equivalent validation failures expose stable diagnostic codes and useful
  source locations where available.
- Legacy behavior changes have pinned regression cases and documented
  compatibility status.
- The R package, contract, and foundation workflows pass without weakening
  existing semantic assertions.

### Phase 2 — Finish native CLI delivery and renderer contracts

**Status:** CLI implemented; release/distribution and renderer work remains.

The Rust CLI is a separate application, not a production shared library.
Continue to make its supported input/output behavior reliable while keeping
renderer contracts independent of the CLI and any future runtime binding.

Relevant work:

- [#49 — CLI umbrella](https://github.com/vsimko/corrplot-ng/issues/49), including
  [#151 — Native release artifacts](https://github.com/vsimko/corrplot-ng/issues/151)
  and [#152 — Installation layout and documentation](https://github.com/vsimko/corrplot-ng/issues/152).
- [#97 — CLI distribution umbrella](https://github.com/vsimko/corrplot-ng/issues/97).
- [#153 — Renderer-neutral output interface](https://github.com/vsimko/corrplot-ng/issues/153).
- [#154 — PNG and RGBA raster rendering](https://github.com/vsimko/corrplot-ng/issues/154).

**Dependency gate:** complete the renderer interface and capability model
before implementing additional backends. The SVG backend must continue to
conform to the existing scene semantics. Raster work depends on the output
contract and renderer capability negotiation.

**Measurable outcomes:**

- A clean environment can install a documented native CLI release and
  render a sample without Python or R.
- Supported platforms, runtime assets, checksums, and installation steps are
  documented and tested.
- SVG and any added raster output consume the same scene semantics.
- Unsupported renderer features are detected before a partial output is
  written.
- Output dimensions, scaling, alpha, background, and pixel-buffer ownership
  have explicit tests and documentation.

Linux distribution repository adoption and DEB/RPM automation remain
explicitly deferred under [#99](https://github.com/vsimko/corrplot-ng/issues/99).
Do not start that work until the native CLI release process is stable and
the issue is deliberately reactivated.

### Phase 3 — Establish the shared Rust core

**Status:** Strategic architecture selected; CLI/R crate-reuse proof and native Node binding slices complete; browser WASM and complete scene construction remain.

The target is one authoritative Rust implementation of shared correlation,
validation, scene construction, layout, and other common visualization
semantics. The CLI, R, Node, and WebAssembly consumers should call that same
core through thin adapters. This avoids semantic drift, duplicated fixes,
and multiple implementations to maintain, while keeping performance-critical
logic in one place.

Rust is selected as the shared-core language because the standalone CLI is
already Rust and the same source can serve native and WebAssembly targets.
Merged PR #180 extracted `crates/corrplot-core` and proved that both the CLI
and disposable R/rextendr adapter call its matrix-validation operation.
This is a real but deliberately small compiled-library reuse proof; the core
does not yet implement the complete visualization pipeline.

The cross-runtime architecture decision is recorded in [ADR 0002](adr/0002-cross-runtime-typescript-architecture.md). The remaining implementation track is:

- [#139 — Cross-runtime Rust and TypeScript architecture](https://github.com/vsimko/corrplot-ng/issues/139) — architecture decision recorded in ADR 0002.
- [#140 — Portable Rust core for native and WebAssembly](https://github.com/vsimko/corrplot-ng/issues/140).
- [#141 — Node.js native binding](https://github.com/vsimko/corrplot-ng/issues/141) — initial N-API validation, typed-scene SVG/RGBA rendering, and clean package-install smoke test are implemented; scene construction and prebuilt distribution remain.
- [#142 — Browser WebAssembly backend](https://github.com/vsimko/corrplot-ng/issues/142).
- [#145 — Unified TypeScript API](https://github.com/vsimko/corrplot-ng/issues/145).

**Completed integration gate:** the CLI and disposable R/rextendr adapter
depend on the exact same reusable crate and call its matrix-validation API.
PR #184 records Rust/Cargo 1.78, extendr 0.9.0, three-platform offline
package checks, and measured source/installed package sizes. This proof does
not establish a complete visualization engine or CRAN acceptance.

**Remaining rollout gates:** obtain CRAN guidance on the vendor metadata
NOTE and installed package size, and measure actual human maintainer effort
against the R-only C++/Rcpp path. These gates apply to production R adoption;
they must not block independent Node/WASM architecture and portability work.

**Next implementation gate:** extend the same crate with portable shared
semantics and build it for native and WASM targets. Bindings must call that
same crate; shared JSON fixtures alone are not proof.

If the R integration cannot meet its packaging constraints, keep the Rust
core authoritative for the CLI and future consumers and assess a narrowly
scoped R adapter exception. Do not fork common semantics silently. Continue
to use normalized-matrix v1 and scene-v1 as stable language-neutral contracts.

**Measurable outcomes for the remaining core/runtime work:**

- [ADR 0002](adr/0002-cross-runtime-typescript-architecture.md) records runtime boundaries, backend selection, portability constraints, and rejected alternatives.
- The same semantic core builds for the approved native and WASM targets.
- At least two real consumers exercise the same operation and shared fixtures.
- Packaging and clean-environment installation work on the declared support
  matrix without requiring a developer toolchain at runtime.
- The additional maintenance burden is justified by demonstrated user value.

### Phase 4 — Add output backends and parity only when consumers exist

**Status:** Future work, dependent on Phases 1–3.

Potential renderer and parity work includes:

- [#144 — Direct browser Canvas rendering](https://github.com/vsimko/corrplot-ng/issues/144).
- [#143 — Multi-format renderer umbrella](https://github.com/vsimko/corrplot-ng/issues/143).
- [#146 — Cross-backend parity, visual regression, and performance tests](https://github.com/vsimko/corrplot-ng/issues/146).

Canvas is a browser adapter, not an SVG intermediate. Raster and Canvas
renderers must declare capabilities and preserve the scene contract. Visual
comparisons should use structural checks and reviewed tolerances appropriate
to the output format; byte equality is not a universal cross-renderer
criterion.

**Dependency gate:** do not build a backend without a real consumer and a
defined output contract. Broad parity and performance suites follow working
native/WASM/renderer paths; they must not become long-running required checks
on every pull request.

**Measurable outcomes:**

- Each renderer declares supported scene features and schema versions.
- Shared semantic fixtures pass on each implemented backend.
- Visual tests detect meaningful regressions without treating different
  rasterizers or font environments as byte-identical.
- Performance measurements record the environment and remain separate from
  fast correctness checks until stable baselines justify a gate.

### Longer-term ideas

Interactive matrix exploration is described in
[#51](https://github.com/vsimko/corrplot-ng/issues/51). Migration tooling for
legacy R calls and additional visualization types may be considered when
there is a clear user need and a small, testable scope. Neither is a
prerequisite for the current contract, R adapter, or CLI work.

## Architecture decisions and non-goals

- Keep normalized-matrix v1 and scene-v1 as language-neutral contracts.
- Treat Rust as the authoritative implementation language for shared
  production semantics across CLI, R, Node, and WebAssembly.
- Keep language-specific adapters thin: R API/legacy compatibility and
  graphics-device behavior remain in R; CLI argument/file handling remains
  in the CLI; TypeScript runtime and DOM integration remain in adapters.
- Keep Python as a reference implementation and test oracle during migration,
  not a second production source of shared semantics.
- Do not promote the current Rust/rextendr or C++/Rcpp spikes directly;
  first prove the shared Rust library crate and pass its packaging gates.
- Do not claim full corrplot compatibility from the representative
  regression corpus.
- Do not promise dates or release milestones in this roadmap.
- Do not start deferred Linux package-manager work under #99 without
  deliberate reactivation.
- Do not duplicate correlation or scene semantics in each language adapter.

The authoritative shared-core language decision is
[ADR 0001: Shared-core architecture](adr/0001-shared-core-architecture.md).
The Node/browser runtime and API boundary is specified in
[ADR 0002: Cross-runtime Rust and TypeScript architecture](adr/0002-cross-runtime-typescript-architecture.md).
Changes to either decision require an explicit update to the relevant ADR.

## How to use this roadmap

Keep this document short enough to show sequence and decision gates. Put
implementation details, task breakdowns, and acceptance checklists in GitHub
issues. Update the current baseline only when merged code or tests provide
evidence. Move work between phases when dependencies change; do not infer
priority or delivery dates from issue numbers.
