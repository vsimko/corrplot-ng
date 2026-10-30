test_that("legacy API inventory is pinned and covers every upstream export", {
  inventory_path <- testthat::test_path("fixtures", "legacy-api-inventory.json")
  inventory <- jsonlite::fromJSON(inventory_path, simplifyVector = FALSE)

  expect_identical(inventory$schema_version, 1L)
  expect_identical(inventory$reference$package, "corrplot")
  expect_identical(inventory$reference$version, "0.95")
  expect_identical(
    inventory$reference$ref,
    "62532f700e6c9d04b93ffbf0bf56e264b0e0368e"
  )

  expected_exports <- vapply(inventory$functions, `[[`, character(1), "name")
  actual_exports <- getNamespaceExports(asNamespace("corrplot"))
  expect_setequal(actual_exports, expected_exports)

  for (entry in inventory$functions) {
    fun <- getExportedValue("corrplot", entry$name)
    actual_formals <- names(formals(fun))
    expect_identical(
      actual_formals,
      unlist(entry$formals, use.names = FALSE),
      info = paste("Formal arguments differ for upstream export:", entry$name)
    )
  }
})

test_that("legacy API inventory records the full documented plot vocabulary", {
  inventory <- jsonlite::fromJSON(
    testthat::test_path("fixtures", "legacy-api-inventory.json"),
    simplifyVector = FALSE
  )
  plot_api <- inventory$functions[[which(vapply(
    inventory$functions, `[[`, character(1), "name"
  ) == "corrplot")]]

  expect_setequal(
    unlist(plot_api$accepted_values$method, use.names = FALSE),
    c("circle", "square", "ellipse", "number", "shade", "color", "pie")
  )
  expect_setequal(
    unlist(plot_api$accepted_values$type, use.names = FALSE),
    c("full", "upper", "lower")
  )
  expect_setequal(
    unlist(plot_api$accepted_values$order, use.names = FALSE),
    c("original", "AOE", "FPC", "hclust", "alphabet")
  )
})
