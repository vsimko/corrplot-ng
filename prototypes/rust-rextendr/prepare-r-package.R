# Generate a disposable R package using rextendr's supported scaffolding.
# The generated package is intentionally not committed.
args <- commandArgs(trailingOnly = TRUE)
root <- if (length(args)) args[[1]] else "prototypes/rust-rextendr"
root <- normalizePath(root, mustWork = TRUE)
pkg_dir <- file.path(root, ".ci-r-package")

if (dir.exists(pkg_dir)) unlink(pkg_dir, recursive = TRUE, force = TRUE)
dir.create(pkg_dir, recursive = TRUE)

# Preserve the upstream extendr license notice in the generated R package.
license_source <- file.path(root, "licenses", "extendr-MIT.txt")
license_readme_source <- file.path(root, "licenses", "README.md")
if (!file.exists(license_source) || !file.exists(license_readme_source)) {
  stop("Pinned extendr license notice or provenance README is missing")
}
license_dir <- file.path(pkg_dir, "inst", "licenses")
dir.create(license_dir, recursive = TRUE)
if (!file.copy(license_source, file.path(license_dir, "extendr-MIT.txt"), overwrite = TRUE) ||
    !file.copy(license_readme_source, file.path(license_dir, "README.md"), overwrite = TRUE)) {
  stop("Could not copy extendr third-party license notices into generated package")
}

writeLines(c(
  "YEAR: 2026",
  "COPYRIGHT HOLDER: corrplot-ng contributors"
), file.path(pkg_dir, "LICENSE"))

writeLines(c(
  "Package: corrplotngrustspike",
  "Title: Temporary Rust Scene-V1 Benchmark",
  "Version: 0.1.0",
  "Authors@R: person(\"corrplot-ng\", \"contributors\", role = c(\"aut\", \"cre\"), email = \"maintainers@example.invalid\")",
  "Description: Temporary package used to validate the isolated Rust scene-v1 builder.",
  "License: MIT + file LICENSE",
  "Encoding: UTF-8",
  "Suggests: testthat",
  "SystemRequirements: Cargo, rustc"
), file.path(pkg_dir, "DESCRIPTION"))

dir.create(file.path(pkg_dir, "R"))
writeLines("# Temporary package generated in CI.", file.path(pkg_dir, "R", "placeholder.R"))
setwd(pkg_dir)
rextendr::use_extendr(path = ".", quiet = TRUE, overwrite = TRUE)

# Keep the generated package aligned with the audited and vendored extendr
# 0.9.0 API and license notice. Pinning avoids scaffold-version drift.
cargo_manifest <- file.path(pkg_dir, "src", "rust", "Cargo.toml")
cargo_lines <- readLines(cargo_manifest, warn = FALSE)
extendr_dependency <- grep("^extendr-api[[:space:]]*=", cargo_lines)
if (length(extendr_dependency) != 1L) {
  stop("Could not identify generated extendr-api dependency in ", cargo_manifest)
}
cargo_lines[extendr_dependency] <- 'extendr-api = "=0.9.0"'
core_dependency <- 'corrplot-core = { path = "../../../../../crates/corrplot-core" }'
if (!any(grepl("^corrplot-core[[:space:]]*=", cargo_lines))) {
  cargo_lines <- append(cargo_lines, core_dependency, after = extendr_dependency)
}
writeLines(cargo_lines, cargo_manifest)

# Reuse the reviewed transitive dependency graph from the committed spike
# lockfile. Only the generated root package identity differs.
package_name_idx <- grep("^name = ", cargo_lines)[1]
package_version_idx <- grep("^version = ", cargo_lines)[1]
package_name <- gsub('^name = "|"$', "", cargo_lines[package_name_idx])
package_version <- gsub('^version = "|"$', "", cargo_lines[package_version_idx])
lock_lines <- readLines(file.path(root, "Cargo.lock"), warn = FALSE)
root_name_idx <- grep('^name = "corrplotng_rust_spike"$', lock_lines)
if (length(root_name_idx) != 1L) stop("Could not identify the prototype root package in Cargo.lock")
lock_lines[root_name_idx] <- paste0('name = "', package_name, '"')
lock_lines[root_name_idx + 1L] <- paste0('version = "', package_version, '"')
writeLines(lock_lines, file.path(pkg_dir, "src", "rust", "Cargo.lock"))

# Keep Cargo output outside the generated package and preserve it across
# package clean steps, matching the existing offline-validation experiment.
target_dir <- normalizePath(file.path(root, ".ci-rust-target"),
                            winslash = "/", mustWork = FALSE)
makevars_file <- file.path(pkg_dir, "src",
  if (.Platform[["OS.type"]] == "windows") "Makevars.win.in" else "Makevars.in")
makevars <- readLines(makevars_file, warn = FALSE)
target_idx <- grep("^TARGET_DIR[[:space:]]*=[[:space:]]*\\./rust/target[[:space:]]*$", makevars)
if (length(target_idx) != 1L) stop("Could not identify generated TARGET_DIR in ", makevars_file)

makevars[target_idx] <- paste0("TARGET_DIR = ", target_dir)

rust_clean_idx <- grep("^rust_clean:", makevars)
clean_idx <- grep("^clean:", makevars)
if (length(rust_clean_idx) != 1L || length(clean_idx) != 1L || clean_idx <= rust_clean_idx) {
  stop("Could not identify generated cleanup targets in ", makevars_file)
}
cleanup_candidates <- seq.int(rust_clean_idx + 1L, clean_idx - 1L)
cleanup_idx <- cleanup_candidates[vapply(makevars[cleanup_candidates], function(line) {
  grepl("@CLEAN_TARGET@", line, fixed = TRUE) || grepl("$(TARGET_DIR)", line, fixed = TRUE)
}, logical(1))]
if (length(cleanup_idx) != 1L) stop("Could not identify target cleanup in ", makevars_file)
makevars[cleanup_idx] <- gsub("@CLEAN_TARGET@", "", makevars[cleanup_idx], fixed = TRUE)
makevars[cleanup_idx] <- gsub("$(TARGET_DIR)", "", makevars[cleanup_idx], fixed = TRUE)
writeLines(makevars, makevars_file)

rust_dir <- file.path(pkg_dir, "src", "rust", "src")
writeLines(readLines(file.path(root, "src", "lib.rs"), warn = FALSE),
           file.path(rust_dir, "lib.rs"))
writeLines(readLines(file.path(root, "src", "scene_v1.rs"), warn = FALSE),
           file.path(rust_dir, "scene_v1.rs"))

namespace_file <- file.path(pkg_dir, "NAMESPACE")
if (!file.exists(namespace_file)) writeLines(character(), namespace_file)
namespace_lines <- readLines(namespace_file, warn = FALSE)
dynlib_directive <- "useDynLib(corrplotngrustspike, .registration = TRUE)"
if (!any(trimws(namespace_lines) == dynlib_directive)) namespace_lines <- c(namespace_lines, dynlib_directive)

wrapper_file <- file.path(pkg_dir, "R", "build_scene_json_rust.R")
writeLines(c(
  "#' Build renderer-neutral scene-v1 JSON.",
  "#' @param labels Character vector of matrix labels.",
  "#' @param matrix Numeric square correlation matrix.",
  "#' @param annotation Optional annotation included in the scene.",
  "#' @return A single JSON string describing the scene-v1 object.",
  "#' @export",
  "build_scene_json_rust <- function(labels, matrix, annotation = \"\") {",
  "  .Call(wrap__build_scene_json_rust, labels, matrix, annotation)",
  "}"
), wrapper_file)
if (!any(grepl("^export\\(build_scene_json_rust\\)$", namespace_lines))) {
  namespace_lines <- c(namespace_lines, "export(build_scene_json_rust)")
}
writeLines(namespace_lines, namespace_file)

# Keep generated-package checks meaningful instead of accepting an empty
# temporary package: ship a help page, an executable example, and a test.
man_dir <- file.path(pkg_dir, "man")
dir.create(man_dir, showWarnings = FALSE)
writeLines(c(
  "\\name{build_scene_json_rust}",
  "\\alias{build_scene_json_rust}",
  "\\title{Build Renderer-Neutral Scene-V1 JSON}",
  "\\description{Builds a renderer-neutral scene-v1 JSON representation from labels, a numeric matrix, and optional annotation text.}",
  "\\usage{",
  "build_scene_json_rust(labels, matrix, annotation = \"\")",
  "}",
  "\\arguments{",
  "\\item{labels}{Character vector containing the matrix labels.}",
  "\\item{matrix}{Numeric square matrix containing the correlation values.}",
  "\\item{annotation}{Optional character string included as scene metadata.}",
  "}",
  "\\value{A character vector of length one containing the scene-v1 JSON.}",
  "\\examples{",
  "labels <- c(\"a\", \"b\")",
  "correlations <- matrix(c(1, 0.2, 0.2, 1), nrow = 2)",
  "scene <- build_scene_json_rust(labels, correlations, \"example\")",
  "stopifnot(is.character(scene), length(scene) == 1L)",
  "}"
), file.path(man_dir, "build_scene_json_rust.Rd"))

test_dir <- file.path(pkg_dir, "tests", "testthat")
dir.create(test_dir, recursive = TRUE)
writeLines(c(
  "if (requireNamespace(\"testthat\", quietly = TRUE)) {",
  "  testthat::test_check(\"corrplotngrustspike\")",
  "}"
), file.path(pkg_dir, "tests", "testthat.R"))
writeLines(c(
  "testthat::test_that(\"scene-v1 JSON is returned for a small matrix\", {",
  "  labels <- c(\"a\", \"b\")",
  "  correlations <- matrix(c(1, 0.2, 0.2, 1), nrow = 2)",
  "  scene <- build_scene_json_rust(labels, correlations, \"test\")",
  "  testthat::expect_type(scene, \"character\")",
  "  testthat::expect_length(scene, 1L)",
  "  testthat::expect_match(scene, \"schema_version\")",
  "})"
), file.path(test_dir, "test-scene-v1.R"))
