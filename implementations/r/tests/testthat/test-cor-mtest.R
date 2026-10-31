test_that("cor.mtest returns symmetric named matrices with legacy diagonal values", {
  mat <- matrix(c(1, 2, 4, 2, 4, 8, 3, 6, 12, 4, 8, 16),
    nrow = 4, dimnames = list(NULL, c("A", "B", "C")))
  result <- cor.mtest(mat)

  expect_named(result, c("p", "lowCI", "uppCI"))
  for (component in result) {
    expect_equal(dim(component), c(3L, 3L))
    expect_identical(rownames(component), colnames(mat))
    expect_identical(colnames(component), colnames(mat))
    expect_equal(component, t(component))
  }
  expect_equal(diag(result$p), c(0, 0, 0))
  expect_equal(diag(result$lowCI), c(1, 1, 1))
  expect_equal(diag(result$uppCI), c(1, 1, 1))
})

test_that("Pearson p-values and confidence intervals agree with stats::cor.test", {
  mat <- cbind(
    alpha = c(1, 2, 4, 3, 7, 6, 8, 9),
    beta = c(2, 1, 3, 5, 4, 8, 7, 10),
    gamma = c(8, 7, 5, 6, 2, 3, 1, 0)
  )
  result <- cor.mtest(mat, method = "pearson", alternative = "two.sided",
    conf.level = 0.9)
  reference <- stats::cor.test(mat[, 1], mat[, 2], method = "pearson",
    alternative = "two.sided", conf.level = 0.9)

  expect_equal(result$p[1, 2], reference$p.value)
  expect_equal(result$lowCI[1, 2], reference$conf.int[[1]])
  expect_equal(result$uppCI[1, 2], reference$conf.int[[2]])
  expect_equal(result$p[1, 2], result$p[2, 1])
  expect_equal(result$lowCI[1, 2], result$lowCI[2, 1])
  expect_equal(result$uppCI[1, 2], result$uppCI[2, 1])
})

test_that("Spearman and Kendall methods forward arguments and leave CIs missing", {
  mat <- cbind(A = c(1, 3, 2, 5, 4, 7, 6), B = c(7, 5, 6, 3, 4, 1, 2))
  for (method in c("spearman", "kendall")) {
    result <- cor.mtest(mat, method = method, exact = FALSE,
      alternative = "greater")
    reference <- stats::cor.test(mat[, 1], mat[, 2], method = method,
      exact = FALSE, alternative = "greater")
    expect_equal(result$p[1, 2], reference$p.value, info = method)
    expect_true(is.na(result$lowCI[1, 2]), info = method)
    expect_true(is.na(result$uppCI[1, 2]), info = method)
    expect_equal(diag(result$lowCI), c(1, 1), info = method)
  }
})

test_that("pairwise missing observations are removed independently", {
  mat <- cbind(
    A = c(1, 2, NA, 4, 5, 6, 7),
    B = c(2, NA, 3, 5, 4, 7, 8),
    C = c(NA, 4, 2, 1, 3, 5, 6)
  )
  result <- cor.mtest(mat, method = "pearson")
  for (i in 1:2) {
    for (j in (i + 1):3) {
      complete <- stats::complete.cases(mat[, i], mat[, j])
      reference <- stats::cor.test(mat[complete, i], mat[complete, j])
      expect_equal(result$p[i, j], reference$p.value)
      expect_equal(result$lowCI[i, j], reference$conf.int[[1]])
      expect_equal(result$uppCI[i, j], reference$conf.int[[2]])
    }
  }
})

test_that("constant columns and insufficient complete pairs return missing results", {
  mat <- cbind(
    constant = rep(1, 5),
    varying = c(1, 2, 3, 4, 5),
    sparse = c(1, 2, NA, NA, NA)
  )
  result <- cor.mtest(mat)
  expect_true(is.na(result$p["constant", "varying"]))
  expect_true(is.na(result$lowCI["constant", "varying"]))
  expect_true(is.na(result$uppCI["constant", "varying"]))
  expect_true(is.na(result$p["varying", "sparse"]))
  expect_true(is.na(result$lowCI["varying", "sparse"]))
  expect_true(is.na(result$uppCI["varying", "sparse"]))
  expect_equal(diag(result$p), c(0, 0, 0))
})

test_that("single-feature inputs retain legacy diagonal conventions", {
  result <- cor.mtest(matrix(1:5, ncol = 1,
    dimnames = list(NULL, "only")))
  expect_identical(dim(result$p), c(1L, 1L))
  expect_identical(rownames(result$p), "only")
  expect_equal(result$p[1, 1], 0)
  expect_equal(result$lowCI[1, 1], 1)
  expect_equal(result$uppCI[1, 1], 1)
})

test_that("numeric data frames are supported and non-numeric columns are rejected", {
  result <- cor.mtest(data.frame(A = 1:5, B = c(2, 4, 1, 5, 3)))
  expect_equal(dim(result$p), c(2L, 2L))
  expect_error(cor.mtest(data.frame(A = 1:5, B = letters[1:5])),
    "column 'B' must be numeric")
})

test_that("invalid inputs and cor.test failures have actionable errors", {
  expect_error(cor.mtest(matrix(numeric(), nrow = 0, ncol = 2)),
    "at least one observation")
  expect_error(cor.mtest(as.logical(matrix(1:5, ncol = 1))),
    "numeric matrix")
  expect_error(cor.mtest(matrix(c(1, Inf, 2, 4), 2, byrow = TRUE)),
    "infinite value at row 1, column 2")
  mat <- cbind(alpha = 1:5, beta = 5:1)
  expect_error(cor.mtest(mat, method = "unsupported"),
    "cor.test failed for features 'alpha' and 'beta'")
})

test_that("ordinary Pearson results match the pinned corrplot 0.95 helper", {
  skip_if_not("cor.mtest" %in% getNamespaceExports("corrplot"))
  mat <- cbind(
    A = c(1, 2, 4, 3, 7, 6, 8, 9),
    B = c(2, 1, 3, 5, 4, 8, 7, 10),
    C = c(8, 7, 5, 6, 2, 3, 1, 0)
  )
  actual <- cor.mtest(mat, method = "pearson", conf.level = 0.9)
  reference <- corrplot::cor.mtest(mat, method = "pearson", conf.level = 0.9)
  expect_equal(actual$p, reference$p)
  expect_equal(actual$lowCI, reference$lowCI)
  expect_equal(actual$uppCI, reference$uppCI)
})
