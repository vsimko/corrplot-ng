#' Normalize a CSV, TSV, or normalized-matrix JSON file
#'
#' CSV and TSV files use the publication-config matrix layout: optional column
#' headers and optional row labels. When both are present, the first header cell
#' is the corner cell and row labels must match column labels in order.
#' Unlabelled axes receive deterministic V1...Vn labels. JSON input must already
#' conform to normalized-matrix v1.
#'
#' @param path Path to a UTF-8 input file.
#' @param format Input format: auto (by extension), csv, tsv, or json.
#' @param header Whether CSV/TSV contains column labels.
#' @param row_labels Whether the first CSV/TSV column contains row labels.
#' @return A normalized-matrix v1 object.
#' @export
corrplot_ng_normalize_file <- function(path, format = c("auto", "csv", "tsv", "json"),
                                       header = TRUE, row_labels = TRUE) {
  if (!is.character(path) || length(path) != 1L || is.na(path) || !nzchar(path)) {
    stop("path must be one non-empty file path", call. = FALSE)
  }
  if (!file.exists(path) || dir.exists(path)) {
    stop("input file does not exist or is not a regular file: ", path, call. = FALSE)
  }
  format <- match.arg(format)
  if (format == "auto") {
    extension <- tolower(tools::file_ext(path))
    format <- switch(extension, csv = "csv", tsv = "tsv", json = "json",
      stop("cannot infer input format from file extension; specify format", call. = FALSE))
  }
  if (!is.logical(header) || length(header) != 1L || is.na(header) ||
      !is.logical(row_labels) || length(row_labels) != 1L || is.na(row_labels)) {
    stop("header and row_labels must be single non-missing logical values", call. = FALSE)
  }

  if (format == "json") {
    if (!requireNamespace("jsonlite", quietly = TRUE)) {
      stop("package 'jsonlite' is required to read normalized-matrix JSON", call. = FALSE)
    }
    input <- jsonlite::fromJSON(path, simplifyVector = FALSE)
    if (!is.list(input) ||
        !(identical(input$schema_version, 1L) || identical(input$schema_version, 1)) ||
        !identical(input$kind, "normalized_matrix")) {
      stop("JSON input must be a normalized-matrix v1 object", call. = FALSE)
    }
    labels <- unlist(input$labels, use.names = FALSE)
    if (!is.character(labels) || !length(labels)) {
      stop("JSON labels must be a non-empty string array", call. = FALSE)
    }
    if (length(input$values) != length(labels)) {
      stop("JSON values must have one row per label", call. = FALSE)
    }
    values <- matrix(NA_real_, nrow = length(labels), ncol = length(labels))
    for (r in seq_along(labels)) {
      if (length(input$values[[r]]) != length(labels)) {
        stop("JSON values[", r, "] must contain exactly ", length(labels), " columns", call. = FALSE)
      }
      for (column in seq_along(labels)) {
        cell <- input$values[[r]][[column]]
        if (!is.null(cell)) {
          if (!is.numeric(cell) || length(cell) != 1L || !is.finite(cell)) {
            stop("JSON values[", r, "][", column, "] must be a finite number or null", call. = FALSE)
          }
          values[r, column] <- cell
        }
      }
    }
    dimnames(values) <- list(labels, labels)
    source <- input$source
    if (!is.list(source) || !identical(source$format, "json") ||
        !is.logical(source$row_labels_present) || length(source$row_labels_present) != 1L ||
        !is.logical(source$column_labels_present) || length(source$column_labels_present) != 1L) {
      stop("JSON source must contain format='json' and logical row/column label flags", call. = FALSE)
    }
    indices <- unlist(input$source_indices, use.names = FALSE)
    if (length(indices) != length(labels) || !is.numeric(indices) ||
        anyNA(indices) || any(!is.finite(indices)) || any(indices < 0) ||
        any(indices != floor(indices)) || anyDuplicated(indices)) {
      stop("JSON source_indices must contain one unique non-negative integer per label", call. = FALSE)
    }
    if (!is.null(input$metadata) && !is.list(input$metadata)) {
      stop("JSON metadata must be an object", call. = FALSE)
    }
    result <- corrplot_ng_normalize_matrix(values)
    result$source <- source
    result$source$path <- normalizePath(path, winslash = "/", mustWork = TRUE)
    result$source_indices <- as.list(as.integer(indices))
    if (!is.null(input$metadata)) result$metadata <- input$metadata
    return(result)
  }

  delimiter <- if (format == "csv") "," else "\t"
  table <- tryCatch(
    utils::read.table(
      file = path, header = header, sep = delimiter,
      row.names = if (row_labels) 1L else NULL,
      colClasses = "character", check.names = FALSE,
      stringsAsFactors = FALSE, quote = "\"",
      comment.char = "", fill = FALSE, na.strings = c("", "NA", "null"),
      strip.white = FALSE
    ),
    error = function(error) {
      stop("cannot parse ", toupper(format), " matrix: ", conditionMessage(error), call. = FALSE)
    }
  )
  if (!nrow(table) || !ncol(table) || nrow(table) != ncol(table)) {
    stop("file input must contain a non-empty square matrix", call. = FALSE)
  }

  labels_from_columns <- if (header) names(table) else NULL
  labels_from_rows <- if (row_labels) rownames(table) else NULL
  if (!header) colnames(table) <- NULL
  if (!row_labels) rownames(table) <- NULL

  raw <- as.matrix(table)
  values <- matrix(NA_real_, nrow = nrow(raw), ncol = ncol(raw))
  for (r in seq_len(nrow(raw))) {
    for (column in seq_len(ncol(raw))) {
      cell <- raw[r, column]
      if (is.na(cell) || cell == "" || cell == "NA" || cell == "null") next
      number <- suppressWarnings(as.numeric(cell))
      if (is.na(number) && !is.nan(number)) {
        stop("non-numeric value at row ", r, ", column ", column, ": '", cell, "'", call. = FALSE)
      }
      values[r, column] <- number
    }
  }
  dimnames(values) <- list(labels_from_rows, labels_from_columns)
  result <- corrplot_ng_normalize_matrix(values)
  result$source$format <- format
  result$source$row_labels_present <- row_labels
  result$source$column_labels_present <- header
  result$source$path <- normalizePath(path, winslash = "/", mustWork = TRUE)
  result
}
