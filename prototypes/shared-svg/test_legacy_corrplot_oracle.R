# Compatibility oracle against the pinned legacy corrplot release.
# This intentionally tests semantic output, not device-specific graphics.
suppressPackageStartupMessages(library(corrplot))

m <- matrix(
  c(1, 0.25, NA,
    0.25, 1, -0.5,
    NA, -0.5, 1),
  nrow = 3,
  byrow = TRUE,
  dimnames = list(c("gamma", "alpha", "beta"),
                  c("gamma", "alpha", "beta"))
)

grDevices::pdf(NULL)
res <- corrplot(
  m,
  method = "circle",
  order = "original",
  type = "full",
  diag = TRUE,
  tl.pos = "n",
  cl.pos = "n",
  addgrid.col = NA
)

stopifnot(identical(res$corr, m))
stopifnot(identical(
  names(res$corrPos),
  c("xName", "yName", "x", "y", "corr")
))

# Legacy corrPos is ordered by x, then top-to-bottom y. It excludes NA cells;
# the returned matrix remains the authoritative source for missing values.
expected <- expand.grid(
  row = seq_len(nrow(m)),
  column = seq_len(ncol(m))
)
expected <- expected[order(expected$column, expected$row), ]
expected <- expected[!is.na(m[cbind(expected$row, expected$column)]), ]
expected$xName <- colnames(m)[expected$column]
expected$yName <- rownames(m)[expected$row]
expected$x <- expected$column
expected$y <- nrow(m) - expected$row + 1
expected$corr <- m[cbind(expected$row, expected$column)]

actual <- res$corrPos
stopifnot(identical(actual$xName, expected$xName))
stopifnot(identical(actual$yName, expected$yName))
stopifnot(isTRUE(all.equal(actual$x, expected$x)))
stopifnot(isTRUE(all.equal(actual$y, expected$y)))
stopifnot(isTRUE(all.equal(actual$corr, expected$corr)))

# Reconstruct a row-major matrix from the sparse legacy corrPos table.
# The coordinates are one-based with y increasing bottom-to-top; NA cells
# are absent from corrPos and must be restored as missing, not as zero.
reconstructed <- matrix(
  NA_real_,
  nrow = nrow(m),
  ncol = ncol(m),
  dimnames = dimnames(m)
)
for (i in seq_len(nrow(actual))) {
  row <- nrow(m) - actual$y[i] + 1
  column <- actual$x[i]
  reconstructed[row, column] <- actual$corr[i]
}
stopifnot(isTRUE(all.equal(reconstructed, res$corr)))
legacy_row_major <- as.vector(t(reconstructed))
stopifnot(identical(
  legacy_row_major,
  as.vector(t(res$corr))
))

# Ordering is performed before coordinates and names are returned.
ordered <- corrplot(
  m,
  method = "circle",
  order = "alphabet",
  type = "full",
  diag = TRUE,
  tl.pos = "n",
  cl.pos = "n",
  addgrid.col = NA
)
permutation <- match(c("alpha", "beta", "gamma"), rownames(m))
expected_ordered <- m[permutation, permutation, drop = FALSE]
stopifnot(identical(rownames(ordered$corr), c("alpha", "beta", "gamma")))
stopifnot(identical(colnames(ordered$corr), c("alpha", "beta", "gamma")))
stopifnot(isTRUE(all.equal(ordered$corr, expected_ordered)))
