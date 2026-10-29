# C++ / Rcpp integration spike

This directory contains a deliberately small integration experiment, comparable to `prototypes/rust-rextendr`. It is not production code and does not change the Python scene prototype or the R compatibility package.

## What it tests

- Compile one C++ function and expose it to R through Rcpp.
- Generate the R/C++ registration glue with `Rcpp::compileAttributes()`.
- Build and install a minimal R package, then call the function from R.
- Run the smoke test on Linux, macOS, and Windows in CI.

The function returns the number of cells in an n-by-n scene: `n_labels * n_labels`.

## Local smoke test

From the repository root, with R and a working C++ toolchain installed:

```sh
Rscript -e 'install.packages("Rcpp", repos = "https://cloud.r-project.org")'
Rscript -e 'Rcpp::compileAttributes("prototypes/cpp-rcpp")'
R CMD INSTALL prototypes/cpp-rcpp
Rscript prototypes/cpp-rcpp/tests/smoke-test.R
```

On Windows, building from source requires the Rtools toolchain matching the installed R version. Installing a prebuilt Windows binary should not require end users to install Rtools; this spike does not yet verify that binary-distribution path.

## Maintenance notes

Rcpp uses R's established compiled-code package workflow. The package needs a compatible C++ compiler when building from source, the Rcpp package dependency, generated registration wrappers, and platform CI. Compare these observed steps with the Rust/rextendr spike before selecting a production core language.

## Open questions

- Verify binary installation on Windows without Rtools or another compiler.
- Check current CRAN policies and toolchain requirements before any release claim.
- Compare maintenance and release workflows with the Rust/rextendr experiment.
