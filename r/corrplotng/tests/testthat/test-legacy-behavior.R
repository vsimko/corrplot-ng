with_legacy_behavior_pdf <- function(code) {
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  on.exit({
    if (grDevices::dev.cur() > 1L) grDevices::dev.off()
    unlink(path)
  }, add = TRUE)
  force(code)
}

test_that("legacy corrplot argument validation matches the pinned 0.95 contract", {
  M <- stats::cor(datasets::mtcars)

  with_legacy_behavior_pdf({
    expect_silent(corrplotng::corrplot(M, col.lim = c(-1, 1)))
    expect_error(
      corrplotng::corrplot(M, col.lim = c(0, 1)),
      regexp = "color limits should cover matrix"
    )
    expect_error(
      corrplotng::corrplot(M, col.lim = c(-1, 2)),
      regexp = "col.lim should be within the interval"
    )

    expect_silent(corrplotng::corrplot(M, tl.pos = "td", type = "upper"))
    expect_error(
      corrplotng::corrplot(M, tl.pos = "td", type = "lower"),
      regexp = "type should be"
    )
    expect_silent(corrplotng::corrplot(M, tl.pos = "ld", type = "lower"))
    expect_error(
      corrplotng::corrplot(M, tl.pos = "ld", type = "upper"),
      regexp = "type should be"
    )
  })
})

test_that("legacy outline and number-label validation retain upstream errors", {
  M <- stats::cor(datasets::mtcars)

  with_legacy_behavior_pdf({
    for (unsupported in list(42, NA, NULL)) {
      expect_error(
        corrplotng::corrplot(M, outline = unsupported),
        regexp = "Unsupported value type for parameter outline"
      )
    }
    expect_error(corrplotng::corrplot(M, outline = ""), regexp = "invalid color name")

    for (digits in c(0, 1, 20)) {
      expect_silent(corrplotng::corrplot(M, method = "number", number.digits = digits))
    }
    expect_error(corrplotng::corrplot(M, number.digits = 1.2), regexp = "is not TRUE")
    expect_error(corrplotng::corrplot(M, number.digits = -1), regexp = "is not TRUE")
  })
})

test_that("legacy NA labels and non-correlation matrix mode retain upstream behavior", {
  M <- stats::cor(datasets::mtcars)
  diag(M) <- NA_real_

  with_legacy_behavior_pdf({
    expect_silent(corrplotng::corrplot(M, na.label = "NA"))
    expect_error(
      corrplotng::corrplot(M, na.label = "ABC"),
      regexp = "Maximum number of characters for NA label is: 2"
    )

    N <- matrix(seq_len(100), nrow = 10)
    expect_error(corrplotng::corrplot(N), regexp = "The matrix is not in")
    result <- corrplotng::corrplot(N, is.corr = FALSE)
    expect_true(is.matrix(result$corr))
    expect_true(is.data.frame(result$corrPos))
  })
})

test_that("legacy ordering modes produce the expected reordered matrix contract", {
  M <- stats::cor(datasets::mtcars)

  with_legacy_behavior_pdf({
    original <- corrplotng::corrplot(M)$corr
    expect_identical(original, M)

    for (method in c("AOE", "FPC", "hclust", "alphabet")) {
      reordered <- corrplotng::corrplot(M, order = method)$corr
      expect_true(is.matrix(reordered))
      expected_names <- rownames(corrplot::corrMatOrder(M, order = method))
      expect_identical(rownames(reordered), expected_names)
      expect_identical(colnames(reordered), expected_names)
    }

    expect_silent(corrplotng::corrplot(M, addrect = 2, order = "hclust", type = "full"))
  })
})

test_that("legacy significance and confidence-interval modes preserve their contracts", {
  M <- stats::cor(datasets::mtcars)
  pmat <- 1 - abs(M)^0.2
  ci <- corrplot::cor.mtest(datasets::mtcars, conf.level = 0.95)

  with_legacy_behavior_pdf({
    expect_silent(corrplotng::corrplot(
      M, p.mat = pmat, insig = "label_sig", pch = "!",
      sig.level = c(0.001, 0.1, 0.99)
    ))
    expect_silent(corrplotng::corrplot(
      M, p.mat = pmat, insig = "blank", type = "lower",
      method = "circle", diag = FALSE
    ))
    expect_warning(
      corrplotng::corrplot(M, p.mat = pmat[, 11:1], insig = "label_sig"),
      regexp = "."
    )

    expect_error(
      corrplotng::corrplot(M, p.mat = pmat, plotCI = "circle"),
      regexp = "Need lowCI.mat and uppCI.mat"
    )
    expect_silent(corrplotng::corrplot(
      M, p.mat = pmat, low = ci$lowCI, upp = ci$uppCI,
      plotCI = "circle", cl.pos = "n"
    ))
    expect_silent(corrplotng::corrplot(
      M, low = ci$lowCI, upp = ci$uppCI,
      plotCI = "square", order = "AOE", cl.pos = "n"
    ))
  })
})

test_that("legacy aspect-ratio restrictions remain explicit", {
  N <- matrix(stats::rnorm(70), ncol = 7)

  with_legacy_behavior_pdf({
    expect_silent(corrplotng::corrplot(
      N, is.corr = FALSE, win.asp = 0.7, method = "circle"
    ))
    expect_silent(corrplotng::corrplot(
      N, is.corr = FALSE, win.asp = 0.7, method = "square"
    ))
    expect_error(
      corrplotng::corrplot(N, is.corr = FALSE, win.asp = 0.7, method = "pie"),
      regexp = "supported only for circle and square methods"
    )
  })
})
