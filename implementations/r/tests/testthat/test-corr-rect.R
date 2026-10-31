with_corrrect_test_pdf <- function(code) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  device <- grDevices::dev.cur()
  on.exit({
    open_devices <- grDevices::dev.list()
    if (!is.null(open_devices) && device %in% open_devices) {
      grDevices::dev.off(which = device)
    }
    unlink(path)
  }, add = TRUE)
  force(code)
}

corrrect_test_matrix <- function() {
  matrix(
    c(
      1, 0.8, 0.1, 0.05,
      0.8, 1, 0.2, 0.1,
      0.1, 0.2, 1, 0.85,
      0.05, 0.1, 0.85, 1
    ),
    nrow = 4,
    byrow = TRUE,
    dimnames = list(LETTERS[1:4], LETTERS[1:4])
  )
}

test_that("corrRect index ranges draw on the active device and return corrRes invisibly", {
  corr <- corrrect_test_matrix()
  with_corrrect_test_pdf({
    result <- corrplot_ng(corr, order = "original", tl.pos = "n", cl.pos = "n")
    device_before <- grDevices::dev.cur()
    devices_before <- grDevices::dev.list()
    adapted <- withVisible(corrRect(result, index = c(1, 2, 4), col = "red", lwd = 2))

    expect_false(adapted$visible)
    expect_identical(adapted$value, result)
    expect_identical(grDevices::dev.cur(), device_before)
    expect_identical(grDevices::dev.list(), devices_before)
  })
})

test_that("index and name ranges produce equivalent shared cluster-region requests", {
  labels <- LETTERS[1:4]
  index_request <- corrplotng:::.corrplot_ng_region_inputs(
    starts = c(1L, 2L), ends = c(2L, 4L),
    labels = labels, source_indices = seq_along(labels),
    col = "black", lwd = 2
  )
  name_boundaries <- match(c("A", "B", "D"), labels)
  name_request <- corrplotng:::.corrplot_ng_region_inputs(
    starts = name_boundaries[-length(name_boundaries)],
    ends = name_boundaries[-1L],
    labels = labels, source_indices = seq_along(labels),
    col = "black", lwd = 2
  )

  expect_identical(name_request, index_request)
  expect_identical(index_request[[1]]$variables, list("A", "B"))
  expect_identical(index_request[[2]]$variables, list("B", "C", "D"))
  expect_identical(index_request[[1]]$color, "#000000")
  expect_equal(index_request[[1]]$line_width, 2)
})

test_that("corrRect name ranges and upper/lower layouts preserve supported drawing", {
  corr <- corrrect_test_matrix()
  with_corrrect_test_pdf({
    full <- corrplot_ng(corr, order = "original", tl.pos = "n", cl.pos = "n")
    expect_no_error(corrRect(full, name = c("A", "C", "D"), col = "#123456"))

    upper <- corrplot_ng(corr, type = "upper", order = "original",
      tl.pos = "n", cl.pos = "n")
    lower <- corrplot_ng(corr, type = "lower", order = "original",
      tl.pos = "n", cl.pos = "n")
    expect_no_error(corrRect(upper, index = c(1, 3, 4)))
    expect_no_error(corrRect(lower, index = c(1, 3, 4)))
  })
})

test_that("namesMat resolves coordinate pairs including differing row and column names", {
  corr <- corrrect_test_matrix()
  dimnames(corr) <- list(paste0("row-", LETTERS[1:4]),
    paste0("col-", LETTERS[1:4]))
  with_corrrect_test_pdf({
    result <- corrplot_ng(corr, tl.pos = "n", cl.pos = "n")
    diagonal <- c("col-A", "row-D", "col-D", "row-A")
    requests <- corrplotng:::.corrplot_ng_namesmat_region_inputs(
      matrix(diagonal, nrow = 1L), corr, result$corrPos, "black", 2
    )
    expect_length(requests, 0L)
    expect_no_error(corrRect(result, namesMat = diagonal, col = "purple"))
    expect_error(corrRect(result, namesMat = c("missing", "row-D", "col-D", "row-A")),
      "were not found")
  })
})

test_that("diagonal namesMat rectangles map to the shared cluster-region request shape", {
  corr <- corrrect_test_matrix()
  with_corrrect_test_pdf({
    result <- corrplot_ng(corr, tl.pos = "n", cl.pos = "n")
    names_mat <- c("A", "D", "D", "A")
    request <- corrplotng:::.corrplot_ng_namesmat_region_inputs(
      matrix(names_mat, nrow = 1L), corr, result$corrPos, "red", 1.5
    )
    expected <- corrplotng:::.corrplot_ng_region_inputs(
      starts = 1L, ends = 4L, labels = LETTERS[1:4],
      source_indices = 1:4, col = "red", lwd = 1.5
    )
    expect_identical(request, expected)
    expect_no_error(corrRect(result, namesMat = names_mat, col = "red", lwd = 1.5))
  })
})

test_that("corrRect rejects ambiguous inputs, invalid ranges, and unknown variables", {
  corr <- corrrect_test_matrix()
  with_corrrect_test_pdf({
    result <- corrplot_ng(corr, tl.pos = "n", cl.pos = "n")
    expect_error(corrRect(result), "supply exactly one")
    expect_error(corrRect(result, index = c(1, 1)), "strictly increasing")
    expect_error(corrRect(result, index = c(1, 5)), "within the matrix")
    expect_error(corrRect(result, index = c(1, 3), name = c("A", "C")),
      "supply exactly one")
    expect_error(corrRect(result, name = c("A", "Z")), "unique existing")
  })
})

test_that("corrRect.hclust maps leaf-order clusters to shared region declarations", {
  corr <- corrrect_test_matrix()
  with_corrrect_test_pdf({
    tree <- stats::hclust(stats::as.dist(1 - corr), method = "complete")
    cluster_data <- corrplotng:::.corrplot_ng_hclust_region_data(
      corr, k = 2, col = "blue", lwd = 2, method = "complete"
    )
    expect_equal(length(cluster_data$requests), 2L)
    expect_identical(cluster_data$order, as.integer(tree$order))
    expect_identical(
      unlist(lapply(cluster_data$requests, function(region) region$variables)),
      rownames(corr)[tree$order]
    )
    expect_no_error(corrRect.hclust(corr, k = 2, col = "blue", method = "complete"))
    expect_no_error(corrRect.hclust(corr, k = 1))
  })
})

test_that("corrRect.hclust validates matrices, cluster counts, and methods", {
  expect_error(corrRect.hclust(matrix(1:6, nrow = 2)), "square")
  expect_error(corrRect.hclust(corrrect_test_matrix(), k = 0), "between 1 and")
  expect_error(corrRect.hclust(corrrect_test_matrix(), k = 5), "between 1 and")
  expect_error(corrRect.hclust(corrrect_test_matrix(), method = "invalid"),
    "arg should be one of")
})
