#' Normalize an R matrix or data frame to normalized-matrix v1
#'
#' This adapter accepts an already-computed square numeric matrix. It does not
#' calculate correlations from raw observations.
#'
#' Row and column names must match exactly when both are supplied. If only one
#' axis has names, those labels are used for both axes. If neither axis has
#' names, labels `V1` through `Vn` are generated. Names are preserved exactly,
#' including Unicode; empty, partially missing, or duplicate names are rejected.
#'
#' R `NA` values become `NULL` entries in the nested `values` list. Numeric
#' zero remains zero. `NaN`, infinities, non-numeric data-frame columns, and
#' non-square inputs are rejected.
#'
#' @param x A square numeric matrix or a data frame with only integer/double
#'   columns. A data frame is interpreted as an already-computed matrix, not as
#'   raw observations.
#' @param metadata Optional named list of JSON-compatible source metadata.
#' @return A list with class `corrplot_ng_normalized_matrix_v1`, conforming to
#'   `prototypes/normalized-matrix-v1/schema.json`. Source indices are zero-based.
#' @export
corrplot_ng_normalize_matrix <- function(x, metadata = NULL) {
  source_format <- NULL
  row_labels <- NULL
  column_labels <- NULL

  if (is.data.frame(x)) {
    source_format <- "r_data_frame"
    if (nrow(x) < 1L || ncol(x) < 1L) {
      stop("data frame must have at least one row and one column", call. = FALSE)
    }
    numeric_columns <- vapply(x, function(column) {
      is.numeric(column)
    }, logical(1))
    if (any(!numeric_columns)) {
      column_index <- which(!numeric_columns)[[1L]]
      column_name <- names(x)[[column_index]]
      if (is.na(column_name) || !nzchar(column_name)) {
        column_name <- paste0("V", column_index)
      }
      stop(
        "data frame column '", column_name,
        "' must contain integer or double values; no coercion is performed",
        call. = FALSE
      )
    }
    values <- do.call(cbind, lapply(x, as.numeric))
    if (is.null(dim(values))) {
      values <- matrix(values, ncol = ncol(x))
    }
    storage.mode(values) <- "double"
    row_labels <- rownames(x)
    if (!is.null(row_labels) &&
        identical(row_labels, as.character(seq_len(nrow(x))))) {
      row_labels <- NULL
    }
    column_labels <- names(x)
  } else if (is.matrix(x)) {
    source_format <- "r_matrix"
    if (!is.numeric(x)) {
      stop("matrix must contain integer or double values", call. = FALSE)
    }
    values <- x
    storage.mode(values) <- "double"
    row_labels <- rownames(x)
    column_labels <- colnames(x)
  } else {
    stop("x must be a numeric matrix or data frame", call. = FALSE)
  }

  n <- nrow(values)
  if (n < 1L || ncol(values) != n) {
    stop("input must be a non-empty square matrix", call. = FALSE)
  }

  validate_axis_labels <- function(labels, axis) {
    if (is.null(labels)) {
      return(NULL)
    }
    if (!is.character(labels) || length(labels) != n) {
      stop(axis, " labels must contain exactly ", n, " character values", call. = FALSE)
    }
    if (anyNA(labels)) {
      index <- which(is.na(labels))[[1L]]
      stop(axis, " label at index ", index, " must not be missing", call. = FALSE)
    }
    empty <- which(!nzchar(trimws(labels)))
    if (length(empty)) {
      stop(axis, " label at index ", empty[[1L]], " must not be empty", call. = FALSE)
    }
    duplicated <- which(duplicated(labels))
    if (length(duplicated)) {
      stop(axis, " labels must be unique; duplicate '", labels[[duplicated[[1L]]]], "'", call. = FALSE)
    }
    labels
  }

  row_labels <- validate_axis_labels(row_labels, "row")
  column_labels <- validate_axis_labels(column_labels, "column")
  if (!is.null(row_labels) && !is.null(column_labels) &&
      !identical(row_labels, column_labels)) {
    mismatch <- which(row_labels != column_labels)[[1L]]
    stop(
      "row and column labels must match exactly; first mismatch at index ",
      mismatch, call. = FALSE
    )
  }
  labels <- if (!is.null(row_labels)) {
    row_labels
  } else if (!is.null(column_labels)) {
    column_labels
  } else {
    paste0("V", seq_len(n))
  }

  nan_cells <- which(is.nan(values), arr.ind = TRUE)
  if (nrow(nan_cells)) {
    stop(
      "matrix value at row ", nan_cells[1L, "row"], ", column ",
      nan_cells[1L, "col"], " is NaN; use NA for missing values",
      call. = FALSE
    )
  }
  infinite_cells <- which(is.infinite(values), arr.ind = TRUE)
  if (nrow(infinite_cells)) {
    stop(
      "matrix value at row ", infinite_cells[1L, "row"], ", column ",
      infinite_cells[1L, "col"], " must be finite",
      call. = FALSE
    )
  }

  if (!is.null(metadata)) {
    if (!is.list(metadata) ||
        (length(metadata) > 0L &&
          (is.null(names(metadata)) || anyNA(names(metadata)) ||
            any(!nzchar(names(metadata))) || anyDuplicated(names(metadata))))) {
      stop("metadata must be a named list with unique non-empty names", call. = FALSE)
    }
  }

  normalized_values <- lapply(seq_len(n), function(row) {
    lapply(seq_len(n), function(column) {
      value <- values[row, column]
      if (is.na(value)) NULL else as.numeric(value)
    })
  })

  result <- list(
    schema_version = 1L,
    kind = "normalized_matrix",
    labels = labels,
    values = normalized_values,
    source = list(
      format = source_format,
      row_labels_present = !is.null(row_labels),
      column_labels_present = !is.null(column_labels)
    ),
    source_indices = as.list(as.integer(seq_len(n) - 1L))
  )
  if (!is.null(metadata) && length(metadata) > 0L) {
    result$metadata <- metadata
  }
  class(result) <- c("corrplot_ng_normalized_matrix_v1", "list")
  result
}
