capture_corrmatorder_condition <- function(expr) {
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

test_that("corrMatOrder matches pinned AOE, FPC, and alphabetical ordering", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat", "wt", "qsec")])
  expected_names <- rownames(data)

  for (ordering in c("AOE", "FPC", "alphabet")) {
    expected <- corrplot::corrMatOrder(data, order = ordering)
    actual <- corrplotng::corrMatOrder(data, order = ordering)
    expect_identical(actual, expected, info = ordering)
    if (ordering == "alphabet") {
      expect_identical(actual, sort(expected_names))
    } else {
      expect_type(actual, "integer")
      expect_setequal(actual, seq_len(nrow(data)))
    }
  }
})

test_that("corrMatOrder matches pinned output for every hierarchical linkage", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat", "wt", "qsec")])
  methods <- c(
    "complete", "ward", "ward.D", "ward.D2", "single",
    "average", "mcquitty", "median", "centroid"
  )

  for (method in methods) {
    expected <- corrplot::corrMatOrder(data, order = "hclust",
                                       hclust.method = method)
    actual <- corrplotng::corrMatOrder(data, order = "hclust",
                                       hclust.method = method)
    expect_identical(actual, expected, info = method)
    expect_type(actual, "integer")
    expect_setequal(actual, seq_len(nrow(data)))
  }
})

test_that("corrMatOrder preserves pinned defaults and match.arg errors", {
  data <- stats::cor(mtcars[, c("mpg", "disp", "hp", "drat")])
  expect_identical(corrplotng::corrMatOrder(data),
                   corrplot::corrMatOrder(data))

  cases <- list(
    list(
      upstream = function() corrplot::corrMatOrder(data, order = "invalid"),
      adapter = function() corrplotng::corrMatOrder(data, order = "invalid")
    ),
    list(
      upstream = function() corrplot::corrMatOrder(data, order = "hclust",
                                                   hclust.method = "invalid"),
      adapter = function() corrplotng::corrMatOrder(data, order = "hclust",
                                                    hclust.method = "invalid")
    ),
    list(
      upstream = function() corrplot::corrMatOrder(data, order = "a"),
      adapter = function() corrplotng::corrMatOrder(data, order = "a")
    )
  )

  for (case in cases) {
    expect_identical(
      capture_corrmatorder_condition(case$adapter()),
      capture_corrmatorder_condition(case$upstream())
    )
  }
})
