# Rust dependency audit results

Audit run: [GitHub Actions run 41020325794](https://github.com/vsimko/corrplot-ng/actions/runs/41020325794)
Audited revision: `c2e0f1317e5072faafdc0cfa063ec13e75db3173`
Scope: the `prototypes/rust-rextendr` Cargo dependency graph only.

This is an initial dependency and vendoring inventory. It is not a legal opinion, a complete CRAN compliance review, or proof that the eventual R source package can be built offline.

## Resolved dependency set

Cargo resolved `extendr-api 0.8.2` and eight transitive crates:

| Crate | Version | Declared license |
|---|---:|---|
| extendr-api | 0.8.2 | MIT |
| extendr-ffi | 0.8.2 | MIT |
| extendr-macros | 0.8.2 | MIT |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| paste | 1.0.15 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |

All nine third-party crates declare permissive license identifiers. The metadata screen does not replace review of the actual license texts or CRAN policy.

## Vendoring measurements and notices

- Vendored source tree: approximately 5.4 MiB uncompressed.
- `vendor.tar.xz`: 523,328 bytes (about 511 KiB).
- The vendored sources for `once_cell`, `paste`, `proc-macro2`, `quote`, `syn`, and `unicode-ident` include license text files.
- The three extendr crates declare MIT in Cargo metadata, but their individual vendored crate directories do not contain a license text file. The upstream extendr repository publishes an MIT `LICENSE`; the eventual R source package should include that notice explicitly.
- `unicode-ident` includes its Unicode license text as well as MIT and Apache-2.0 texts.

## Reproducibility and CI status

The successful run linked above used an earlier version of the dependency-audit workflow. Its job resolved dependencies and generated a lockfile during the run before capturing metadata and vendoring sources. It therefore demonstrates that the audit steps completed for that run, but it does **not** demonstrate that CI validated a committed lockfile with `--locked`.

A later workflow revision proposed in [PR #30](https://github.com/vsimko/corrplot-ng/pull/30) adds a committed `Cargo.lock`, checks it with `cargo metadata --locked`, and uses locked Cargo operations for metadata capture, vendoring, and offline compilation. That workflow revision is separate from this report; its locked path should be treated as validated only after its own CI run succeeds.

The full vendoring and offline build are intended as scheduled or manually dispatched checks, rather than work on every ordinary PR. A lightweight license-policy check can run when Rust dependency inputs change.

## Interpretation and next steps

The dependency graph is small, and the source archive is compact enough to investigate inclusion in an R source package. The audit also exposed a packaging task: license notices cannot simply be assumed to be present in every Cargo vendor directory.

Before considering production use or a CRAN submission:

1. Include all required third-party license notices in a reproducible package-generation step.
2. Verify the committed lockfile and locked audit workflow in CI.
3. Test a clean source build using only vendored sources and Cargo offline mode as part of the actual package-generation path.
4. Build an actual R source tarball and run `R CMD check --as-cran` on supported platforms.
5. Reassess CRAN policy and package metadata.

No production R package files are changed by this report.
