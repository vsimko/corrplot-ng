# Language-neutral contracts

Canonical schemas: [normalized matrix v1](normalized-matrix-v1/schema.json), [scene v1](scene-v1/schema.json), [diagnostics v1](diagnostics-v1/schema.json), [renderer capability v1](renderer-capability-v1/schema.json), and [publication config v1](publication-config-v1/schema.json).

A `-vN` directory identifies a major contract version. Published vN schemas and semantics are immutable. Any change that can affect validation, interpretation, required behavior, or accepted document shape requires a new v(N+1) contract. This conservative rule is necessary because consumers reject unknown fields. Editorial clarification must not redefine normative meaning. Consumers explicitly declare supported versions; unknown versions are errors, not best-effort parses.

Contract changes update canonical fixtures and conformance tests in the same PR. Stable fixture IDs are not reused for different behavior. Runtime packages do not own competing schema copies.


Each schema's `$id` identifies its canonical schema document. It is not a substitute for the contract's in-document version field; consumers must validate both the document version and the supported contract directory.


Validation scripts and runtime test harnesses must reference these canonical paths; they must not retain fallback reads from retired prototype or test-fixture locations.


The repository foundation workflow validates the shared fixtures and schema references without requiring any ecosystem package manager.


## Normative documentation

- [Normalized matrix v1](normalized-matrix-v1/README.md)
- [Scene v1](scene-v1/README.md)
- [Diagnostics v1](diagnostics-v1/README.md)
- [Renderer capability v1](renderer-capability-v1/README.md)
- [Publication config v1](publication-config-v1/README.md), including [color-scale semantics](publication-config-v1/COLOR-SCALES.md)
