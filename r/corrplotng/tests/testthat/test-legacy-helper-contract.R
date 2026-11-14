test_that("legacy cor.mtest preserves its pairwise result contract", {
  data <- datasets::mtcars[, c("mpg", "cyl", "disp", "hp")]

  for (method in c("pearson", "spearman")) {
    expected <- corrplot::cor.mtest(data, method = method, conf.level = 0.9)
    actual <- corrplotng::cor.mtest(data, method = method, conf.level = 0.9)

    expect_identical(names(actual), c("p", "lowCI", "uppCI"))
    expect_identical(actual, expected)

    for (component in names(actual)) {
      expect_true(is.matrix(actual[[component]]))
      expect_identical(dim(actual[[component]]), c(ncol(data), ncol(data)))
      expect_identical(
        dimnames(actual[[component]]),
        list(colnames(data), colnames(data))
      )
    }
  }
})

test_that("legacy sequential and diverging palettes match every pinned palette", {
  sequential <- c(
    "Oranges", "Purples", "Reds", "Blues", "Greens",
    "Greys", "OrRd", "YlOrRd", "YlOrBr", "YlGn"
  )
  diverging <- c("RdBu", "BrBG", "PiYG", "PRGn", "PuOr", "RdYlBu")

  for (palette in sequential) {
    expected <- corrplot::COL1(sequential = palette, n = 17)
    actual <- corrplotng::COL1(sequential = palette, n = 17)

    expect_identical(actual, expected, info = paste("COL1 palette:", palette))
    expect_length(actual, 17)
    expect_silent(grDevices::col2rgb(actual))
  }

  for (palette in diverging) {
    expected <- corrplot::COL2(diverging = palette, n = 17)
    actual <- corrplotng::COL2(diverging = palette, n = 17)

    expect_identical(actual, expected, info = paste("COL2 palette:", palette))
    expect_length(actual, 17)
    expect_silent(grDevices::col2rgb(actual))
  }
})
