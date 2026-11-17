test_that("legacy palette helpers preserve interpolation sizes at boundary values", {
  sequential <- c(
    "Oranges", "Purples", "Reds", "Blues", "Greens",
    "Greys", "OrRd", "YlOrRd", "YlOrBr", "YlGn"
  )
  diverging <- c("RdBu", "BrBG", "PiYG", "PRGn", "PuOr", "RdYlBu")
  sizes <- c(1L, 2L, 3L, 17L, 200L)

  for (palette in sequential) {
    for (n in sizes) {
      expected <- corrplot::COL1(sequential = palette, n = n)
      actual <- corrplotng::COL1(sequential = palette, n = n)

      expect_identical(
        actual, expected,
        info = paste("COL1 palette:", palette, "n:", n)
      )
      expect_length(actual, n)
      expect_silent(grDevices::col2rgb(actual))
    }
  }

  for (palette in diverging) {
    for (n in sizes) {
      expected <- corrplot::COL2(diverging = palette, n = n)
      actual <- corrplotng::COL2(diverging = palette, n = n)

      expect_identical(
        actual, expected,
        info = paste("COL2 palette:", palette, "n:", n)
      )
      expect_length(actual, n)
      expect_silent(grDevices::col2rgb(actual))
    }
  }
})

test_that("legacy palette helpers preserve their default palette and size", {
  expect_identical(corrplotng::COL1(), corrplot::COL1())
  expect_identical(corrplotng::COL2(), corrplot::COL2())
  expect_length(corrplotng::COL1(), 200)
  expect_length(corrplotng::COL2(), 200)
})
