# Generate a disposable R package using rextendr's supported scaffolding.
# The generated package is intentionally not committed.
args <- commandArgs(trailingOnly = TRUE)
root <- if (length(args)) args[[1]] else "prototypes/rust-rextendr"
root <- normalizePath(root, mustWork = TRUE)
pkg_dir <- file.path(root, ".ci-r-package")

if (dir.exists(pkg_dir)) unlink(pkg_dir, recursive = TRUE, force = TRUE)
dir.create(pkg_dir, recursive = TRUE)

writeLines(c(
  "Package: corrplotngrustspike",
  "Title: Temporary Rust Scene-v1 Benchmark",
  "Version: 0.0.0.9000",
  "Authors@R: person(\"CI\", \"Runner\", email = \"ci@example.invalid\", role = c(\"aut\", \"cre\"))",
  "Description: Temporary package used to benchmark the Rust scene-v1 builder.",
  "License: MIT",
  "Encoding: UTF-8"
), file.path(pkg_dir, "DESCRIPTION"))

dir.create(file.path(pkg_dir, "R"))
writeLines("# Temporary package generated in CI.", file.path(pkg_dir, "R", "placeholder.R"))
setwd(pkg_dir)
rextendr::use_extendr(path = ".", quiet = TRUE, overwrite = TRUE)

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
  "#' @export",
  "build_scene_json_rust <- function(labels, matrix, annotation = \"\") {",
  "  .Call(wrap__build_scene_json_rust, labels, matrix, annotation)",
  "}"
), wrapper_file)
if (!any(grepl("^export\\(build_scene_json_rust\\)$", namespace_lines))) {
  namespace_lines <- c(namespace_lines, "export(build_scene_json_rust)")
}
writeLines(namespace_lines, namespace_file)
