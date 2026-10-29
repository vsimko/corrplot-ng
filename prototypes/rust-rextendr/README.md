# Rust integration spike

This directory is an isolated experiment. It does not change the production R package or select Rust for the shared core.

## Purpose

Check the smallest useful Rust-to-R integration with `extendr` and `rextendr`. The function in this spike computes the number of cells in a square plot scene. It is only a binding smoke test, not production plot logic.

## Prerequisites

- R
- The CRAN package `rextendr`
- A Rust toolchain supported by `rextendr`
- Platform build tools required by the local R installation

Install the R package with:

```r
install.packages("rextendr")
```

Set up a temporary R package and let `rextendr` generate its platform-specific build files and R wrappers. Then copy `src/lib.rs` into the generated Rust crate and build/test it with `R CMD INSTALL` and `R CMD check`. The generated Makevars files must not be hand-written from memory; use the installed `rextendr` scaffolding for the current toolchain.

## Experiment boundary

- Keep this spike separate from `implementations/r/`.
- Do not replace `prototypes/shared-svg/`.
- Do not add Rust as a dependency of the user-facing R package yet.
- Do not treat a local source build as proof that Windows users can install without a toolchain.

## Questions to answer

1. Can the generated R wrapper call the Rust function on Windows, macOS, and Linux in CI?
2. What files and toolchain versions does `rextendr` generate or require?
3. Can a built Windows binary be installed and used on a clean machine without Rust or Rtools?
4. What vendoring, licensing, and CRAN checks would be needed before production use?

A successful local build answers only the binding question. Distribution and maintenance remain separate acceptance criteria.
