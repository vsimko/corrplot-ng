test_that("COL1 and COL2 expose the pinned corrplot 0.95 interfaces", {
  expect_true(all(c("COL1", "COL2") %in% getNamespaceExports("corrplotng")))
  expect_identical(names(formals(corrplotng::COL1)), names(formals(corrplot::COL1)))
  expect_identical(names(formals(corrplotng::COL2)), names(formals(corrplot::COL2)))
  expect_identical(lapply(formals(corrplotng::COL1), deparse),
                   lapply(formals(corrplot::COL1), deparse))
  expect_identical(lapply(formals(corrplotng::COL2), deparse),
                   lapply(formals(corrplot::COL2), deparse))
})

test_that("all legacy palette families match the pinned upstream outputs", {
  fixture <- jsonlite::fromJSON(test_path("fixtures", "legacy-palettes.json"))
  expect_identical(fixture$reference$version,
                   as.character(utils::packageVersion("corrplot")))
  expect_identical(fixture$reference$commit,
                   "62532f700e6c9d04b93ffbf0bf56e264b0e0368e")

  for (name in fixture$sequential) {
    for (n in fixture$lengths) {
      expect_identical(corrplotng::COL1(name, n), corrplot::COL1(name, n),
                       info = paste("COL1", name, n))
    }
  }
  for (name in fixture$diverging) {
    for (n in fixture$lengths) {
      expect_identical(corrplotng::COL2(name, n), corrplot::COL2(name, n),
                       info = paste("COL2", name, n))
    }
  }
})

test_that("palette helpers validate names and positive integer lengths", {
  expect_error(COL1("not-a-palette"), "arg")
  expect_error(COL2("not-a-palette"), "arg")
  for (n in list(0, -1, 1.5, NA_real_, Inf, c(2, 3), "3")) {
    expect_error(COL1(n = n), "positive integer")
    expect_error(COL2(n = n), "positive integer")
  }
  expect_length(COL1(n = 1), 1L)
  expect_length(COL2(n = 1), 1L)
})
