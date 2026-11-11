#' Compute pairwise correlation p-values and confidence intervals
#'
#' Computes pairwise tests between input features using stats::cor.test().
#' Additional arguments are forwarded unchanged to cor.test(), including
#' method, alternative, exact, continuity, and conf.level.
#'
#' The result follows corrplot 0.95: diagonal p-values are zero and diagonal
#' confidence limits are one. Pairwise tests use complete observations. Pairs
#' with fewer than three complete observations or a constant feature return
#' NA for the p-value and both confidence limits. Methods that do not provide
#' confidence intervals leave off-diagonal confidence limits as NA.
#'
#' @param mat Numeric matrix or data frame. Rows are observations and columns
#'   are features. Non-numeric columns and infinite values are rejected.
#' @param ... Additional arguments passed to stats::cor.test().
#' @return A list containing symmetric square matrices p, lowCI, and uppCI.
#' @export
cor.mtest <- function(mat, ...) {
  if (is.data.frame(mat)) {
    numeric_columns <- vapply(mat, is.numeric, logical(1))
    if (any(!numeric_columns)) {
      column <- names(mat)[which(!numeric_columns)[[1L]]]
      if (is.null(column) || is.na(column) || !nzchar(column)) {
        column <- as.character(which(!numeric_columns)[[1L]])
      }
      stop("mat column '", column, "' must be numeric", call. = FALSE)
    }
    mat <- as.matrix(mat)
  } else if (!is.matrix(mat) || !is.numeric(mat)) {
    stop("mat must be a numeric matrix or a data frame with numeric columns",
      call. = FALSE)
  }

  if (!is.numeric(mat) || length(dim(mat)) != 2L ||
      nrow(mat) < 1L || ncol(mat) < 1L) {
    stop("mat must have at least one observation and one numeric feature",
      call. = FALSE)
  }
  infinite <- which(is.infinite(mat), arr.ind = TRUE)
  if (nrow(infinite)) {
    stop("mat contains an infinite value at row ", infinite[1L, "row"],
      ", column ", infinite[1L, "col"], call. = FALSE)
  }

  dots <- list(...)
  n <- ncol(mat)
  p <- low_ci <- upper_ci <- matrix(NA_real_, nrow = n, ncol = n)
  diag(p) <- 0
  diag(low_ci) <- diag(upper_ci) <- 1

  feature_names <- colnames(mat)
  if (is.null(feature_names)) feature_names <- as.character(seq_len(n))

  if (n >= 2L) {
    for (i in seq_len(n - 1L)) {
      for (j in seq.int(i + 1L, n)) {
        complete <- stats::complete.cases(mat[, i], mat[, j])
        x <- mat[complete, i]
        y <- mat[complete, j]

        if (length(x) < 3L || length(unique(x)) < 2L ||
            length(unique(y)) < 2L) {
          next
        }

        test <- tryCatch(
          do.call(stats::cor.test, c(list(x = x, y = y), dots)),
          error = function(error) {
            stop(
              "cor.test failed for features '", feature_names[[i]], "' and '",
              feature_names[[j]], "': ", conditionMessage(error),
              call. = FALSE
            )
          }
        )
        p[i, j] <- p[j, i] <- as.numeric(test$p.value)
        if (!is.null(test$conf.int)) {
          low_ci[i, j] <- low_ci[j, i] <- as.numeric(test$conf.int[[1L]])
          upper_ci[i, j] <- upper_ci[j, i] <- as.numeric(test$conf.int[[2L]])
        }
      }
    }
  }

  dimnames(p) <- dimnames(low_ci) <- dimnames(upper_ci) <-
    list(colnames(mat), colnames(mat))
  list(p = p, lowCI = low_ci, uppCI = upper_ci)
}
