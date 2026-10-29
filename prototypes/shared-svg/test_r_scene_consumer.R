# Smoke test that R can consume the language-neutral scene contract.
# Run from the repository root after installing jsonlite.
if (!requireNamespace("jsonlite", quietly = TRUE)) {
  stop("jsonlite is required for this prototype test")
}
scene_path <- file.path("prototypes", "shared-svg", "scene-v1.example.json")
scene <- jsonlite::fromJSON(scene_path, simplifyVector = FALSE)

stopifnot(identical(scene$schema_version, 1L) || identical(scene$schema_version, 1))
stopifnot(identical(scene$kind, "correlation_plot"))
stopifnot(length(scene$labels) == 3L)
stopifnot(length(scene$cells) == 9L)
stopifnot(is.numeric(scene$cells[[1]]$value), scene$cells[[1]]$value == 1)
stopifnot(identical(scene$cells[[6]]$value, NULL))
stopifnot(identical(scene$cells[[8]]$value, NULL))
stopifnot(identical(scene$style$positive_color, "#2166ac"))
cat("R scene contract consumer smoke test passed\n")
