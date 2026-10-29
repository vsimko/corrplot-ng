# Rust integration spike

This directory is an isolated experiment. It does not change the production R package or select Rust for the shared core.

## Purpose

Check the smallest useful Rust-to-R integration with `extendr` and `rextendr`. The function in this spike computes the number of cells in a square plot scene. It is only a binding smoke test, not production plot logic.

## Local toolchain check

Prerequisites are R, the CRAN package `rextendr`, a Rust toolchain supported by `rextendr`, and platform build tools required by the local R installation.

```sh
cargo check --manifest-path prototypes/rust-rextendr/Cargo.toml
```

## End-to-end R integration check

The CI job uses `rextendr::use_extendr()` to generate a disposable R package with the platform-specific build files. It copies this spike's `src/lib.rs` into the generated crate, generates the R wrappers, installs the package, and calls the Rust function from R. The generated package is not committed.

To run the same steps locally, install `rextendr` and `devtools`, then run:

```sh
Rscript prototypes/rust-rextendr/prepare-r-package.R
R CMD INSTALL prototypes/rust-rextendr/.ci-r-package
Rscript -e "library(corrplotngrustspike); source('prototypes/rust-rextendr/test-r-integration.R')"
```

## Experiment boundary

- Keep this spike separate from `implementations/r/`.
- Do not replace `prototypes/shared-svg/`.
- Do not add Rust as a dependency of the user-facing R package yet.
- Do not treat a local source build as proof that Windows users can install without a toolchain.

## Questions to answer

1. Does the generated R wrapper call the Rust function on Windows, macOS, and Linux in CI?
2. What files and toolchain versions does `rextendr` generate or require?
3. Can a built Windows binary be installed and used on a clean machine without Rust or Rtools?
4. What vendoring, licensing, and CRAN checks would be needed before production use?

A successful source build answers the binding question only. Distribution and maintenance remain separate acceptance criteria.
