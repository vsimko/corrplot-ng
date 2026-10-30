# Compatibility oracle against the pinned legacy corrplot release.
# This intentionally tests semantic output, not device-specific graphics.
suppressPackageStartupMessages(library(corrplot))

plot_legacy <- function(correlation, order = "original") {
  grDevices::pdf(NULL)
  corrplot(
    correlation,
    method = "circle",
    order = order,
    type = "full",
    diag = TRUE,
    tl.pos = "n",
    cl.pos = "n",
    addgrid.col = NA
  )
}

verify_legacy_result <- function(correlation, result) {
  stopifnot(isTRUE(all.equal(result$corr, correlation)))
  stopifnot(identical(
    names(result$corrPos),
    c("xName", "yName", "x", "y", "corr")
  ))

  # Legacy corrPos is ordered by x, then top-to-bottom y. It excludes NA
  # cells; the returned matrix remains authoritative for missing values.
  expected <- expand.grid(
    row = seq_len(nrow(correlation)),
    column = seq_len(ncol(correlation))
  )
  expected <- expected[order(expected$column, expected$row), ]
  expected <- expected[
    !is.na(correlation[cbind(expected$row, expected$column)]),
  ]
  expected$xName <- colnames(correlation)[expected$column]
  expected$yName <- rownames(correlation)[expected$row]
  expected$x <- expected$column
  expected$y <- nrow(correlation) - expected$row + 1
  expected$corr <- correlation[cbind(expected$row, expected$column)]

  actual <- result$corrPos
  stopifnot(identical(actual$xName, expected$xName))
  stopifnot(identical(actual$yName, expected$yName))
  stopifnot(isTRUE(all.equal(actual$x, expected$x)))
  stopifnot(isTRUE(all.equal(actual$y, expected$y)))
  stopifnot(isTRUE(all.equal(actual$corr, expected$corr)))

  reconstructed <- matrix(
    NA_real_,
    nrow = nrow(correlation),
    ncol = ncol(correlation),
    dimnames = dimnames(correlation)
  )
  for (i in seq_len(nrow(actual))) {
    row <- nrow(correlation) - actual$y[i] + 1
    column <- actual$x[i]
    reconstructed[row, column] <- actual$corr[i]
  }
  stopifnot(isTRUE(all.equal(reconstructed, result$corr)))
  stopifnot(identical(
    as.vector(t(reconstructed)),
    as.vector(t(result$corr))
  ))
  invisible(TRUE)
}

m <- matrix(
  c(1, 0.25, NA,
    0.25, 1, -0.5,
    NA, -0.5, 1),
  nrow = 3,
  byrow = TRUE,
  dimnames = list(c("gamma", "alpha", "beta"),
                  c("gamma", "alpha", "beta"))
)

res <- plot_legacy(m)
verify_legacy_result(m, res)

# Ordering is performed before coordinates and names are returned.
ordered <- plot_legacy(m, order = "alphabet")
permutation <- match(c("alpha", "beta", "gamma"), rownames(m))
expected_ordered <- m[permutation, permutation, drop = FALSE]
stopifnot(identical(rownames(ordered$corr), c("alpha", "beta", "gamma")))
stopifnot(identical(colnames(ordered$corr), c("alpha", "beta", "gamma")))
verify_legacy_result(expected_ordered, ordered)

# Exercise different matrix dimensions through the real legacy plotting API.
small <- matrix(
  c(1, -0.35, -0.35, 1),
  nrow = 2,
  byrow = TRUE,
  dimnames = list(c("zeta", "eta"), c("zeta", "eta"))
)
small_result <- plot_legacy(small)
verify_legacy_result(small, small_result)

large <- matrix(
  c(1, 0.2, NA, -0.1,
    0.2, 1, 0.45, 0,
    NA, 0.45, 1, -0.6,
    -0.1, 0, -0.6, 1),
  nrow = 4,
  byrow = TRUE,
  dimnames = list(c("delta", "alpha", "gamma", "beta"),
                  c("delta", "alpha", "gamma", "beta"))
)
large_result <- plot_legacy(large)
verify_legacy_result(large, large_result)

# Perfect positive/negative correlations and a constant-like variable.
# The third variable has no defined off-diagonal correlations (NA), while
# its diagonal remains 1 as in a conventional correlation matrix.
extremes <- matrix(
  c(1, 1, -1, NA,
    1, 1, -1, NA,
    -1, -1, 1, NA,
    NA, NA, NA, 1),
  nrow = 4,
  byrow = TRUE,
  dimnames = list(c("positive_a", "positive_b", "negative", "constant"),
                  c("positive_a", "positive_b", "negative", "constant"))
)
extremes_result <- plot_legacy(extremes)
verify_legacy_result(extremes, extremes_result)
stopifnot(extremes_result$corrPos$corr[extremes_result$corrPos$xName == "positive_b" &
                                         extremes_result$corrPos$yName == "positive_a"] == 1)
stopifnot(extremes_result$corrPos$corr[extremes_result$corrPos$xName == "negative" &
                                         extremes_result$corrPos$yName == "positive_a"] == -1)
stopifnot(!("constant" %in% extremes_result$corrPos$xName[
  extremes_result$corrPos$yName != "constant"
]))

# An entirely missing variable, including its diagonal, probes the legacy
# behavior for a fully undefined row/column rather than assuming it is valid.
all_missing <- matrix(
  c(1, 0.3, NA,
    0.3, 1, NA,
    NA, NA, NA),
  nrow = 3,
  byrow = TRUE,
  dimnames = list(c("observed_a", "observed_b", "all_missing"),
                  c("observed_a", "observed_b", "all_missing"))
)
all_missing_result <- plot_legacy(all_missing)
verify_legacy_result(all_missing, all_missing_result)

# Export legacy results as language-neutral scene inputs.
if (!requireNamespace("jsonlite", quietly = TRUE)) {
  stop("jsonlite is required to export the legacy scene inputs")
}

make_scene_input <- function(correlation, annotation) {
  scene_matrix <- lapply(seq_len(nrow(correlation)), function(row) {
    lapply(seq_len(ncol(correlation)), function(column) {
      value <- correlation[row, column]
      if (is.na(value)) NULL else unname(as.numeric(value))
    })
  })
  list(
    labels = as.list(rownames(correlation)),
    matrix = scene_matrix,
    annotation = annotation
  )
}

legacy_scene_inputs <- list(
  cases = list(
    three_original_order = make_scene_input(
      res$corr,
      "Legacy 3x3 correlation matrix (original order)"
    ),
    three_alphabetic_order = make_scene_input(
      ordered$corr,
      "Legacy 3x3 correlation matrix (alphabetic order)"
    ),
    two_variables = make_scene_input(
      small_result$corr,
      "Legacy 2x2 correlation matrix"
    ),
    four_variables_with_missing = make_scene_input(
      large_result$corr,
      "Legacy 4x4 correlation matrix with missing values"
    ),
    perfect_correlations_and_constant = make_scene_input(
      extremes_result$corr,
      "Perfect positive/negative correlations and constant-like variable"
    ),
    fully_missing_variable = make_scene_input(
      all_missing_result$corr,
      "Legacy matrix with an entirely missing variable"
    )
  )
)
jsonlite::write_json(
  legacy_scene_inputs,
  path = "/tmp/corrplot-legacy-scene-input.json",
  auto_unbox = TRUE,
  null = "null",
  digits = NA,
  pretty = TRUE
)
