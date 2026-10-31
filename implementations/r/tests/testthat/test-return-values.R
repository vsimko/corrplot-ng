with_return_test_pdf <- function(code) {
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

return_contract_matrix <- function() {
  matrix(
    c(
      1, 0.12, 0.13,
      0.21, 1, 0.23,
      0.31, 0.32, 1
    ),
    nrow = 3,
    byrow = TRUE,
    dimnames = list(c("C", "A", "B"), c("C", "A", "B"))
  )
}

test_that("corrplot_ng preserves the upstream invisible return object and reordered matrix", {
  corr <- return_contract_matrix()
  with_return_test_pdf({
    direct <- withVisible(corrplot::corrplot(
      corr, order = "alphabet", type = "full", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    ))
    adapted <- withVisible(corrplot_ng(
      corr, order = "alphabet", type = "full", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    ))

    expect_identical(adapted$visible, FALSE)
    expect_identical(adapted, direct)
    expect_named(adapted$value, c("corr", "corrPos", "arg"))
    expect_identical(rownames(adapted$value$corr), c("A", "B", "C"))
    expect_identical(colnames(adapted$value$corr), c("A", "B", "C"))
    expect_identical(adapted$value$arg$type, "full")
  })
})

test_that("corrPos names and coordinates map to reordered matrix cells", {
  corr <- return_contract_matrix()
  with_return_test_pdf({
    result <- corrplot_ng(
      corr, order = "alphabet", type = "full", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    )
    positions <- result$corrPos[, c("xName", "yName", "x", "y", "corr")]
    expected <- data.frame(
      xName = c("A", "A", "B", "B", "C", "C"),
      yName = c("B", "C", "A", "C", "A", "B"),
      x = c(1, 1, 2, 2, 3, 3),
      y = c(2, 1, 3, 1, 3, 2),
      corr = c(0.32, 0.12, 0.23, 0.13, 0.21, 0.31),
      stringsAsFactors = FALSE
    )
    expect_equal(positions, expected)
    expect_null(rownames(result$corrPos))
  })
})

test_that("upper, lower, and diagonal-hidden return coordinates follow legacy filtering", {
  corr <- return_contract_matrix()
  with_return_test_pdf({
    upper <- corrplot_ng(
      corr, order = "alphabet", type = "upper", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    )
    lower <- corrplot_ng(
      corr, order = "alphabet", type = "lower", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    )

    expect_identical(upper$arg$type, "upper")
    expect_identical(lower$arg$type, "lower")
    expect_equal(nrow(upper$corrPos), 3L)
    expect_equal(nrow(lower$corrPos), 3L)
    expect_equal(upper$corrPos$x, c(2, 3, 3))
    expect_equal(upper$corrPos$y, c(3, 3, 2))
    expect_equal(lower$corrPos$x, c(1, 1, 2))
    expect_equal(lower$corrPos$y, c(2, 1, 1))
    expect_false(any(upper$corrPos$x + upper$corrPos$y == 4))
    expect_false(any(lower$corrPos$x + lower$corrPos$y == 4))
  })
})

test_that("corrPos includes p.value only when a p-value matrix is supplied", {
  corr <- return_contract_matrix()
  p_mat <- matrix(0.01, nrow = 3, ncol = 3, dimnames = dimnames(corr))
  with_return_test_pdf({
    without_p <- corrplot_ng(
      corr, order = "alphabet", type = "upper", diag = FALSE,
      tl.pos = "n", cl.pos = "n"
    )
    with_p <- corrplot_ng(
      corr, order = "alphabet", type = "upper", diag = FALSE,
      p.mat = p_mat, tl.pos = "n", cl.pos = "n"
    )
    expect_false("p.value" %in% names(without_p$corrPos))
    expect_true("p.value" %in% names(with_p$corrPos))
    expect_equal(with_p$corrPos$p.value, rep(0.01, 3))
  })
})

test_that("corrplot_ng_mixed preserves the combined invisible upstream return object", {
  corr <- return_contract_matrix()
  with_return_test_pdf({
    direct <- withVisible(corrplot::corrplot.mixed(
      corr, lower = "number", upper = "circle", diag = "n",
      tl.pos = "n", cl.pos = "n"
    ))
    adapted <- withVisible(corrplot_ng_mixed(
      corr, lower = "number", upper = "circle", diag = "n",
      tl.pos = "n", cl.pos = "n"
    ))

    expect_identical(adapted$visible, FALSE)
    expect_identical(adapted, direct)
    expect_named(adapted$value, c("corr", "corrPos"))
    expect_equal(nrow(adapted$value$corrPos), 9L)
    # Legacy mixed output retains the upper call's diagonal positions even
    # when diag = "n" later overdraws those glyphs with blank rectangles.
    expect_equal(sum(adapted$value$corrPos$x + adapted$value$corrPos$y == 4), 3L)
  })
})

test_that("corrplot_ng_mixed preserves legacy duplicate diagonal coordinates", {
  corr <- return_contract_matrix()
  with_return_test_pdf({
    result <- corrplot_ng_mixed(
      corr, lower = "number", upper = "circle", diag = "l",
      tl.pos = "n", cl.pos = "n"
    )
    expect_equal(nrow(result$corrPos), 12L)
    expect_equal(sum(result$corrPos$x + result$corrPos$y == 4), 6L)
  })
})

test_that("mixed scene rendering fails explicitly instead of changing return shape", {
  expect_error(
    corrplot_ng_mixed(return_contract_matrix(), engine = "scene"),
    "only engine = 'legacy'"
  )
})
