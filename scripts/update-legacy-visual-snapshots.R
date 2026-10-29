#!/usr/bin/env Rscript

# Regenerate committed legacy visual snapshots intentionally.
# Run from the repository root:
#   UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 Rscript scripts/update-legacy-visual-snapshots.R

if (Sys.getenv("UPDATE_LEGACY_VISUAL_SNAPSHOTS") != "1") {
  stop("Set UPDATE_LEGACY_VISUAL_SNAPSHOTS=1 to confirm snapshot regeneration.")
}

required <- c("corrplot", "jsonlite")
missing <- required[!vapply(required, requireNamespace, logical(1), quietly = TRUE)]
if (length(missing)) {
  stop("Install required packages first: ", paste(missing, collapse = ", "))
}

manifest_path <- "implementations/r/tests/testthat/fixtures/legacy-visual-cases.json"
cases_path <- "implementations/r/tests/testthat/helper-legacy-visual-cases.R"
snapshot_dir <- "implementations/r/tests/testthat/fixtures/legacy-visual"

manifest <- jsonlite::fromJSON(manifest_path)
if (as.character(utils::packageVersion("corrplot")) != manifest$reference_version) {
  stop("Expected corrplot ", manifest$reference_version, "; found ",
       as.character(utils::packageVersion("corrplot")))
}

# The shared helper defines the same 15 cases used by testthat.
source(cases_path, local = TRUE)
dir.create(snapshot_dir, recursive = TRUE, showWarnings = FALSE)

render_case <- function(case, path) {
  grDevices::png(
    filename = path, width = 900, height = 900, units = "px",
    res = 120, bg = "white", type = "cairo"
  )
  on.exit(grDevices::dev.off(), add = TRUE)
  graphics::par(mar = c(2, 2, 2, 2), xaxs = "i", yaxs = "i")
  case(function(corr, ...) corrplot::corrplot(corr = corr, ...))
}

if (!setequal(manifest$cases$id, names(legacy_visual_cases))) {
  stop("Manifest case IDs do not match executable legacy visual cases.")
}

for (case_id in names(legacy_visual_cases)) {
  output <- file.path(snapshot_dir, paste0(case_id, ".png"))
  render_case(legacy_visual_cases[[case_id]], output)
  message("Wrote ", output)
}
message("Snapshots generated with corrplot ", manifest$reference_version,
        ". Review the PNGs before committing them.")
