x <- build_scene_json_rust(
  c("a", "b"),
  matrix(c(1, 0.2, 0.2, 1), nrow = 2),
  "offline smoke"
)
stopifnot(
  is.character(x),
  length(x) == 1L,
  grepl('"schema_version":1', x, fixed = TRUE)
)
cat("Offline Rust-to-R integration smoke test passed.\n")

scene_fixture <- readLines(
  "prototypes/shared-svg/scene-v1-construction.example.json",
  warn = FALSE
)
scene_json <- paste(scene_fixture, collapse = "\n")
scene_svg <- render_scene_svg_rust(scene_json)
stopifnot(
  is.character(scene_svg),
  length(scene_svg) == 1L,
  grepl("<svg ", scene_svg, fixed = TRUE),
  grepl("A caption", scene_svg, fixed = TRUE),
  grepl("second line", scene_svg, fixed = TRUE)
)
cat("Shared scene-v1 construction fixture passed through Rust/rextendr.\n")
