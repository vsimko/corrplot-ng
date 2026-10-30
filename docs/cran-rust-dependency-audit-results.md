# Rust dependency audit results

Audit run: [GitHub Actions run 41020325794](https://github.com/vsimko/corrplot-ng/actions/runs/41020325794)
Audited revision: `c2e0f1317e5072faafdc0cfa063ec13e75db3173`
Scope: `prototypes/rust-rextendr` Cargo dependency graph only.

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

All nine third-party crates declare permissive license identifiers. This is an initial metadata screen, not a legal opinion or a complete CRAN license approval.

## Vendoring measurements and notices

- Vendored source tree: approximately 5.4 MiB uncompressed.
- `vendor.tar.xz`: 523,328 bytes (about 511 KiB).
- The vendored sources for `once_cell`, `paste`, `proc-macro2`, `quote`, `syn`, and `unicode-ident` include license text files.
- The three extendr crates declare MIT in Cargo metadata, but their individual vendored crate directories do not contain a license text file. The upstream extendr repository publishes an MIT `LICENSE`; the eventual R source package should include that notice explicitly.
- `unicode-ident` includes its Unicode license text as well as MIT and Apache-2.0 texts.

## Interpretation

The dependency graph is small, and the source archive is compact enough to investigate inclusion in an R source package. The first audit also exposed a packaging task: license notices cannot simply be assumed to be present in every Cargo vendor directory.

The workflow currently generates the lockfile and vendor archive as a short-lived CI artifact. Neither is committed as the package's offline build input yet. The generated R package is still disposable CI scaffolding, so this does not establish that a CRAN source tarball can build offline.

## Next validation steps

1. Include the required third-party license notices in a reproducible package-generation step.
2. Pin the resolved dependency graph with a committed `Cargo.lock`.
3. Test a clean source build using only the vendored sources and Cargo offline mode.
4. Build an actual R source tarball and run `R CMD check --as-cran` on supported platforms.
5. Reassess CRAN policy and package metadata before any submission.

No production R package files are changed by this audit.
