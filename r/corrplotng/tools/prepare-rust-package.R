args <- commandArgs(trailingOnly = TRUE)
script_arg <- grep("^--file=", commandArgs(), value = TRUE)
if (length(script_arg) != 1L) {
  stop("Could not determine the package preparation script path", call. = FALSE)
}
script_path <- normalizePath(sub("^--file=", "", script_arg[[1L]]), mustWork = TRUE)
package_root <- normalizePath(file.path(dirname(script_path), ".."), mustWork = TRUE)
repository_root <- normalizePath(file.path(package_root, "..", ".."), mustWork = TRUE)
core_source <- file.path(repository_root, "rust", "crates", "corrplot-core")
core_target <- file.path(package_root, "src", "rust", "corrplot-core")

required <- c("Cargo.toml", "src/lib.rs", "src/scene.rs", "src/raster.rs",
              "src/output.rs", "src/json.rs", "renderer-capabilities.json")
missing <- required[!file.exists(file.path(core_source, required))]
if (length(missing)) {
  stop("Shared Rust core is incomplete: ", paste(missing, collapse = ", "),
       call. = FALSE)
}

if (dir.exists(core_target)) {
  unlink(core_target, recursive = TRUE, force = TRUE)
}
dir.create(core_target, recursive = TRUE, showWarnings = FALSE)
core_entries <- list.files(core_source, all.files = TRUE, no.. = TRUE,
                           full.names = TRUE)
if (!length(core_entries) ||
    !all(file.copy(core_entries, core_target, recursive = TRUE))) {
  stop("Could not copy the authoritative shared Rust core into the R source package",
       call. = FALSE)
}

rust_dir <- file.path(package_root, "src", "rust")
setwd(rust_dir)

run_checked <- function(command, arguments, stdout = FALSE) {
  result <- system2(command, arguments, stdout = stdout, stderr = TRUE)
  status <- attr(result, "status")
  if (!is.null(status) && status != 0L) {
    stop(command, " failed with exit status ", status, call. = FALSE)
  }
  invisible(result)
}

run_checked("cargo", c("metadata", "--locked", "--format-version", "1"))
if (dir.exists("vendor")) {
  unlink("vendor", recursive = TRUE, force = TRUE)
}
if (file.exists("vendor.tar.xz")) unlink("vendor.tar.xz")
run_checked("cargo", c("vendor", "--locked", "vendor"), stdout = "vendor-config.toml")

vendor_config <- readLines("vendor-config.toml", warn = FALSE)
directory_line <- grep('directory = "vendor"', vendor_config, fixed = TRUE)
if (length(directory_line) != 1L) {
  stop("cargo vendor output did not contain exactly one vendor directory", call. = FALSE)
}
vendor_config[[directory_line]] <- 'directory = "rust/vendor"'
writeLines(vendor_config, "vendor-config.toml")

run_checked("tar", c("-cJf", "vendor.tar.xz", "-C", "..", "rust/vendor"))
unlink("vendor", recursive = TRUE, force = TRUE)

message("Prepared the corrplotng source package with the authoritative shared Rust core and offline Cargo vendor archive.")
