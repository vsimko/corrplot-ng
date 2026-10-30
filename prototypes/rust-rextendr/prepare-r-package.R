# Generate a disposable R package using rextendr's supported scaffolding.
# The generated package is intentionally not committed.
args <- commandArgs(trailingOnly = TRUE)
root <- if (length(args)) args[[1]] else "prototypes/rust-rextendr"
root <- normalizePath(root, mustWork = TRUE)
pkg_dir <- file.path(root, ".ci-r-package")

if (dir.exists(pkg_dir)) {
  unlink(pkg_dir, recursive = TRUE, force = TRUE)
}
dir.create(pkg_dir, recursive = TRUE)

writeLines(c(
  "Package: corrplotngrustspike",
  "Title: Temporary Rust Integration Smoke Test",
  "Version: 0.0.0.9000",
  "Authors@R: person(\"CI\", \"Runner\", email = \"ci@example.invalid\", role = c(\"aut\", \"cre\"))",
  "Description: Temporary package used to test the rextendr Rust-to-R boundary.",
  "License: MIT",
  "Encoding: UTF-8"
), file.path(pkg_dir, "DESCRIPTION"))

# Supply the standard MIT license file required by R package checks.
writeLines(c(
  "YEAR: 2026",
  "COPYRIGHT HOLDER: corrplot-ng contributors"
), file.path(pkg_dir, "LICENSE"))

dir.create(file.path(pkg_dir, "R"))
writeLines("# Temporary package generated in CI.", file.path(pkg_dir, "R", "placeholder.R"))

# Preserve the upstream MIT notice for the extendr crates. The source notice is
# pinned in this repository to the exact upstream extendr-api-v0.8.2 tag.
license_source <- file.path(root, "licenses", "extendr-MIT.txt")
license_readme_source <- file.path(root, "licenses", "README.md")
if (!file.exists(license_source) || !file.exists(license_readme_source)) {
  stop("Pinned extendr license notice or its provenance README is missing")
}
license_dir <- file.path(pkg_dir, "inst", "licenses")
dir.create(license_dir, recursive = TRUE)
if (!file.copy(license_source, file.path(license_dir, "extendr-MIT.txt"), overwrite = TRUE) ||
    !file.copy(license_readme_source, file.path(license_dir, "README.md"), overwrite = TRUE)) {
  stop("Could not copy extendr third-party license notices into generated package")
}

# usethis/rextendr discover the package from the working directory. Enter the
# disposable package before invoking the scaffolding helpers.
setwd(pkg_dir)
rextendr::use_extendr(path = ".", quiet = TRUE, overwrite = TRUE)

# rextendr's generated Makevars sets TARGET_DIR to ./rust/target and removes
# that directory in its rust_clean target. Override both behaviors so Cargo
# writes to the persistent CI cache directory and the build does not delete it.
target_dir <- normalizePath(
  file.path(root, ".ci-rust-target"),
  winslash = "/",
  mustWork = FALSE
)
makevars_file <- file.path(
  pkg_dir,
  "src",
  if (.Platform[["OS.type"]] == "windows") "Makevars.win.in" else "Makevars.in"
)
makevars <- readLines(makevars_file, warn = FALSE)

target_dir_idx <- grep("^TARGET_DIR[[:space:]]*=[[:space:]]*\\./rust/target[[:space:]]*$", makevars)
if (length(target_dir_idx) != 1L) {
  stop("Could not identify the generated rextendr TARGET_DIR setting in ", makevars_file)
}
makevars[target_dir_idx] <- paste0("TARGET_DIR = ", target_dir)

rust_clean_idx <- grep("^rust_clean:", makevars)
clean_idx <- grep("^clean:", makevars)
if (length(rust_clean_idx) != 1L || length(clean_idx) != 1L ||
    clean_idx <= rust_clean_idx) {
  stop("Could not identify the generated rextendr cleanup targets in ", makevars_file)
}
cleanup_candidates <- seq.int(rust_clean_idx + 1L, clean_idx - 1L)
cleanup_idx <- cleanup_candidates[
  vapply(
    makevars[cleanup_candidates],
    function(line) {
      grepl("@CLEAN_TARGET@", line, fixed = TRUE) ||
        grepl("$(TARGET_DIR)", line, fixed = TRUE)
    },
    logical(1)
  )
]
if (length(cleanup_idx) != 1L) {
  stop("Could not identify the target-directory cleanup in ", makevars_file)
}
makevars[cleanup_idx] <- gsub("@CLEAN_TARGET@", "", makevars[cleanup_idx], fixed = TRUE)
makevars[cleanup_idx] <- gsub("$(TARGET_DIR)", "", makevars[cleanup_idx], fixed = TRUE)
writeLines(makevars, makevars_file)

rust_src <- readLines(file.path(root, "src", "lib.rs"), warn = FALSE)
writeLines(rust_src, file.path(pkg_dir, "src", "rust", "src", "lib.rs"))

# pkgload installs the package before roxygen2 has generated NAMESPACE.
# Seed the required file so this first install can compile the Rust DLL.
namespace_file <- file.path(pkg_dir, "NAMESPACE")
if (!file.exists(namespace_file)) {
  writeLines(character(), namespace_file)
}

# This smoke test only needs one stable wrapper. Generate it directly instead
# of calling rextendr::document(), which builds Rust and cleans its target dir;
# R CMD INSTALL would then compile the same Rust code again.
# Keep this smoke test independent of wrapper-generation behavior across
# rextendr versions: the exported wrapper calls the registered extendr symbol.
wrapper_file <- file.path(pkg_dir, "R", "scene_cell_count.R")
if (!file.exists(wrapper_file) ||
    !grepl("scene_cell_count", paste(readLines(wrapper_file, warn = FALSE), collapse = "\\n"), fixed = TRUE)) {
  writeLines(c(
    "#' Return the number of cells in a square plot scene.",
    "#' @export",
    "scene_cell_count <- function(n_labels) .Call(wrap__scene_cell_count, n_labels)"
  ), wrapper_file)
}
# R CMD check expects an Rd page for every exported function.
man_dir <- file.path(pkg_dir, "man")
dir.create(man_dir, showWarnings = FALSE)
writeLines(c(
  "\\name{scene_cell_count}",
  "\\alias{scene_cell_count}",
  "\\title{Count Cells in a Plot Scene}",
  "\\description{Return the number of cells in a square plot scene.}",
  "\\usage{",
  "scene_cell_count(n_labels)",
  "}",
  "\\arguments{",
  "\\item{n_labels}{Number of labels in the scene.}",
  "}",
  "\\value{The number of cells in the scene.}",
  "\\keyword{internal}"
), file.path(man_dir, "scene_cell_count.Rd"))

namespace_lines <- readLines(namespace_file, warn = FALSE)
# The initial empty NAMESPACE prevents pkgload from running before Rust
# compilation, but it also makes rextendr skip its usual DLL registration.
# Add the registration directive explicitly so .Call can resolve wrappers.
dynlib_directive <- "useDynLib(corrplotngrustspike, .registration = TRUE)"
if (!any(trimws(namespace_lines) == dynlib_directive)) {
  namespace_lines <- c(namespace_lines, dynlib_directive)
}
if (!any(grepl("^export\\(scene_cell_count\\)$", namespace_lines))) {
  writeLines(c(namespace_lines, "export(scene_cell_count)"), namespace_file)
}
