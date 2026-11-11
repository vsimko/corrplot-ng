test_that("the Rust SVG renderer consumes scene-v1 objects from the R adapter", {
  skip_if_not_installed("jsonlite")
  corr <- matrix(c(1, 0.6, 0.6, 1), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  scene <- corrplot_ng(corr, engine = "scene", annotation = "Rust SVG")
  svg <- corrplot_ng_render_svg(scene)
  expect_type(svg, "character")
  expect_length(svg, 1L)
  expect_match(svg, "<svg ")
  expect_match(svg, "<circle")
})

test_that("the Rust SVG renderer preserves display and annotation metadata", {
  corr <- matrix(c(1, 0.4, 0.4, 1), 2, byrow = TRUE,
    dimnames = list(c("A", "B"), c("A", "B")))
  scene <- corrplot_ng(corr, engine = "scene", type = "upper", diag = FALSE)
  expect_identical(scene$display_type, "upper")
  expect_false(scene$show_diagonal)
  svg <- corrplot_ng_render_svg(scene)
  expect_match(svg, "<svg ")
})

test_that("the Rust SVG renderer rejects unsupported glyphs and non-scenes", {
  expect_error(corrplot_ng_render_svg(list()), "corrplot_ng_scene")
  scene <- corrplot_ng(diag(2), engine = "scene", method = "number")
  expect_error(corrplot_ng_render_svg(scene), "only circle glyph scenes")
})
