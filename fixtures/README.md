# Shared conformance fixtures

Language-neutral, deterministic, versioned test inputs and expected outputs consumed by multiple runtimes.

- [normalized-matrix-v1](normalized-matrix-v1/): valid and invalid adapter cases.
- [scene-v1](scene-v1/): canonical prepared scenes and statistical overlays.
- [matrix-input-v1](matrix-input-v1/): correlation and general-numeric matrix corpora.
- [legacy-visual](legacy-visual/): pinned corrplot 0.95 visual-case manifest and provenance for R pixel baselines.
- [publication](publication/): small publication/LaTeX interoperability fixture(s).

Keep JSON UTF-8 and identify cases with stable IDs. Runtime-specific harnesses stay in their owning projects and consume these files. Renderer-specific snapshots remain with their renderer. A standalone package may carry an exact byte-for-byte mirror only when required for distribution (currently R `inst/extdata`); repository CI verifies the mirror against the canonical fixture, and changes are made to the canonical file first.

Treat expected-output changes as contract changes: update the canonical fixture, its consumer assertions, and the relevant compatibility documentation in the same pull request.
