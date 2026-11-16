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


test_that("legacy cor.mtest forwards Kendall and cor.test options exactly", {
  data <- data.frame(
    x = c(1, 2, 3, 4, 5, 6, 7, 8),
    y = c(2, 1, 4, 3, 6, 5, 8, 7),
    z = c(1, 3, 2, 5, 4, 7, 6, 8)
  )

  for (method in c("pearson", "spearman", "kendall")) {
    expected <- corrplot::cor.mtest(
      data, method = method, alternative = "greater", conf.level = 0.9
    )
    actual <- corrplotng::cor.mtest(
      data, method = method, alternative = "greater", conf.level = 0.9
    )

    expect_identical(actual, expected, info = paste("method:", method))
    expect_identical(names(actual), c("p", "lowCI", "uppCI"))
    expect_identical(diag(actual$p), c(0, 0, 0))
    expect_identical(diag(actual$lowCI), c(1, 1, 1))
    expect_identical(diag(actual$uppCI), c(1, 1, 1))
  }

  kendall <- corrplotng::cor.mtest(data, method = "kendall")
  expect_true(all(is.na(kendall$lowCI[upper.tri(kendall$lowCI)])))
  expect_true(all(is.na(kendall$uppCI[upper.tri(kendall$uppCI)])))
})

test_that("legacy cor.mtest missing-value errors match the pinned oracle", {
  data <- data.frame(
    x = c(1, 2, NA_real_, 4, 5, 6),
    y = c(2, 1, 4, 3, 6, 5),
    z = c(3, 5, 2, 6, 1, 4)
  )

  expected <- tryCatch(
    corrplot::cor.mtest(data, method = "pearson"),
    error = identity
  )
  actual <- tryCatch(
    corrplotng::cor.mtest(data, method = "pearson"),
    error = identity
  )

  expect_s3_class(expected, "error")
  expect_s3_class(actual, "error")
  expect_identical(conditionMessage(actual), conditionMessage(expected))
})
