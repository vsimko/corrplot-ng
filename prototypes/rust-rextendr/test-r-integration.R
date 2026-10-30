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
