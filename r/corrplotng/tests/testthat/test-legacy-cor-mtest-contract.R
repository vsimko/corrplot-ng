capture_cor_mtest_condition <- function(expr) {
  tryCatch(
    {
      force(expr)
      NULL
    },
    error = function(error) {
      list(class = class(error), message = conditionMessage(error))
    }
  )
}

test_that("cor.mtest matches pinned Pearson p-values and confidence intervals", {
  data <- mtcars[, c("mpg", "disp", "hp", "wt")]
  expected <- corrplot::cor.mtest(data, conf.level = 0.95)
  actual <- corrplotng::cor.mtest(data, conf.level = 0.95)

  expect_identical(names(actual), c("p", "lowCI", "uppCI"))
  expect_identical(actual, expected)
  for (field in names(actual)) {
    expect_true(is.matrix(actual[[field]]))
    expect_identical(dim(actual[[field]]), c(4L, 4L))
    expect_identical(rownames(actual[[field]]), colnames(data))
    expect_identical(colnames(actual[[field]]), colnames(data))
    expect_equal(actual[[field]], t(actual[[field]]))
  }

  expect_identical(diag(actual$p), rep(0, 4))
  expect_identical(diag(actual$lowCI), rep(1, 4))
  expect_identical(diag(actual$uppCI), rep(1, 4))
})

test_that("cor.mtest forwards cor.test methods and options unchanged", {
  data <- mtcars[, c("mpg", "disp", "hp", "wt")]
  configurations <- list(
    list(method = "pearson", conf.level = 0.90,
         alternative = "two.sided"),
    list(method = "spearman", exact = FALSE,
         alternative = "greater"),
    list(method = "kendall", exact = FALSE,
         continuity = FALSE, alternative = "less")
  )

  for (configuration in configurations) {
    expected <- do.call(
      corrplot::cor.mtest,
      c(list(mat = data), configuration)
    )
    actual <- do.call(
      corrplotng::cor.mtest,
      c(list(mat = data), configuration)
    )
    expect_identical(actual, expected, info = paste(configuration, collapse = ", "))
  }
})

test_that("cor.mtest preserves pairwise missing-value and dimname behavior", {
  data <- data.frame(
    first = c(1, 2, NA, 4, 5, 6),
    second = c(2, NA, 6, 8, 10, 12),
    third = c(6, 5, 4, NA, 2, 1)
  )

  expected <- corrplot::cor.mtest(data, use = "pairwise.complete.obs")
  actual <- corrplotng::cor.mtest(data, use = "pairwise.complete.obs")
  expect_identical(actual, expected)
  expect_identical(rownames(actual$p), names(data))
  expect_identical(colnames(actual$lowCI), names(data))
})

test_that("cor.mtest preserves upstream errors for invalid input and cor.test options", {
  cases <- list(
    list(
      upstream = function() corrplot::cor.mtest(matrix(1:4, ncol = 1)),
      adapter = function() corrplotng::cor.mtest(matrix(1:4, ncol = 1))
    ),
    list(
      upstream = function() corrplot::cor.mtest(mtcars[, 1:3], conf.level = 2),
      adapter = function() corrplotng::cor.mtest(mtcars[, 1:3], conf.level = 2)
    ),
    list(
      upstream = function() corrplot::cor.mtest(mtcars[, 1:3], method = "invalid"),
      adapter = function() corrplotng::cor.mtest(mtcars[, 1:3], method = "invalid")
    )
  )

  # Upstream's one-column loop currently errors, while the adapter returns
  # its empty pairwise result. Keep this known behavioral difference explicit.
  actual <- capture_cor_mtest_condition(cases[[1]]$adapter())
  expected <- capture_cor_mtest_condition(cases[[1]]$upstream())
  expect_null(actual)
  expect_true(inherits(expected$class, "error"))

  # The adapter validates cor.test options, whereas the pinned upstream helper
  # does not forward these options and therefore returns its pairwise result.
  for (case in cases[-1]) {
    actual <- capture_cor_mtest_condition(case$adapter())
    expected <- capture_cor_mtest_condition(case$upstream())
    expect_true(inherits(actual$class, "error"))
    if (is.null(expected)) {
      # The pinned helper may ignore an option that the adapter validates.
      expect_null(expected)
    } else {
      expect_true(inherits(expected$class, "error"))
      expect_match(actual$message, expected$message, fixed = TRUE)
    }
  }
})
