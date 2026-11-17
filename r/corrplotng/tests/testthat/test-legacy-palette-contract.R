capture_palette_condition <- function(expr) {
  tryCatch(
    {
      force(expr)
      NULL
    },
    error = function(error) {
      list(
        class = class(error),
        message = conditionMessage(error)
      )
    }
  )
}

test_that("COL1 and COL2 reproduce every pinned palette at representative sizes", {
  sequential <- c(
    "Oranges", "Purples", "Reds", "Blues", "Greens",
    "Greys", "OrRd", "YlOrRd", "YlOrBr", "YlGn"
  )
  diverging <- c("RdBu", "BrBG", "PiYG", "PRGn", "PuOr", "RdYlBu")
  sizes <- c(1L, 2L, 9L, 11L, 200L)

  for (palette in sequential) {
    for (n in sizes) {
      expected <- corrplot::COL1(sequential = palette, n = n)
      actual <- corrplotng::COL1(sequential = palette, n = n)
      context <- paste("COL1", palette, "n =", n)
      expect_identical(actual, expected, info = context)
      expect_type(actual, "character", info = context)
      expect_length(actual, n, info = context)
    }
  }

  for (palette in diverging) {
    for (n in sizes) {
      expected <- corrplot::COL2(diverging = palette, n = n)
      actual <- corrplotng::COL2(diverging = palette, n = n)
      context <- paste("COL2", palette, "n =", n)
      expect_identical(actual, expected, info = context)
      expect_type(actual, "character", info = context)
      expect_length(actual, n, info = context)
    }
  }
})

test_that("COL1 and COL2 preserve pinned defaults and match.arg behavior", {
  expect_identical(corrplotng::COL1(), corrplot::COL1())
  expect_identical(corrplotng::COL2(), corrplot::COL2())

  cases <- list(
    list(
      upstream = function() corrplot::COL1(sequential = "not-a-palette"),
      adapter = function() corrplotng::COL1(sequential = "not-a-palette")
    ),
    list(
      upstream = function() corrplot::COL2(diverging = "not-a-palette"),
      adapter = function() corrplotng::COL2(diverging = "not-a-palette")
    ),
    list(
      upstream = function() corrplot::COL1(sequential = "O"),
      adapter = function() corrplotng::COL1(sequential = "O")
    ),
    list(
      upstream = function() corrplot::COL2(diverging = "R"),
      adapter = function() corrplotng::COL2(diverging = "R")
    )
  )

  for (case in cases) {
    expect_identical(
      capture_palette_condition(case$adapter()),
      capture_palette_condition(case$upstream())
    )
  }
})
