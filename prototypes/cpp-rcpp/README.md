# C++ / Rcpp scene-v1 benchmark spike

This isolated package implements the experimental scene-v1 builder and returns
the canonical scene as JSON. It exists only for a controlled comparison with
the Python reference and Rust/rextendr candidate.

The exported `build_scene_json_cpp(labels, matrix, annotation)` function
validates dimensions and coefficient ranges, preserves missing values as JSON
`null`, emits cells in row-major order, and applies the scene-v1 geometry and
style constants.

The package is built in CI using `Rcpp::compileAttributes()` and
`R CMD INSTALL`. It is not production code and does not modify
`implementations/r/` or the legacy `taiyun/corrplot` package.

Source builds require an R-compatible C++ toolchain. This benchmark does not
validate clean Windows binary installation or CRAN release readiness.
