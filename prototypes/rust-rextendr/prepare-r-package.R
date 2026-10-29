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

dir.create(file.path(pkg_dir, "R"))
writeLines("# Temporary package generated in CI.", file.path(pkg_dir, "R", "placeholder.R"))

# usethis/rextendr discover the package from the working directory. Enter the
# disposable package before invoking the scaffolding helpers.
setwd(pkg_dir)
rextendr::use_extendr(path = ".", quiet = TRUE, overwrite = TRUE)

rust_src <- readLines(file.path(root, "src", "lib.rs"), warn = FALSE)
writeLines(rust_src, file.path(pkg_dir, "src", "rust", "src", "lib.rs"))

# Generate the R wrappers from the temporary package directory.
devtools::document(pkg = pkg_dir, quiet = TRUE)
