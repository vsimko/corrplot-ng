testthat::test_that("scene-v1 JSON is returned for a small matrix", {
  labels <- c("a", "b")
  correlations <- matrix(c(1, 0.2, 0.2, 1), nrow = 2)
  scene <- build_scene_json_cpp(labels, correlations, "test")
  testthat::expect_type(scene, "character")
  testthat::expect_length(scene, 1L)
  testthat::expect_match(scene, '\"schema_version\":1')
  testthat::expect_match(scene, '\"annotation\":\"test\"')
})

testthat::test_that("dimension mismatches are rejected", {
  testthat::expect_error(
    build_scene_json_cpp("a", matrix(1:4, nrow = 2)),
    "dimensions must agree"
  )
})
